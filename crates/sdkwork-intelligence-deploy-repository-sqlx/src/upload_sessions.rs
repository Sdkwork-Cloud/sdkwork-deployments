use sqlx::Row;

use sdkwork_deploy_contract::{
    CreateDeployUploadSessionRequest, DeployAppRequestContext, DeployServiceError,
    DeployServiceResult, DeployUploadSessionResponse,
};

use crate::support::{
    datetime_from_row, new_uuid, next_id, now_rfc3339, resolve_site_internal_id, store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(crate) async fn create_upload_session_ref_repo(
        &self,
        tenant_id: i64,
        context: &DeployAppRequestContext,
        request: &CreateDeployUploadSessionRequest,
        drive: &DeployUploadSessionResponse,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let site_internal_id = match request.site_id.as_deref() {
            Some(site_uuid) => {
                Some(resolve_site_internal_id(&self.pool, tenant_id, site_uuid).await?)
            }
            None => None,
        };
        let id = next_id(&self.id_generator)?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        sqlx::query(
            "INSERT INTO deploy_upload_session_ref
             (id, uuid, tenant_id, site_id, drive_upload_session_id, drive_upload_item_id,
              drive_space_id, drive_node_id, package_type, file_name, content_type,
              content_length, checksum, status, idempotency_key, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&drive.drive_upload_session_id)
        .bind(drive.drive_upload_item_id.as_deref())
        .bind(drive.drive_space_id.as_deref())
        .bind(drive.drive_node_id.as_deref())
        .bind(request.package_type)
        .bind(&request.file_name)
        .bind(&request.content_type)
        .bind(request.content_length)
        .bind(request.checksum.as_deref())
        .bind(drive.status)
        .bind(&request.idempotency_key)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("create deploy upload session ref", error))?;

        let _ = context;
        Ok(DeployUploadSessionResponse {
            id: uuid,
            site_id: request.site_id.clone(),
            package_type: request.package_type,
            file_name: request.file_name.clone(),
            content_type: request.content_type.clone(),
            content_length: request.content_length,
            checksum: request.checksum.clone(),
            status: drive.status,
            drive_upload_session_id: drive.drive_upload_session_id.clone(),
            drive_upload_item_id: drive.drive_upload_item_id.clone(),
            drive_space_id: drive.drive_space_id.clone(),
            drive_node_id: drive.drive_node_id.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub(crate) async fn retrieve_upload_session_ref_repo(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let row = sqlx::query(
            "SELECT uuid, site_id, package_type, file_name, content_type, content_length,
                    checksum, status, drive_upload_session_id, drive_upload_item_id,
                    drive_space_id, drive_node_id, created_at, updated_at
             FROM deploy_upload_session_ref
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(upload_session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy upload session ref", error))?
        .ok_or_else(|| DeployServiceError::not_found("upload session not found"))?;

        let site_uuid = match row.try_get::<Option<i64>, _>("site_id").ok().flatten() {
            Some(site_id) => {
                Some(crate::support::resolve_site_uuid(&self.pool, tenant_id, site_id).await?)
            }
            None => None,
        };

        map_upload_session_row(&row, site_uuid).map_err(|error| {
            DeployServiceError::Internal(format!("map deploy upload session ref: {error}"))
        })
    }

    pub(crate) async fn find_upload_session_by_idempotency_key_repo(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<DeployUploadSessionResponse>> {
        let row = sqlx::query(
            "SELECT uuid, site_id, package_type, file_name, content_type, content_length,
                    checksum, status, drive_upload_session_id, drive_upload_item_id,
                    drive_space_id, drive_node_id, created_at, updated_at
             FROM deploy_upload_session_ref
             WHERE tenant_id = $1 AND idempotency_key = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find deploy upload session ref", error))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let site_uuid = match row.try_get::<Option<i64>, _>("site_id").ok().flatten() {
            Some(site_id) => {
                Some(crate::support::resolve_site_uuid(&self.pool, tenant_id, site_id).await?)
            }
            None => None,
        };

        map_upload_session_row(&row, site_uuid)
            .map(Some)
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy upload session ref: {error}"))
            })
    }

    pub(crate) async fn update_upload_session_status_repo(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
        status: i32,
        drive_node_id: Option<&str>,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let now = now_rfc3339();
        sqlx::query(
            "UPDATE deploy_upload_session_ref
             SET status = $1,
                 drive_node_id = COALESCE($2, drive_node_id),
                 updated_at = $3
             WHERE tenant_id = $4 AND uuid = $5 AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(drive_node_id)
        .bind(&now)
        .bind(tenant_id)
        .bind(upload_session_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("update deploy upload session ref", error))?;
        self.retrieve_upload_session_ref_repo(tenant_id, upload_session_id)
            .await
    }
}

fn map_upload_session_row(
    row: &sqlx::postgres::PgRow,
    site_id: Option<String>,
) -> Result<DeployUploadSessionResponse, sqlx::Error> {
    Ok(DeployUploadSessionResponse {
        id: row.try_get("uuid")?,
        site_id,
        package_type: row.try_get("package_type")?,
        file_name: row.try_get("file_name")?,
        content_type: row.try_get("content_type")?,
        content_length: row.try_get("content_length")?,
        checksum: row.try_get("checksum").ok(),
        status: row.try_get("status")?,
        drive_upload_session_id: row.try_get("drive_upload_session_id")?,
        drive_upload_item_id: row.try_get("drive_upload_item_id").ok(),
        drive_space_id: row.try_get("drive_space_id").ok(),
        drive_node_id: row.try_get("drive_node_id").ok(),
        created_at: datetime_from_row(row, "created_at")?,
        updated_at: datetime_from_row(row, "updated_at")?,
    })
}
