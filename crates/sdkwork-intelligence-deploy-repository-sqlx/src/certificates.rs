use sdkwork_deploy_contract::{
    CertificatePage, CertificateResponse, CreateCertificateRequest, DeployServiceError,
    DeployServiceResult, DeployUploadSessionResponse, UploadCustomCertificateRequest,
    CERTIFICATE_RENEWAL_STATUS_PLANNED, CERTIFICATE_STATUS_ACTIVE, CERTIFICATE_STATUS_PENDING,
    CERTIFICATE_STATUS_REVOKED, CERTIFICATE_TYPE_CUSTOM, CERTIFICATE_TYPE_LETS_ENCRYPT,
};
use serde_json::json;
use sqlx::{any::AnyRow, Row};

use crate::support::{
    new_uuid, next_id, now_rfc3339, pagination, resolve_domain_internal_id,
    resolve_site_internal_id, store_error,
};
use crate::DeployRepository;

const CERTIFICATE_SELECT: &str =
    "uuid, cert_name, cert_type, issuer, not_before, not_after, auto_renew, status, created_at";

impl DeployRepository {
    pub(super) async fn list_certificates_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_certificate WHERE tenant_id = $1 AND status <> $2",
        )
        .bind(tenant_id)
        .bind(CERTIFICATE_STATUS_REVOKED)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_certificate", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let query = format!(
            "SELECT {CERTIFICATE_SELECT}
             FROM deploy_certificate
             WHERE tenant_id = $1 AND status <> $2
             ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        );
        let rows = sqlx::query(&query)
            .bind(tenant_id)
            .bind(CERTIFICATE_STATUS_REVOKED)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list deploy_certificate", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_certificate_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_certificate row: {error}"))
            })?);
        }

        Ok(CertificatePage { items, total })
    }

    pub(super) async fn retrieve_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse> {
        let query = format!(
            "SELECT {CERTIFICATE_SELECT}
             FROM deploy_certificate
             WHERE tenant_id = $1 AND uuid = $2 AND status <> $3"
        );
        let row = sqlx::query(&query)
            .bind(tenant_id)
            .bind(certificate_id)
            .bind(CERTIFICATE_STATUS_REVOKED)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve deploy_certificate", error))?;

        row.as_ref()
            .map(map_certificate_row)
            .transpose()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_certificate row: {error}"))
            })?
            .ok_or_else(|| DeployServiceError::not_found("certificate not found"))
    }

    pub(super) async fn create_certificate_repo(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse> {
        let site_internal_id = match request.site_id.as_deref() {
            Some(site_uuid) => {
                Some(resolve_site_internal_id(&self.pool, tenant_id, site_uuid).await?)
            }
            None => None,
        };
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_certificate (
                id, uuid, tenant_id, site_id, domain_id, cert_name, cert_type, status, metadata,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, NULL, $5, $6, $7, '{}', $8, $8, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&request.cert_name)
        .bind(CERTIFICATE_TYPE_LETS_ENCRYPT)
        .bind(CERTIFICATE_STATUS_PENDING)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_certificate", error))?;

        Ok(CertificateResponse {
            id: uuid,
            cert_name: request.cert_name.clone(),
            cert_type: Some(CERTIFICATE_TYPE_LETS_ENCRYPT),
            issuer: None,
            not_before: None,
            not_after: None,
            auto_renew: Some(true),
            status: CERTIFICATE_STATUS_PENDING,
            created_at: now,
        })
    }

    pub(super) async fn find_certificate_by_idempotency_key_repo(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<CertificateResponse>> {
        let query = format!(
            "SELECT {CERTIFICATE_SELECT}
             FROM deploy_certificate
             WHERE tenant_id = $1 AND idempotency_key = $2 AND status <> $3"
        );
        let row = sqlx::query(&query)
            .bind(tenant_id)
            .bind(idempotency_key)
            .bind(CERTIFICATE_STATUS_REVOKED)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("find deploy_certificate by idempotency", error))?;

        row.as_ref()
            .map(map_certificate_row)
            .transpose()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_certificate row: {error}"))
            })
    }

    pub(super) async fn upload_custom_certificate_repo(
        &self,
        tenant_id: i64,
        request: &UploadCustomCertificateRequest,
        certificate_upload: &DeployUploadSessionResponse,
        private_key_upload: &DeployUploadSessionResponse,
    ) -> DeployServiceResult<CertificateResponse> {
        let site_internal_id = match request.site_id.as_deref() {
            Some(site_uuid) => {
                Some(resolve_site_internal_id(&self.pool, tenant_id, site_uuid).await?)
            }
            None => None,
        };
        let domain_internal_id = match request.domain_id.as_deref() {
            Some(domain_uuid) => {
                Some(resolve_domain_internal_id(&self.pool, tenant_id, domain_uuid).await?)
            }
            None => None,
        };
        let cert_node_id = certificate_upload.drive_node_id.as_deref().ok_or_else(|| {
            DeployServiceError::validation("certificate upload session has no drive node")
        })?;
        let key_node_id = private_key_upload.drive_node_id.as_deref().ok_or_else(|| {
            DeployServiceError::validation("private key upload session has no drive node")
        })?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();
        let metadata = json!({
            "idempotencyKey": request.idempotency_key,
            "certificateUploadSessionId": certificate_upload.id,
            "privateKeyUploadSessionId": private_key_upload.id,
            "certificateDriveNodeId": cert_node_id,
            "privateKeyDriveNodeId": key_node_id,
            "certificateDriveSpaceId": certificate_upload.drive_space_id,
            "privateKeyDriveSpaceId": private_key_upload.drive_space_id,
        });
        let cert_path = format!("drive://node/{cert_node_id}");
        let key_path = format!("drive://node/{key_node_id}");

        sqlx::query(
            "INSERT INTO deploy_certificate (
                id, uuid, tenant_id, site_id, domain_id, cert_name, cert_type, cert_path, key_path,
                status, metadata, idempotency_key, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_internal_id)
        .bind(&request.cert_name)
        .bind(CERTIFICATE_TYPE_CUSTOM)
        .bind(&cert_path)
        .bind(&key_path)
        .bind(CERTIFICATE_STATUS_ACTIVE)
        .bind(metadata.to_string())
        .bind(&request.idempotency_key)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert custom deploy_certificate", error))?;

        Ok(CertificateResponse {
            id: uuid,
            cert_name: request.cert_name.clone(),
            cert_type: Some(CERTIFICATE_TYPE_CUSTOM),
            issuer: None,
            not_before: None,
            not_after: None,
            auto_renew: Some(false),
            status: CERTIFICATE_STATUS_ACTIVE,
            created_at: now,
        })
    }

    pub(super) async fn delete_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<()> {
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE deploy_certificate
             SET status = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND status <> $3",
        )
        .bind(tenant_id)
        .bind(certificate_id)
        .bind(CERTIFICATE_STATUS_REVOKED)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("revoke deploy_certificate", error))?;

        if result.rows_affected() == 0 {
            return Err(DeployServiceError::not_found("certificate not found"));
        }
        Ok(())
    }

    pub(super) async fn renew_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse> {
        let existing = self
            .retrieve_certificate_repo(tenant_id, certificate_id)
            .await?;
        if existing.cert_type != Some(CERTIFICATE_TYPE_LETS_ENCRYPT) {
            return Err(DeployServiceError::validation(
                "only managed Let's Encrypt certificates can be renewed; custom certificates must use certificates.upload",
            ));
        }

        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE deploy_certificate
             SET renewal_status = $3, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND status <> $5",
        )
        .bind(tenant_id)
        .bind(certificate_id)
        .bind(CERTIFICATE_RENEWAL_STATUS_PLANNED)
        .bind(&now)
        .bind(CERTIFICATE_STATUS_REVOKED)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("schedule deploy_certificate renewal", error))?;

        if result.rows_affected() == 0 {
            return Err(DeployServiceError::not_found("certificate not found"));
        }

        self.retrieve_certificate_repo(tenant_id, certificate_id)
            .await
    }
}

fn map_certificate_row(row: &AnyRow) -> Result<CertificateResponse, sqlx::Error> {
    Ok(CertificateResponse {
        id: row.try_get("uuid")?,
        cert_name: row.try_get("cert_name")?,
        cert_type: Some(row.try_get("cert_type")?),
        issuer: row.try_get("issuer").ok(),
        not_before: row.try_get("not_before").ok(),
        not_after: row.try_get("not_after").ok(),
        auto_renew: Some(row.try_get("auto_renew")?),
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
