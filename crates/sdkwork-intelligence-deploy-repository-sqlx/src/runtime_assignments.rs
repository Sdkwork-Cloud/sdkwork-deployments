use std::borrow::Cow;

use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_deploy_web_port::RuntimeAssignmentReceipt;
use sdkwork_intelligence_deploy_service::runtime_publication::{
    DeployRuntimeAssignmentMutationPort, PersistRuntimeAssignmentCommand,
    RuntimeAssignmentPublishStatus, RuntimeAssignmentState,
};
use sqlx::{any::AnyRow, Any, AnyPool, Row, Transaction};

use crate::support::{next_id, store_error};
use crate::DeployRepository;

const MAXIMUM_RUNTIME_GENERATION: u64 = 9_007_199_254_740_991;

fn normalize_timestamp(value: &str, field: &str) -> DeployServiceResult<String> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map(|timestamp| timestamp.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|error| {
            DeployServiceError::Internal(format!("invalid {field} RFC 3339 timestamp: {error}"))
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeAssignmentDatabase {
    PostgreSql,
    Sqlite,
}

impl RuntimeAssignmentDatabase {
    fn resolve(backend_name: &str) -> DeployServiceResult<Self> {
        match backend_name {
            "PostgreSQL" => Ok(Self::PostgreSql),
            "SQLite" => Ok(Self::Sqlite),
            _ => Err(DeployServiceError::Internal(format!(
                "unsupported runtime assignment database backend {backend_name}"
            ))),
        }
    }
}

struct SqlxRuntimeAssignmentMutation {
    database: RuntimeAssignmentDatabase,
    transaction: Transaction<'static, Any>,
    id_generator: SnowflakeIdGenerator,
    target_id: i64,
    tenant_id: i64,
    latest: Option<RuntimeAssignmentState>,
    next_generation: u64,
}

impl DeployRepository {
    pub(super) async fn latest_runtime_assignment_repo(
        &self,
        target_uuid: &str,
    ) -> DeployServiceResult<Option<RuntimeAssignmentState>> {
        let row = sqlx::query(
            "SELECT a.uuid AS assignment_uuid, t.uuid AS target_uuid, a.tenant_id,
                    a.generation, a.snapshot_uuid, a.snapshot_sha256,
                    a.desired_state_sha256,
                    CAST(a.runtime_set_json AS TEXT) AS runtime_set_json, a.publish_status,
                    a.attempt_count, a.lease_owner
             FROM deploy_runtime_assignment a
             INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
             WHERE t.uuid = $1 AND t.deleted_at IS NULL
             ORDER BY a.generation DESC
             LIMIT 1",
        )
        .bind(target_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("load latest deploy runtime assignment", error))?;
        row.as_ref().map(map_assignment_row).transpose()
    }

    pub(super) async fn begin_runtime_assignment_mutation_repo(
        &self,
        target_uuid: &str,
        tenant_id: i64,
    ) -> DeployServiceResult<Box<dyn DeployRuntimeAssignmentMutationPort>> {
        let (database, mut transaction) = begin_runtime_assignment_transaction(&self.pool).await?;
        let target_query = if database == RuntimeAssignmentDatabase::PostgreSql {
            "SELECT id, tenant_id FROM deploy_web_node_target
             WHERE uuid = $1 AND tenant_id = $2 AND status = 'ACTIVE' AND deleted_at IS NULL
             FOR UPDATE"
        } else {
            "SELECT id, tenant_id FROM deploy_web_node_target
             WHERE uuid = $1 AND tenant_id = $2 AND status = 'ACTIVE' AND deleted_at IS NULL"
        };
        let target = sqlx::query(target_query)
            .bind(target_uuid)
            .bind(tenant_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("resolve deploy Web Node target", error))?
            .ok_or_else(|| DeployServiceError::not_found("Web Node target not found"))?;
        let target_id: i64 = target
            .try_get("id")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
        let latest = latest_runtime_assignment_for_target(&mut transaction, target_id).await?;
        let next_generation = latest
            .as_ref()
            .map_or(1, |assignment| assignment.generation.saturating_add(1));
        Ok(Box::new(SqlxRuntimeAssignmentMutation {
            database,
            transaction,
            id_generator: self.id_generator.clone(),
            target_id,
            tenant_id,
            latest,
            next_generation,
        }))
    }

    pub(super) async fn claim_due_runtime_assignments_repo(
        &self,
        maximum_items: i64,
        now: &str,
        lease_owner: &str,
        lease_expires_at: &str,
        maximum_attempts: i32,
    ) -> DeployServiceResult<Vec<RuntimeAssignmentState>> {
        let now = normalize_timestamp(now, "runtime assignment claim time")?;
        let lease_expires_at =
            normalize_timestamp(lease_expires_at, "runtime assignment lease expiry")?;
        let (database, mut transaction) = begin_runtime_assignment_transaction(&self.pool).await?;
        let assignments = if database == RuntimeAssignmentDatabase::PostgreSql {
            let rows = sqlx::query(
                "WITH candidates AS (
                    SELECT a.id
                    FROM deploy_runtime_assignment a
                    INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
                    WHERE (
                        (a.publish_status IN ('PENDING', 'FAILED')
                         AND a.attempt_count < $2
                         AND (a.next_attempt_at IS NULL
                              OR a.next_attempt_at <= CAST($1 AS TIMESTAMPTZ)))
                        OR (a.publish_status = 'PUBLISHING'
                            AND a.attempt_count < $2
                            AND a.lease_expires_at <= CAST($1 AS TIMESTAMPTZ))
                    )
                      AND t.status = 'ACTIVE' AND t.deleted_at IS NULL
                    ORDER BY COALESCE(a.next_attempt_at, a.lease_expires_at, a.created_at),
                             a.created_at, a.id
                    FOR UPDATE OF a SKIP LOCKED
                    LIMIT $3
                 ), claimed AS (
                    UPDATE deploy_runtime_assignment a
                    SET publish_status = 'PUBLISHING', attempt_count = attempt_count + 1,
                        next_attempt_at = NULL, lease_owner = $4,
                        lease_expires_at = CAST($5 AS TIMESTAMPTZ),
                        updated_at = CAST($1 AS TIMESTAMPTZ), version = version + 1
                    FROM candidates c
                    WHERE a.id = c.id
                    RETURNING a.*
                 )
                 SELECT a.uuid AS assignment_uuid, t.uuid AS target_uuid, a.tenant_id,
                        a.generation, a.snapshot_uuid, a.snapshot_sha256,
                        a.desired_state_sha256,
                        CAST(a.runtime_set_json AS TEXT) AS runtime_set_json, a.publish_status,
                        a.attempt_count, a.lease_owner
                 FROM claimed a
                 INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
                 ORDER BY a.created_at, a.id",
            )
            .bind(&now)
            .bind(maximum_attempts)
            .bind(maximum_items)
            .bind(lease_owner)
            .bind(&lease_expires_at)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|error| store_error("claim PostgreSQL runtime assignments", error))?;
            rows.iter()
                .map(map_assignment_row)
                .collect::<DeployServiceResult<Vec<_>>>()?
        } else {
            let candidate_rows = sqlx::query(
                "SELECT a.id
                 FROM deploy_runtime_assignment a
                 INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
                 WHERE (
                    (a.publish_status IN ('PENDING', 'FAILED')
                     AND a.attempt_count < $2
                     AND (a.next_attempt_at IS NULL OR a.next_attempt_at <= $1))
                    OR (a.publish_status = 'PUBLISHING'
                        AND a.attempt_count < $2
                        AND a.lease_expires_at <= $1)
                 )
                   AND t.status = 'ACTIVE' AND t.deleted_at IS NULL
                 ORDER BY COALESCE(a.next_attempt_at, a.lease_expires_at, a.created_at),
                          a.created_at, a.id
                 LIMIT $3",
            )
            .bind(&now)
            .bind(maximum_attempts)
            .bind(maximum_items)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|error| store_error("select SQLite runtime assignment claims", error))?;
            let mut assignments = Vec::with_capacity(candidate_rows.len());
            for candidate in candidate_rows {
                let assignment_id: i64 = candidate
                    .try_get("id")
                    .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
                sqlx::query(
                    "UPDATE deploy_runtime_assignment
                     SET publish_status = 'PUBLISHING', attempt_count = attempt_count + 1,
                         next_attempt_at = NULL, lease_owner = $2, lease_expires_at = $3,
                         updated_at = $4, version = version + 1
                     WHERE id = $1",
                )
                .bind(assignment_id)
                .bind(lease_owner)
                .bind(&lease_expires_at)
                .bind(&now)
                .execute(&mut *transaction)
                .await
                .map_err(|error| store_error("claim SQLite runtime assignment", error))?;
                assignments.push(
                    runtime_assignment_by_id(&mut transaction, assignment_id)
                        .await?
                        .ok_or_else(|| {
                            DeployServiceError::Internal(
                                "claimed runtime assignment could not be reloaded".to_owned(),
                            )
                        })?,
                );
            }
            assignments
        };
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit deploy runtime assignment claims", error))?;
        Ok(assignments)
    }

    pub(super) async fn mark_runtime_assignment_published_repo(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        receipt: &RuntimeAssignmentReceipt,
        published_at: &str,
    ) -> DeployServiceResult<()> {
        let published_at =
            normalize_timestamp(published_at, "runtime assignment publication time")?;
        let mut connection =
            self.pool.acquire().await.map_err(|error| {
                store_error("acquire deploy runtime publication connection", error)
            })?;
        let database = RuntimeAssignmentDatabase::resolve(connection.backend_name())?;
        let update_query = match database {
            RuntimeAssignmentDatabase::PostgreSql => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'PUBLISHED', remote_assignment_uuid = $3,
                     published_at = CAST($4 AS TIMESTAMPTZ), next_attempt_at = NULL,
                     lease_owner = NULL, lease_expires_at = NULL, last_error_code = NULL,
                     updated_at = CAST($4 AS TIMESTAMPTZ), version = version + 1
                 WHERE uuid = $1 AND lease_owner = $2 AND snapshot_uuid = $5
                   AND snapshot_sha256 = $6 AND generation = $7
                   AND publish_status = 'PUBLISHING'"
            }
            RuntimeAssignmentDatabase::Sqlite => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'PUBLISHED', remote_assignment_uuid = $3,
                     published_at = $4, next_attempt_at = NULL, lease_owner = NULL,
                     lease_expires_at = NULL, last_error_code = NULL,
                     updated_at = $4, version = version + 1
                 WHERE uuid = $1 AND lease_owner = $2 AND snapshot_uuid = $5
                   AND snapshot_sha256 = $6 AND generation = $7
                   AND publish_status = 'PUBLISHING'"
            }
        };
        let result = sqlx::query(update_query)
            .bind(assignment_uuid)
            .bind(lease_owner)
            .bind(&receipt.assignment_uuid)
            .bind(&published_at)
            .bind(&receipt.snapshot_uuid)
            .bind(&receipt.snapshot_sha256)
            .bind(receipt.generation.parse::<i64>().unwrap_or_default())
            .execute(&mut *connection)
            .await
            .map_err(|error| store_error("mark deploy runtime assignment published", error))?;
        if result.rows_affected() == 0 {
            return Err(DeployServiceError::conflict(
                "runtime assignment publication state changed concurrently",
            ));
        }
        Ok(())
    }

    pub(super) async fn mark_runtime_assignment_failed_repo(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        error_code: &str,
        next_attempt_at: Option<&str>,
        updated_at: &str,
    ) -> DeployServiceResult<()> {
        let next_attempt_at = next_attempt_at
            .map(|value| normalize_timestamp(value, "runtime assignment next attempt time"))
            .transpose()?;
        let updated_at = normalize_timestamp(updated_at, "runtime assignment failure time")?;
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|error| store_error("acquire deploy runtime failure connection", error))?;
        let database = RuntimeAssignmentDatabase::resolve(connection.backend_name())?;
        let update_query = match database {
            RuntimeAssignmentDatabase::PostgreSql => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'FAILED',
                     next_attempt_at = CAST($3 AS TIMESTAMPTZ), lease_owner = NULL,
                     lease_expires_at = NULL, last_error_code = $4,
                     updated_at = CAST($5 AS TIMESTAMPTZ), version = version + 1
                 WHERE uuid = $1 AND lease_owner = $2 AND publish_status = 'PUBLISHING'"
            }
            RuntimeAssignmentDatabase::Sqlite => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'FAILED', next_attempt_at = $3,
                     lease_owner = NULL, lease_expires_at = NULL, last_error_code = $4,
                     updated_at = $5, version = version + 1
                 WHERE uuid = $1 AND lease_owner = $2 AND publish_status = 'PUBLISHING'"
            }
        };
        let result = sqlx::query(update_query)
            .bind(assignment_uuid)
            .bind(lease_owner)
            .bind(next_attempt_at.as_deref())
            .bind(error_code)
            .bind(&updated_at)
            .execute(&mut *connection)
            .await
            .map_err(|error| store_error("mark deploy runtime assignment failed", error))?;
        if result.rows_affected() == 0 {
            return Err(DeployServiceError::conflict(
                "runtime assignment publication state changed concurrently",
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl DeployRuntimeAssignmentMutationPort for SqlxRuntimeAssignmentMutation {
    fn latest_runtime_assignment(&self) -> Option<&RuntimeAssignmentState> {
        self.latest.as_ref()
    }

    fn next_generation(&self) -> u64 {
        self.next_generation
    }

    async fn commit_without_change(self: Box<Self>) -> DeployServiceResult<()> {
        self.transaction
            .commit()
            .await
            .map_err(|error| store_error("commit deploy runtime assignment read", error))
    }

    async fn persist_runtime_assignment(
        self: Box<Self>,
        command: PersistRuntimeAssignmentCommand,
    ) -> DeployServiceResult<RuntimeAssignmentState> {
        let SqlxRuntimeAssignmentMutation {
            database,
            mut transaction,
            id_generator,
            target_id,
            tenant_id,
            latest: _,
            next_generation,
        } = *self;
        if next_generation > MAXIMUM_RUNTIME_GENERATION {
            return Err(DeployServiceError::conflict(
                "runtime assignment generation is exhausted",
            ));
        }
        if command
            .runtime_set
            .get("generation")
            .and_then(serde_json::Value::as_u64)
            != Some(next_generation)
        {
            return Err(DeployServiceError::conflict(
                "runtime assignment generation does not match its transaction reservation",
            ));
        }
        let id = next_id(&id_generator)?;
        let runtime_set_json = serde_json::to_string(&command.runtime_set)
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
        let created_at =
            normalize_timestamp(&command.created_at, "runtime assignment creation time")?;
        let insert_query = if database == RuntimeAssignmentDatabase::PostgreSql {
            "INSERT INTO deploy_runtime_assignment (
                id, uuid, tenant_id, node_target_id, generation, snapshot_uuid,
                snapshot_sha256, desired_state_sha256, runtime_set_json, runtime_set_bytes,
                publish_status, attempt_count, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS JSONB), $10,
                 'PENDING', 0, CAST($11 AS TIMESTAMPTZ), CAST($11 AS TIMESTAMPTZ), 1
             )"
        } else {
            "INSERT INTO deploy_runtime_assignment (
                id, uuid, tenant_id, node_target_id, generation, snapshot_uuid,
                snapshot_sha256, desired_state_sha256, runtime_set_json, runtime_set_bytes,
                publish_status, attempt_count, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                'PENDING', 0, $11, $11, 1
             )"
        };
        sqlx::query(insert_query)
            .bind(id)
            .bind(&command.assignment_uuid)
            .bind(tenant_id)
            .bind(target_id)
            .bind(next_generation as i64)
            .bind(&command.snapshot_uuid)
            .bind(&command.snapshot_sha256)
            .bind(&command.desired_state_sha256)
            .bind(&runtime_set_json)
            .bind(command.runtime_set_bytes as i64)
            .bind(&created_at)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert deploy runtime assignment", error))?;
        let supersede_query = match database {
            RuntimeAssignmentDatabase::PostgreSql => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'SUPERSEDED', lease_owner = NULL,
                     lease_expires_at = NULL, updated_at = CAST($1 AS TIMESTAMPTZ),
                     version = version + 1
                 WHERE node_target_id = $2 AND generation < $3
                   AND publish_status <> 'SUPERSEDED'"
            }
            RuntimeAssignmentDatabase::Sqlite => {
                "UPDATE deploy_runtime_assignment
                 SET publish_status = 'SUPERSEDED', lease_owner = NULL,
                     lease_expires_at = NULL, updated_at = $1, version = version + 1
                 WHERE node_target_id = $2 AND generation < $3
                   AND publish_status <> 'SUPERSEDED'"
            }
        };
        sqlx::query(supersede_query)
            .bind(&created_at)
            .bind(target_id)
            .bind(next_generation as i64)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("supersede older deploy runtime assignments", error))?;
        let assignment = runtime_assignment_by_uuid(&mut transaction, &command.assignment_uuid)
            .await?
            .ok_or_else(|| {
                DeployServiceError::Internal(
                    "persisted runtime assignment could not be reloaded".to_owned(),
                )
            })?;
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit deploy runtime assignment", error))?;
        Ok(assignment)
    }
}

