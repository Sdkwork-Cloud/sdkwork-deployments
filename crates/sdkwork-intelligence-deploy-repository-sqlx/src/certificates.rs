use std::collections::BTreeSet;

use sdkwork_deploy_contract::{
    CertificatePage, CertificateResponse, CreateCertificateRequest, DeployServiceError,
    DeployServiceResult, CERTIFICATE_RENEWAL_STATUS_PLANNED, CERTIFICATE_SOURCE_MANAGED,
    CERTIFICATE_STATUS_REVOKED,
};
use sqlx::{postgres::PgRow, AssertSqlSafe, Row};

use crate::support::{
    datetime_from_row, json_from_row, new_uuid, next_id, optional_datetime_from_row, pagination,
    sha256_hex, store_error,
};
use crate::DeployRepository;

const CERTIFICATE_SELECT: &str = "c.uuid, c.cert_name, c.certificate_source, c.ca_profile,
     c.preferred_key_algorithm, c.auto_renew, c.renewal_status, c.status,
     c.created_at, c.updated_at, c.version,
     v.uuid AS current_version_uuid, v.issuer, v.not_before, v.not_after,
     COALESCE((
         SELECT jsonb_agg(ci.hostname_ascii ORDER BY ci.position)
         FROM deploy_certificate_identifier ci
         WHERE ci.certificate_id = c.id
     ), '[]'::jsonb) AS identifiers";

impl DeployRepository {
    pub(super) async fn list_certificates_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM deploy_certificate
             WHERE tenant_id = $1 AND status <> $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(CERTIFICATE_STATUS_REVOKED)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_certificate", error))?;
        let rows = sqlx::query(AssertSqlSafe(format!(
            "SELECT {CERTIFICATE_SELECT}
             FROM deploy_certificate c
             LEFT JOIN deploy_certificate_version v ON v.id = c.current_version_id
             WHERE c.tenant_id = $1 AND c.status <> $2 AND c.deleted_at IS NULL
             ORDER BY c.updated_at DESC, c.id DESC LIMIT $3 OFFSET $4"
        )))
        .bind(tenant_id)
        .bind(CERTIFICATE_STATUS_REVOKED)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_certificate", error))?;
        let items = rows
            .iter()
            .map(map_certificate_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_certificate row: {error}"))
            })?;
        Ok(CertificatePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn retrieve_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse> {
        let row = sqlx::query(AssertSqlSafe(format!(
            "SELECT {CERTIFICATE_SELECT}
             FROM deploy_certificate c
             LEFT JOIN deploy_certificate_version v ON v.id = c.current_version_id
             WHERE c.tenant_id = $1 AND c.uuid = $2
               AND c.status <> $3 AND c.deleted_at IS NULL"
        )))
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
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        idempotency_key: &str,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse> {
        validate_create_certificate(request, idempotency_key)?;
        let request_json = serde_json::to_string(request)
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
        let request_sha256 = sha256_hex(&request_json);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin create deploy_certificate", error))?;
        if let Some(existing) = sqlx::query(
            "SELECT uuid, request_sha256 FROM deploy_certificate
             WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| store_error("load idempotent deploy_certificate", error))?
        {
            let stored_hash: String = existing.try_get("request_sha256").map_err(|error| {
                DeployServiceError::Internal(format!("map certificate request hash: {error}"))
            })?;
            if stored_hash != request_sha256 {
                return Err(DeployServiceError::conflict(
                    "Idempotency-Key was already used with another certificate request",
                ));
            }
            let certificate_id: String = existing.try_get("uuid").map_err(|error| {
                DeployServiceError::Internal(format!("map certificate UUID: {error}"))
            })?;
            transaction
                .commit()
                .await
                .map_err(|error| store_error("commit idempotent deploy_certificate", error))?;
            return self
                .retrieve_certificate_repo(tenant_id, &certificate_id)
                .await;
        }

        let domain_ids = request
            .domain_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let domains = sqlx::query(
            "SELECT id, uuid, hostname_ascii, hostname_type
             FROM deploy_domain
             WHERE tenant_id = $1 AND uuid = ANY($2)
               AND verification_status = 'VERIFIED' AND status = 'ACTIVE'
               AND deleted_at IS NULL
             ORDER BY hostname_ascii, id
             FOR SHARE",
        )
        .bind(tenant_id)
        .bind(&domain_ids)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| store_error("resolve certificate hostname identifiers", error))?;
        if domains.len() != request.domain_ids.len() {
            return Err(DeployServiceError::validation(
                "every domainId must reference an active verified hostname in the current tenant",
            ));
        }

        let certificate_id = next_id(self.id_generator())?;
        let certificate_uuid = new_uuid();
        sqlx::query(
            "INSERT INTO deploy_certificate (
                id, uuid, tenant_id, organization_id, cert_name, certificate_source,
                ca_profile, preferred_key_algorithm, auto_renew, renewal_status, status,
                idempotency_key, request_sha256, created_by, updated_by
             ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,TRUE,'NONE','PENDING',$9,$10,$11,$11
             )",
        )
        .bind(certificate_id)
        .bind(&certificate_uuid)
        .bind(tenant_id)
        .bind(organization_id.unwrap_or(0))
        .bind(request.cert_name.trim())
        .bind(CERTIFICATE_SOURCE_MANAGED)
        .bind(&request.ca_profile)
        .bind(&request.preferred_key_algorithm)
        .bind(idempotency_key)
        .bind(&request_sha256)
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert deploy_certificate", error))?;

        for (position, domain) in domains.iter().enumerate() {
            sqlx::query(
                "INSERT INTO deploy_certificate_identifier (
                    id, uuid, tenant_id, certificate_id, domain_id, identifier_type,
                    hostname_ascii, position
                 ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            )
            .bind(next_id(self.id_generator())?)
            .bind(new_uuid())
            .bind(tenant_id)
            .bind(certificate_id)
            .bind(domain.try_get::<i64, _>("id").map_err(|error| {
                DeployServiceError::Internal(format!("map certificate domain id: {error}"))
            })?)
            .bind(
                domain
                    .try_get::<String, _>("hostname_type")
                    .map_err(|error| {
                        DeployServiceError::Internal(format!(
                            "map certificate identifier type: {error}"
                        ))
                    })?,
            )
            .bind(
                domain
                    .try_get::<String, _>("hostname_ascii")
                    .map_err(|error| {
                        DeployServiceError::Internal(format!("map certificate hostname: {error}"))
                    })?,
            )
            .bind(position as i32)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert deploy_certificate_identifier", error))?;
        }
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit deploy_certificate", error))?;
        self.retrieve_certificate_repo(tenant_id, &certificate_uuid)
            .await
    }

    pub(super) async fn delete_certificate_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<()> {
        let result = sqlx::query(
            "UPDATE deploy_certificate
             SET status = $3, auto_renew = FALSE, renewal_status = 'NONE',
                 updated_at = CURRENT_TIMESTAMP, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND status <> $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(certificate_id)
        .bind(CERTIFICATE_STATUS_REVOKED)
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
        let result = sqlx::query(
            "UPDATE deploy_certificate
             SET renewal_status = $3, updated_at = CURRENT_TIMESTAMP, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND certificate_source = $4
               AND auto_renew = TRUE AND status IN ('ACTIVE', 'FAILED')
               AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(certificate_id)
        .bind(CERTIFICATE_RENEWAL_STATUS_PLANNED)
        .bind(CERTIFICATE_SOURCE_MANAGED)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("schedule deploy_certificate renewal", error))?;
        if result.rows_affected() == 0 {
            return Err(DeployServiceError::validation(
                "only active managed certificates with auto-renew enabled can be renewed",
            ));
        }
        self.retrieve_certificate_repo(tenant_id, certificate_id)
            .await
    }
}

