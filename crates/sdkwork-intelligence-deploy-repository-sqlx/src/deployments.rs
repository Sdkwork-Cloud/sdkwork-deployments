use sdkwork_deploy_contract::{
    CreateDeploymentRequest, DeployServiceError, DeployServiceResult, DeploymentPage,
    DeploymentResponse,
};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::support::{
    new_uuid, next_id, now_rfc3339, pagination, resolve_release_internal_id,
    resolve_site_internal_id, resolve_site_uuid, store_error,
};
use crate::DeployRepository;

const DEPLOYMENT_SELECT: &str = "d.uuid, d.site_id, d.status, d.deploy_type, d.created_at,
    r.uuid AS release_uuid";

impl DeployRepository {
    pub(super) async fn list_deployments_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<DeploymentPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
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
                 ORDER BY d.created_at DESC LIMIT $4 OFFSET $5"
            );
            let rows = sqlx::query(&query)
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
                 ORDER BY d.created_at DESC LIMIT $3 OFFSET $4"
            );
            let rows = sqlx::query(&query)
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
        })
    }

    pub(super) async fn create_deployment_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse> {
        if let Some(idempotency_key) = request.idempotency_key.as_deref() {
            if !idempotency_key.trim().is_empty() {
                if let Some(existing) = self
                    .find_deployment_by_idempotency_key_repo(tenant_id, site_id, idempotency_key)
                    .await?
                {
                    return Ok(existing);
                }
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

        sqlx::query(
            "INSERT INTO deploy_deployment (
                id, uuid, tenant_id, user_id, site_id, deploy_type, environment, status,
                release_id, artifact_path, artifact_size, artifact_hash, idempotency_key,
                metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, $8, $9, $10, $11, $12, '{}', $13, $13, 0
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
        .bind(request.idempotency_key.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_deployment", error))?;

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
        let query = format!(
            "SELECT {DEPLOYMENT_SELECT}
             FROM deploy_deployment d
             LEFT JOIN deploy_release r ON r.id = d.release_id
             WHERE d.tenant_id = $1 AND d.site_id = $2 AND d.idempotency_key = $3"
        );
        let row = sqlx::query(&query)
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
        let row = sqlx::query(&query)
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

        sqlx::query(
            "UPDATE deploy_deployment
             SET status = 5, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(deployment_id)
        .bind(&now)
        .execute(&self.pool)
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
                $1, $2, $3, $4, $5, $6, $7, 0, $8, $9, $10, $11, $12, '{}', $13, $13, 0
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
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert rollback deploy_deployment", error))?;

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
        created_at: row.try_get("created_at")?,
    })
}