async fn begin_runtime_assignment_transaction(
    pool: &AnyPool,
) -> DeployServiceResult<(RuntimeAssignmentDatabase, Transaction<'static, Any>)> {
    let connection = pool
        .acquire()
        .await
        .map_err(|error| store_error("acquire deploy runtime assignment connection", error))?;
    let database = RuntimeAssignmentDatabase::resolve(connection.backend_name())?;
    let begin_statement = match database {
        RuntimeAssignmentDatabase::PostgreSql => None,
        RuntimeAssignmentDatabase::Sqlite => Some(Cow::Borrowed("BEGIN IMMEDIATE")),
    };
    let transaction = Transaction::begin(connection, begin_statement)
        .await
        .map_err(|error| store_error("begin deploy runtime assignment transaction", error))?;
    Ok((database, transaction))
}

async fn latest_runtime_assignment_for_target(
    transaction: &mut Transaction<'static, Any>,
    target_id: i64,
) -> DeployServiceResult<Option<RuntimeAssignmentState>> {
    let row = sqlx::query(
        "SELECT a.uuid AS assignment_uuid, t.uuid AS target_uuid, a.tenant_id,
                a.generation, a.snapshot_uuid, a.snapshot_sha256,
                a.desired_state_sha256,
                CAST(a.runtime_set_json AS TEXT) AS runtime_set_json, a.publish_status,
                a.attempt_count, a.lease_owner
         FROM deploy_runtime_assignment a
         INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
         WHERE a.node_target_id = $1
         ORDER BY a.generation DESC
         LIMIT 1",
    )
    .bind(target_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| store_error("load locked deploy runtime assignment", error))?;
    row.as_ref().map(map_assignment_row).transpose()
}