fn validate_create_certificate(
    request: &CreateCertificateRequest,
    idempotency_key: &str,
) -> DeployServiceResult<()> {
    if request.cert_name.trim().is_empty() || request.cert_name.trim().len() > 200 {
        return Err(DeployServiceError::validation(
            "certName must contain between 1 and 200 characters",
        ));
    }
    if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
        return Err(DeployServiceError::validation(
            "Idempotency-Key must contain between 1 and 128 characters",
        ));
    }
    if request.domain_ids.is_empty() || request.domain_ids.len() > 100 {
        return Err(DeployServiceError::validation(
            "domainIds must contain between 1 and 100 hostnames",
        ));
    }
    let unique_domain_ids = request.domain_ids.iter().collect::<BTreeSet<_>>();
    if unique_domain_ids.len() != request.domain_ids.len()
        || request.domain_ids.iter().any(|id| id.trim().is_empty())
    {
        return Err(DeployServiceError::validation(
            "domainIds must contain unique non-empty hostname identifiers",
        ));
    }
    if !matches!(
        request.ca_profile.as_str(),
        "LETS_ENCRYPT_STAGING" | "LETS_ENCRYPT_PRODUCTION"
    ) {
        return Err(DeployServiceError::validation(
            "caProfile must be LETS_ENCRYPT_STAGING or LETS_ENCRYPT_PRODUCTION",
        ));
    }
    if !matches!(request.preferred_key_algorithm.as_str(), "RSA" | "ECDSA") {
        return Err(DeployServiceError::validation(
            "preferredKeyAlgorithm must be RSA or ECDSA",
        ));
    }
    Ok(())
}

fn map_certificate_row(row: &PgRow) -> Result<CertificateResponse, sqlx::Error> {
    let identifiers =
        json_from_row(row, "identifiers")?.unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
    let identifiers = serde_json::from_value::<Vec<String>>(identifiers)
        .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    Ok(CertificateResponse {
        id: row.try_get("uuid")?,
        cert_name: row.try_get("cert_name")?,
        certificate_source: row.try_get("certificate_source")?,
        ca_profile: row.try_get("ca_profile")?,
        preferred_key_algorithm: row.try_get("preferred_key_algorithm")?,
        identifiers,
        current_version_id: row.try_get("current_version_uuid")?,
        issuer: row.try_get("issuer")?,
        not_before: optional_datetime_from_row(row, "not_before")?,
        not_after: optional_datetime_from_row(row, "not_after")?,
        auto_renew: row.try_get("auto_renew")?,
        renewal_status: row.try_get("renewal_status")?,
        status: row.try_get("status")?,
        created_at: datetime_from_row(row, "created_at")?,
        updated_at: datetime_from_row(row, "updated_at")?,
        version: row.try_get::<i64, _>("version")?.to_string(),
    })
}
