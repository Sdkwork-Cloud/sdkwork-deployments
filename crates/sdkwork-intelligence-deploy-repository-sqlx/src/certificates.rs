use sdkwork_deploy_contract::{
    CertificatePage, CertificateResponse, CreateCertificateRequest, DeployServiceError,
    DeployServiceResult,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{
    new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id, store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_certificates_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM deploy_certificate WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count deploy_certificate", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, cert_name, status, created_at
             FROM deploy_certificate
             WHERE tenant_id = $1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
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
                id, uuid, tenant_id, site_id, domain_id, cert_name, status, metadata,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, NULL, $5, 0, '{}', $6, $6, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&request.cert_name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_certificate", error))?;

        Ok(CertificateResponse {
            id: uuid,
            cert_name: request.cert_name.clone(),
            status: 0,
            created_at: now,
        })
    }
}

fn map_certificate_row(row: &AnyRow) -> Result<CertificateResponse, sqlx::Error> {
    Ok(CertificateResponse {
        id: row.try_get("uuid")?,
        cert_name: row.try_get("cert_name")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