async fn runtime_assignment_by_uuid(
    transaction: &mut Transaction<'static, Any>,
    assignment_uuid: &str,
) -> DeployServiceResult<Option<RuntimeAssignmentState>> {
    let row = sqlx::query(
        "SELECT a.uuid AS assignment_uuid, t.uuid AS target_uuid, a.tenant_id,
                a.generation, a.snapshot_uuid, a.snapshot_sha256,
                a.desired_state_sha256,
                CAST(a.runtime_set_json AS TEXT) AS runtime_set_json, a.publish_status,
                a.attempt_count, a.lease_owner
         FROM deploy_runtime_assignment a
         INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
         WHERE a.uuid = $1",
    )
    .bind(assignment_uuid)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| store_error("reload deploy runtime assignment", error))?;
    row.as_ref().map(map_assignment_row).transpose()
}

async fn runtime_assignment_by_id(
    transaction: &mut Transaction<'static, Any>,
    assignment_id: i64,
) -> DeployServiceResult<Option<RuntimeAssignmentState>> {
    let row = sqlx::query(
        "SELECT a.uuid AS assignment_uuid, t.uuid AS target_uuid, a.tenant_id,
                a.generation, a.snapshot_uuid, a.snapshot_sha256,
                a.desired_state_sha256,
                CAST(a.runtime_set_json AS TEXT) AS runtime_set_json, a.publish_status,
                a.attempt_count, a.lease_owner
         FROM deploy_runtime_assignment a
         INNER JOIN deploy_web_node_target t ON t.id = a.node_target_id
         WHERE a.id = $1",
    )
    .bind(assignment_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| store_error("reload claimed deploy runtime assignment", error))?;
    row.as_ref().map(map_assignment_row).transpose()
}

