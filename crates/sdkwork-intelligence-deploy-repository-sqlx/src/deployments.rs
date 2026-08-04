use sdkwork_deploy_contract::{
    CreateDeploymentRequest, DeployServiceError, DeployServiceResult, DeploymentPage,
    DeploymentResponse,
};
use sqlx::{postgres::PgRow, AssertSqlSafe, Error as SqlxError, PgPool, Row};

use crate::support::{
    datetime_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_release_internal_id,
    resolve_site_internal_id, resolve_site_uuid, store_error,
};
use crate::DeployRepository;

const DEPLOYMENT_SELECT: &str = "d.id, d.uuid, d.site_id, d.status, d.deploy_type, d.created_at,
    r.uuid AS release_uuid";

/// 幂等键以 SHA-256 哈希落库（对齐 Web repository：raw keys are never
/// stored），客户端原始值仅在请求内存中短暂存在。
fn idempotency_key_hash(key: &str) -> String {
    crate::support::sha256_hex(key.trim())
}

impl DeployRepository {
    pub(super) async fn list_deployments_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
        cursor: Option<&str>,
    ) -> DeployServiceResult<DeploymentPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;

        // keyset 模式（PAGINATION_SPEC §6）：growing 表走 (created_at, id)
        // 元组游标，O(page size) 内存、无深 OFFSET、无 COUNT。
        if let Some(cursor) = cursor {
            if !(1..=200).contains(&page_size) {
                return Err(DeployServiceError::validation(
                    "page_size must be between 1 and 200",
                ));
            }
            let (cursor_created_at, cursor_id) = crate::support::decode_keyset_cursor(cursor)
                .ok_or_else(|| DeployServiceError::validation("cursor is invalid"))?;
            let query = format!(
                "SELECT {DEPLOYMENT_SELECT}
                 FROM deploy_deployment d
                 LEFT JOIN deploy_release r ON r.id = d.release_id
                 WHERE d.tenant_id = $1 AND d.site_id = $2
                   AND ($3 IS NULL OR d.status = $3)
                   AND (d.created_at, d.id) < ($4, $5)
                 ORDER BY d.created_at DESC, d.id DESC LIMIT $6"
            );
            let fetch_size = i64::from(page_size) + 1;
            let rows = sqlx::query(AssertSqlSafe(&*query))
                .bind(tenant_id)
                .bind(site_internal_id)
                .bind(status)
                .bind(&cursor_created_at)
                .bind(cursor_id)
                .bind(fetch_size)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| store_error("list deploy_deployment cursor", error))?;
            let has_more = rows.len() > page_size as usize;
            let page_rows = rows
                .into_iter()
                .take(page_size as usize)
                .collect::<Vec<_>>();
            let mut items = Vec::with_capacity(page_rows.len());
            for row in &page_rows {
                items.push(
                    map_deployment_row(&self.pool, tenant_id, row)
                        .await
                        .map_err(|error| {
                            DeployServiceError::Internal(format!(
                                "map deploy_deployment row: {error}"
                            ))
                        })?,
                );
            }
            let next_cursor = has_more
                .then(|| {
                    let last = page_rows.last().expect("non-empty page when has_more");
                    let created_at: String = last.try_get("created_at").map_err(|error| {
                        store_error("map deploy_deployment cursor instant", error)
                    })?;
                    let id: i64 = last
                        .try_get("id")
                        .map_err(|error| store_error("map deploy_deployment cursor id", error))?;
                    Ok::<_, DeployServiceError>(crate::support::encode_keyset_cursor(
                        &created_at,
                        id,
                    ))
                })
                .transpose()?;
            return Ok(DeploymentPage {
                items,
                total: 0,
                page: 0,
                page_size,
                next_cursor,
                has_more: Some(has_more),
            });
        }

        let (page, page_size, offset) = pagination(page, page_size);

        let (count_row, rows) = if let Some(status) = status {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM deploy_deployment
                 WHERE tenant_id = $1 AND site_id = $2 AND status = $3",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count deploy_deployment", error))?;

            let query = format!(
                "SELECT {DEPLOYMENT_SELECT}
                 FROM deploy_deployment d
                 LEFT JOIN deploy_release r ON r.id = d.release_id
                 WHERE d.tenant_id = $1 AND d.site_id = $2 AND d.status = $3
                 ORDER BY d.created_at DESC, d.id DESC LIMIT $4 OFFSET $5"
            );
            let rows = sqlx::query(AssertSqlSafe(&*query))
                .bind(tenant_id)
                .bind(site_internal_id)
                .bind(status)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| store_error("list deploy_deployment", error))?;

            (count_row, rows)
        } else {
            let count_row = sqlx::query(
                "SELECT COUNT(*) AS total FROM deploy_deployment
                 WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count deploy_deployment", error))?;

            let query = format!(
                "SELECT {DEPLOYMENT_SELECT}
                 FROM deploy_deployment d
                 LEFT JOIN deploy_release r ON r.id = d.release_id
                 WHERE d.tenant_id = $1 AND d.site_id = $2
                 ORDER BY d.created_at DESC, d.id DESC LIMIT $3 OFFSET $4"
            );
            let rows = sqlx::query(AssertSqlSafe(&*query))
                .bind(tenant_id)
                .bind(site_internal_id)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| store_error("list deploy_deployment", error))?;

            (count_row, rows)
        };

        let total: i64 = count_row.try_get("total").unwrap_or(0);
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(
                map_deployment_row(&self.pool, tenant_id, row)
                    .await
                    .map_err(|error| {
                        DeployServiceError::Internal(format!("map deploy_deployment row: {error}"))
                    })?,
            );
        }

        Ok(DeploymentPage {
            items,
            total,
            page,
            page_size,
            next_cursor: None,
            has_more: None,
        })
    }

    pub(super) async fn create_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse> {
        let idempotency_key = request
            .idempotency_key
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(idempotency_key_hash);
        if let Some(idempotency_key) = idempotency_key.as_deref() {
            if let Some(existing) = self
                .find_deployment_by_idempotency_key_repo(tenant_id, site_id, idempotency_key)
                .await?
            {
                return Ok(existing);
            }
        }

        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let environment = request
            .environment
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("production");

        let (release_internal_id, artifact_path, artifact_size, artifact_hash): (
            Option<i64>,
            Option<String>,
            Option<i64>,
            Option<String>,
        ) = if let Some(release_uuid) = request.release_id.as_deref() {
            let release_internal =
                resolve_release_internal_id(&self.pool, tenant_id, site_internal_id, release_uuid)
                    .await?;
            let artifact_row = sqlx::query(
                "SELECT a.drive_path, a.content_length, a.checksum_sha256
                     FROM deploy_release r
                     JOIN deploy_artifact a ON a.id = r.artifact_id
                     WHERE r.tenant_id = $1 AND r.site_id = $2 AND r.id = $3",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(release_internal)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("load release artifact for deployment", error))?
            .ok_or_else(|| DeployServiceError::not_found("release not found"))?;

            (
                Some(release_internal),
                artifact_row.try_get("drive_path").ok(),
                artifact_row.try_get("content_length").ok(),
                artifact_row.try_get("checksum_sha256").ok(),
            )
        } else {
            (None, None, None, None)
        };

        if let Err(error) = sqlx::query(
            "INSERT INTO deploy_deployment (
                id, uuid, tenant_id, user_id, site_id, deploy_type, environment, status,
                release_id, artifact_path, artifact_size, artifact_hash, idempotency_key,
                metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, $8, $9, $10, $11, $12, '{}', CAST($13 AS TIMESTAMPTZ), CAST($13 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(site_internal_id)
        .bind(request.deploy_type)
        .bind(environment)
        .bind(release_internal_id)
        .bind(artifact_path.as_deref())
        .bind(artifact_size)
        .bind(artifact_hash.as_deref())
        .bind(idempotency_key.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        {
            // 幂等重放：并发提交下唯一约束先到者胜，重复提交返回已存在的部署
            // 记录而不是 409（对齐 Web repository 的 23505 兜底语义）。
            if let Some(idempotency_key) = idempotency_key.as_deref() {
                if matches!(&error, SqlxError::Database(db) if db.is_unique_violation()) {
                    if let Some(existing) = self
                        .find_deployment_by_idempotency_key_repo(
                            tenant_id,
                            site_id,
                            idempotency_key,
                        )
                        .await?
                    {
                        return Ok(existing);
                    }
                }
            }
            return Err(store_error("insert deploy_deployment", error));
        }

        self.retrieve_deployment_repo(tenant_id, site_id, &uuid)
            .await
    }

    pub(super) async fn find_deployment_by_idempotency_key_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<DeploymentResponse>> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        // 入参为已哈希的存储值（create 路径传入）；直接比较哈希列。
        let query = format!(
            "SELECT {DEPLOYMENT_SELECT}
             FROM deploy_deployment d
             LEFT JOIN deploy_release r ON r.id = d.release_id
             WHERE d.tenant_id = $1 AND d.site_id = $2 AND d.idempotency_key = $3"
        );
        let row = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(idempotency_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("find deploy_deployment by idempotency", error))?;

        match row {
            Some(row) => map_deployment_row(&self.pool, tenant_id, &row)
                .await
                .map(Some)
                .map_err(|error| DeployServiceError::Internal(error.to_string())),
            None => Ok(None),
        }
    }

    pub(super) async fn retrieve_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let query = format!(
            "SELECT {DEPLOYMENT_SELECT}
             FROM deploy_deployment d
             LEFT JOIN deploy_release r ON r.id = d.release_id
             WHERE d.tenant_id = $1 AND d.site_id = $2 AND d.uuid = $3"
        );
        let row = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(deployment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve deploy_deployment", error))?
            .ok_or_else(|| DeployServiceError::not_found("deployment not found"))?;

        map_deployment_row(&self.pool, tenant_id, &row)
            .await
            .map_err(|error| DeployServiceError::Internal(error.to_string()))
    }

    pub(super) async fn rollback_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<DeploymentResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let source = sqlx::query(
            "SELECT id, deploy_type, environment, release_id, artifact_path, artifact_size,
                    artifact_hash
             FROM deploy_deployment
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("rollback deploy_deployment lookup", error))?
        .ok_or_else(|| DeployServiceError::not_found("deployment not found"))?;

        let source_id: i64 = source
            .try_get("id")
            .map_err(|error| store_error("rollback deploy_deployment source id", error))?;
        let deploy_type: i32 = source
            .try_get("deploy_type")
            .map_err(|error| store_error("rollback deploy_deployment deploy_type", error))?;
        let environment: String = source
            .try_get("environment")
            .map_err(|error| store_error("rollback deploy_deployment environment", error))?;
        let release_id: Option<i64> = source.try_get("release_id").ok();
        let artifact_path: Option<String> = source.try_get("artifact_path").ok();
        let artifact_size: Option<i64> = source.try_get("artifact_size").ok();
        let artifact_hash: Option<String> = source.try_get("artifact_hash").ok();
        let now = now_rfc3339();

        // 回滚记录与源记录的状态推进必须在同一事务内完成：INSERT 失败时源
        // 记录不得停留在"已回滚"状态（对齐 Web repository 的单事务语义）。
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                store_error("begin rollback deploy_deployment transaction", error)
            })?;

        sqlx::query(
            "UPDATE deploy_deployment
             SET status = 5, updated_at = CAST($4 AS TIMESTAMPTZ), version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("mark deploy_deployment rolled back", error))?;

        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        sqlx::query(
            "INSERT INTO deploy_deployment (
                id, uuid, tenant_id, user_id, site_id, deploy_type, environment, status,
                release_id, artifact_path, artifact_size, artifact_hash,
                rollback_from, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, $8, $9, $10, $11, $12, '{}', CAST($13 AS TIMESTAMPTZ), CAST($13 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(site_internal_id)
        .bind(deploy_type)
        .bind(&environment)
        .bind(release_id)
        .bind(artifact_path.as_deref())
        .bind(artifact_size)
        .bind(artifact_hash.as_deref())
        .bind(source_id)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert rollback deploy_deployment", error))?;

        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit rollback deploy_deployment transaction", error))?;

        self.retrieve_deployment_repo(tenant_id, site_id, &uuid)
            .await
    }
}

async fn map_deployment_row(
    pool: &PgPool,
    tenant_id: i64,
    row: &PgRow,
) -> Result<DeploymentResponse, sqlx::Error> {
    let site_internal_id: i64 = row.try_get("site_id")?;
    let site_uuid = resolve_site_uuid(pool, tenant_id, site_internal_id)
        .await
        .map_err(|error| sqlx::Error::Decode(error.to_string().into()))?;

    Ok(DeploymentResponse {
        id: row.try_get("uuid")?,
        site_id: site_uuid,
        status: row.try_get("status")?,
        deploy_type: row.try_get("deploy_type")?,
        release_id: row.try_get("release_uuid").ok(),
        created_at: datetime_from_row(row, "created_at")?,
    })
}
