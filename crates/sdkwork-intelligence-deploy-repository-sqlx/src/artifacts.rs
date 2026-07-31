use sdkwork_deploy_contract::{
    is_deploy_package_artifact_type, ArtifactPage, ArtifactResponse, CreateArtifactRequest,
    DeployServiceError, DeployServiceResult, ARTIFACT_STATUS_ACTIVE, ARTIFACT_STATUS_RETAINED,
    UPLOAD_SESSION_STATUS_COMPLETED,
};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::support::{
    datetime_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_site_uuid, store_error,
};
use crate::DeployRepository;

const ARTIFACT_SELECT: &str = "a.uuid, a.site_id, a.package_type, a.file_name, a.content_type,
    a.content_length, a.checksum_sha256, a.drive_node_id, a.status, a.created_at,
    u.uuid AS upload_session_uuid";

impl DeployRepository {
    pub(super) async fn list_artifacts_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ArtifactPage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_artifact
             WHERE tenant_id = $1 AND status = $2",
        )
        .bind(tenant_id)
        .bind(ARTIFACT_STATUS_ACTIVE)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_artifact", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let query = format!(
            "SELECT {ARTIFACT_SELECT}
             FROM deploy_artifact a
             JOIN deploy_upload_session_ref u ON u.id = a.upload_session_ref_id
             WHERE a.tenant_id = $1 AND a.status = $2
             ORDER BY a.created_at DESC LIMIT $3 OFFSET $4"
        );
        let rows = sqlx::query(&query)
            .bind(tenant_id)
            .bind(ARTIFACT_STATUS_ACTIVE)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list deploy_artifact", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(
                map_artifact_row(&self.pool, tenant_id, row)
                    .await
                    .map_err(|error| {
                        DeployServiceError::Internal(format!("map deploy_artifact row: {error}"))
                    })?,
            );
        }

        Ok(ArtifactPage { items, total })
    }

    pub(super) async fn retrieve_artifact_repo(
        &self,
        tenant_id: i64,
        artifact_id: &str,
    ) -> DeployServiceResult<ArtifactResponse> {
        let query = format!(
            "SELECT {ARTIFACT_SELECT}
             FROM deploy_artifact a
             JOIN deploy_upload_session_ref u ON u.id = a.upload_session_ref_id
             WHERE a.tenant_id = $1 AND a.uuid = $2 AND a.status = $3"
        );
        let row = sqlx::query(&query)
            .bind(tenant_id)
            .bind(artifact_id)
            .bind(ARTIFACT_STATUS_ACTIVE)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve deploy_artifact", error))?;

        let Some(row) = row else {
            return Err(DeployServiceError::not_found("artifact not found"));
        };
        map_artifact_row(&self.pool, tenant_id, &row)
            .await
            .map_err(|error| DeployServiceError::Internal(error.to_string()))
    }

    pub(super) async fn create_artifact_from_drive_repo(
        &self,
        tenant_id: i64,
        request: &CreateArtifactRequest,
    ) -> DeployServiceResult<ArtifactResponse> {
        if let Some(existing) = self
            .find_upload_session_by_idempotency_key_repo(tenant_id, &request.idempotency_key)
            .await?
        {
            return self
                .create_artifact_from_upload_session_repo(
                    tenant_id,
                    &existing.id,
                    request.checksum_sha256.as_deref().unwrap_or(""),
                )
                .await;
        }

        let site_internal_id = match request.site_id.as_deref() {
            Some(site_id) => Some(
                crate::support::resolve_site_internal_id(&self.pool, tenant_id, site_id).await?,
            ),
            None => None,
        };
        let reference_id = next_id(self.id_generator())?;
        let reference_uuid = new_uuid();
        let now = now_rfc3339();
        sqlx::query(
            "INSERT INTO deploy_upload_session_ref
             (id, uuid, tenant_id, site_id, drive_upload_session_id, drive_upload_item_id,
              drive_space_id, drive_node_id, package_type, file_name, content_type,
              content_length, checksum, status, idempotency_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $16)",
        )
        .bind(reference_id)
        .bind(&reference_uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(request.drive_upload_session_id.trim())
        .bind(request.drive_upload_item_id.as_deref())
        .bind(request.drive_space_id.trim())
        .bind(request.drive_node_id.trim())
        .bind(request.package_type)
        .bind(request.file_name.trim())
        .bind(request.content_type.trim())
        .bind(request.content_length)
        .bind(request.checksum_sha256.as_deref())
        .bind(UPLOAD_SESSION_STATUS_COMPLETED)
        .bind(request.idempotency_key.trim())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("register Drive artifact reference", error))?;

        self.create_artifact_from_upload_session_repo(
            tenant_id,
            &reference_uuid,
            request.checksum_sha256.as_deref().unwrap_or(""),
        )
        .await
    }

    pub(super) async fn retain_artifact_repo(
        &self,
        tenant_id: i64,
        artifact_id: &str,
    ) -> DeployServiceResult<()> {
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE deploy_artifact
             SET status = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND status = $5",
        )
        .bind(tenant_id)
        .bind(artifact_id)
        .bind(ARTIFACT_STATUS_RETAINED)
        .bind(&now)
        .bind(ARTIFACT_STATUS_ACTIVE)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("retain deploy_artifact", error))?;

        if result.rows_affected() == 0 {
            return Err(DeployServiceError::not_found("artifact not found"));
        }
        Ok(())
    }

    pub(super) async fn create_artifact_from_upload_session_repo(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
        checksum_sha256: &str,
    ) -> DeployServiceResult<ArtifactResponse> {
        let session_row = sqlx::query(
            "SELECT id, uuid, site_id, package_type, file_name, content_type, content_length,
                    status, drive_node_id, drive_space_id
             FROM deploy_upload_session_ref
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(upload_session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("load upload session for artifact", error))?
        .ok_or_else(|| DeployServiceError::not_found("upload session not found"))?;

        let session_internal_id: i64 = session_row
            .try_get("id")
            .map_err(|error| DeployServiceError::Internal(format!("upload session id: {error}")))?;
        let package_type: i32 = session_row.try_get("package_type").unwrap_or(1);
        let status: i32 = session_row.try_get("status").unwrap_or(0);

        if !is_deploy_package_artifact_type(package_type) {
            return Err(DeployServiceError::validation(
                "upload session package type does not produce a deploy artifact",
            ));
        }
        if status != UPLOAD_SESSION_STATUS_COMPLETED {
            return Err(DeployServiceError::validation(
                "upload session must be completed before creating an artifact",
            ));
        }

        let existing =
            sqlx::query("SELECT uuid FROM deploy_artifact WHERE upload_session_ref_id = $1")
                .bind(session_internal_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("find deploy_artifact by upload session", error))?;

        if let Some(row) = existing {
            let artifact_uuid: String = row
                .try_get("uuid")
                .map_err(|error| DeployServiceError::Internal(format!("artifact uuid: {error}")))?;
            return self.retrieve_artifact_repo(tenant_id, &artifact_uuid).await;
        }

        let drive_node_id: Option<String> = session_row.try_get("drive_node_id").ok();
        let drive_node_id = drive_node_id.ok_or_else(|| {
            DeployServiceError::validation("completed upload session has no drive node")
        })?;
        let drive_space_id: Option<String> = session_row.try_get("drive_space_id").ok();
        let site_internal_id: Option<i64> = session_row.try_get("site_id").ok();
        let file_name: String = session_row.try_get("file_name").unwrap_or_default();
        let content_type: String = session_row.try_get("content_type").unwrap_or_default();
        let content_length: i64 = session_row.try_get("content_length").unwrap_or(0);
        let drive_path = format!("drive://node/{drive_node_id}");

        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_artifact (
                id, uuid, tenant_id, site_id, upload_session_ref_id, package_type,
                file_name, content_type, content_length, checksum_sha256, drive_node_id,
                drive_space_id, drive_path, status, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, '{}', $15, $15, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(session_internal_id)
        .bind(package_type)
        .bind(&file_name)
        .bind(&content_type)
        .bind(content_length)
        .bind(checksum_sha256)
        .bind(&drive_node_id)
        .bind(drive_space_id.as_deref())
        .bind(&drive_path)
        .bind(ARTIFACT_STATUS_ACTIVE)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_artifact", error))?;

        self.retrieve_artifact_repo(tenant_id, &uuid).await
    }

    pub(super) async fn load_artifact_for_release_repo(
        &self,
        tenant_id: i64,
        artifact_id: &str,
    ) -> DeployServiceResult<(i64, ArtifactResponse)> {
        let internal_id =
            crate::support::resolve_artifact_internal_id(&self.pool, tenant_id, artifact_id)
                .await?;
        let artifact = self.retrieve_artifact_repo(tenant_id, artifact_id).await?;
        Ok((internal_id, artifact))
    }
}

async fn map_artifact_row(
    pool: &PgPool,
    tenant_id: i64,
    row: &PgRow,
) -> Result<ArtifactResponse, sqlx::Error> {
    let site_id = match row.try_get::<Option<i64>, _>("site_id").ok().flatten() {
        Some(site_internal_id) => Some(
            resolve_site_uuid(pool, tenant_id, site_internal_id)
                .await
                .map_err(|error| sqlx::Error::Decode(error.to_string().into()))?,
        ),
        None => None,
    };

    Ok(ArtifactResponse {
        id: row.try_get("uuid")?,
        site_id,
        package_type: row.try_get("package_type")?,
        file_name: row.try_get("file_name")?,
        content_type: row.try_get("content_type")?,
        content_length: row.try_get("content_length")?,
        checksum_sha256: row.try_get("checksum_sha256").ok(),
        drive_node_id: row.try_get("drive_node_id")?,
        upload_session_id: row.try_get("upload_session_uuid")?,
        status: row.try_get("status")?,
        created_at: datetime_from_row(row, "created_at")?,
    })
}