fn map_assignment_row(row: &AnyRow) -> DeployServiceResult<RuntimeAssignmentState> {
    let generation: i64 = row
        .try_get("generation")
        .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
    let runtime_set_json: String = row
        .try_get("runtime_set_json")
        .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
    let publish_status: String = row
        .try_get("publish_status")
        .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
    Ok(RuntimeAssignmentState {
        assignment_uuid: row
            .try_get("assignment_uuid")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        target_uuid: row
            .try_get("target_uuid")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        tenant_id: row
            .try_get("tenant_id")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        generation: generation.try_into().map_err(|_| {
            DeployServiceError::Internal("stored runtime generation is negative".to_owned())
        })?,
        snapshot_uuid: row
            .try_get("snapshot_uuid")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        snapshot_sha256: row
            .try_get("snapshot_sha256")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        desired_state_sha256: row
            .try_get("desired_state_sha256")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        runtime_set: serde_json::from_str(&runtime_set_json)
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        publish_status: parse_publish_status(&publish_status)?,
        attempt_count: row
            .try_get("attempt_count")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
        lease_owner: row
            .try_get("lease_owner")
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?,
    })
}

fn parse_publish_status(value: &str) -> DeployServiceResult<RuntimeAssignmentPublishStatus> {
    match value {
        "PENDING" => Ok(RuntimeAssignmentPublishStatus::Pending),
        "PUBLISHING" => Ok(RuntimeAssignmentPublishStatus::Publishing),
        "PUBLISHED" => Ok(RuntimeAssignmentPublishStatus::Published),
        "FAILED" => Ok(RuntimeAssignmentPublishStatus::Failed),
        "SUPERSEDED" => Ok(RuntimeAssignmentPublishStatus::Superseded),
        _ => Err(DeployServiceError::Internal(format!(
            "unknown runtime assignment publish status {value}"
        ))),
    }
}
