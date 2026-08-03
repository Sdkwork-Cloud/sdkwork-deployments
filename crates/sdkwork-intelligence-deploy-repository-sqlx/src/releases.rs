use sdkwork_deploy_contract::{
    CreateReleaseRequest, DeployServiceError, DeployServiceResult, ReleasePage, ReleaseResponse,
    RELEASE_STATUS_ACTIVE,
};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::support::{
    datetime_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id,
    resolve_site_uuid, store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_releases_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ReleasePage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let (page, page_size, offset) = pagination(page, page_size);

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_release
             WHERE tenant_id = $1 AND site_id = $2 AND status = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(RELEASE_STATUS_ACTIVE)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_release", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT r.uuid, r.site_id, a.uuid AS artifact_uuid, r.version_tag, r.status, r.created_at
             FROM deploy_release r
             JOIN deploy_artifact a ON a.id = r.artifact_id
             WHERE r.tenant_id = $1 AND r.site_id = $2 AND r.status = $3
             ORDER BY r.created_at DESC, r.id DESC LIMIT $4 OFFSET $5",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(RELEASE_STATUS_ACTIVE)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_release", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(
                map_release_row(&self.pool, tenant_id, row)
                    .await
                    .map_err(|error| {
                        DeployServiceError::Internal(format!("map deploy_release row: {error}"))
                    })?,
            );
        }

        Ok(ReleasePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn retrieve_release_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<ReleaseResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT r.uuid, r.site_id, a.uuid AS artifact_uuid, r.version_tag, r.status, r.created_at
             FROM deploy_release r
             JOIN deploy_artifact a ON a.id = r.artifact_id
             WHERE r.tenant_id = $1 AND r.site_id = $2 AND r.uuid = $3 AND r.status = $4",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(release_id)
        .bind(RELEASE_STATUS_ACTIVE)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_release", error))?;

        let Some(row) = row else {
            return Err(DeployServiceError::not_found("release not found"));
        };
        map_release_row(&self.pool, tenant_id, &row)
            .await
            .map_err(|error| DeployServiceError::Internal(error.to_string()))
    }

    pub(super) async fn find_release_by_idempotency_key_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<ReleaseResponse>> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT r.uuid, r.site_id, a.uuid AS artifact_uuid, r.version_tag, r.status, r.created_at
             FROM deploy_release r
             JOIN deploy_artifact a ON a.id = r.artifact_id
             WHERE r.tenant_id = $1 AND r.site_id = $2 AND r.idempotency_key = $3",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find deploy_release by idempotency", error))?;

        match row {
            Some(row) => map_release_row(&self.pool, tenant_id, &row)
                .await
                .map(Some)
                .map_err(|error| DeployServiceError::Internal(error.to_string())),
            None => Ok(None),
        }
    }

    pub(super) async fn create_release_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateReleaseRequest,
    ) -> DeployServiceResult<ReleaseResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let (artifact_internal_id, artifact) = self
            .load_artifact_for_release_repo(tenant_id, &request.artifact_id)
            .await?;

        if let Some(artifact_site) = artifact.site_id.as_deref() {
            if artifact_site != site_id {
                return Err(DeployServiceError::validation(
                    "artifact siteId must match release siteId when bound",
                ));
            }
        }

        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_release (
                id, uuid, tenant_id, site_id, artifact_id, version_tag, status,
                idempotency_key, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, '{}', CAST($9 AS TIMESTAMPTZ), CAST($9 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(artifact_internal_id)
        .bind(request.version_tag.as_deref())
        .bind(RELEASE_STATUS_ACTIVE)
        .bind(&request.idempotency_key)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_release", error))?;

        self.retrieve_release_repo(tenant_id, site_id, &uuid).await
    }
}

async fn map_release_row(
    pool: &PgPool,
    tenant_id: i64,
    row: &PgRow,
) -> Result<ReleaseResponse, sqlx::Error> {
    let site_internal_id: i64 = row.try_get("site_id")?;
    let site_uuid = resolve_site_uuid(pool, tenant_id, site_internal_id)
        .await
        .map_err(|error| sqlx::Error::Decode(error.to_string().into()))?;

    Ok(ReleaseResponse {
        id: row.try_get("uuid")?,
        site_id: site_uuid,
        artifact_id: row.try_get("artifact_uuid")?,
        version_tag: row.try_get("version_tag").ok(),
        status: row.try_get("status")?,
        created_at: datetime_from_row(row, "created_at")?,
    })
}
