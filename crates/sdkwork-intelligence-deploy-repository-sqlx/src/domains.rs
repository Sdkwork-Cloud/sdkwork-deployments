use sdkwork_deploy_contract::{
    CreateDomainRequest, DeployServiceError, DeployServiceResult, DomainPage, DomainResponse,
};
use sdkwork_intelligence_deploy_service::dns_txt_record_name;
use sdkwork_intelligence_deploy_service::DomainVerificationChallenge;
use sdkwork_utils_rust::crypto::sha256_hash;
use sqlx::{postgres::PgRow, Row};

use crate::support::{
    bool_from_row, new_uuid, next_id, now_rfc3339, pagination, resolve_site_internal_id,
    store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_domains_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_domain
             WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_domain", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, hostname, is_primary, is_verified, ssl_enabled, ssl_provider, status, created_at
             FROM deploy_domain
             WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_domain", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_domain_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_domain row: {error}"))
            })?);
        }

        Ok(DomainPage { items, total })
    }

    pub(super) async fn create_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<DomainResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        if request.is_primary {
            sqlx::query(
                "UPDATE deploy_domain SET is_primary = 0, updated_at = $3
                 WHERE tenant_id = $1 AND site_id = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id)
            .bind(site_internal_id)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("clear primary deploy_domain", error))?;
        }

        sqlx::query(
            "INSERT INTO deploy_domain (
                id, uuid, tenant_id, site_id, hostname, is_primary, is_verified,
                ssl_enabled, ssl_provider, status, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 0, $7, $8, 0, '{}', $9, $9, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(&request.hostname)
        .bind(request.is_primary)
        .bind(request.ssl_enabled)
        .bind(&request.ssl_provider)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_domain", error))?;

        self.retrieve_domain_repo(tenant_id, site_id, &uuid).await
    }

    pub(super) async fn retrieve_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT uuid, hostname, is_primary, is_verified, ssl_enabled, ssl_provider, status, created_at
             FROM deploy_domain
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_domain", error))?
        .ok_or_else(|| DeployServiceError::not_found("domain not found"))?;

        map_domain_row(&row).map_err(|error| DeployServiceError::Internal(error.to_string()))
    }

    pub(super) async fn delete_domain_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let now = now_rfc3339();
        let result = sqlx::query(
            "UPDATE deploy_domain
             SET deleted_at = $4, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("delete deploy_domain", error))?;

        if result.rows_affected() == 0 {
            return Err(DeployServiceError::not_found("domain not found"));
        }
        Ok(())
    }

    pub(super) async fn domain_verification_challenge_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainVerificationChallenge> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let row = sqlx::query(
            "SELECT id, hostname, is_verified FROM deploy_domain
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("verify deploy_domain lookup", error))?
        .ok_or_else(|| DeployServiceError::not_found("domain not found"))?;

        let is_verified = bool_from_row(&row, "is_verified").unwrap_or(false);
        let hostname: String = row.try_get("hostname").map_err(|error| {
            DeployServiceError::Internal(format!("map deploy_domain hostname: {error}"))
        })?;

        if is_verified {
            return Ok(DomainVerificationChallenge {
                verification_id: None,
                hostname,
                record_name: None,
                verified: true,
                proof_sha256: None,
                token: None,
                expires_at: None,
            });
        }

        let domain_internal_id: i64 = row.try_get("id").map_err(|error| {
            DeployServiceError::Internal(format!("map deploy_domain id: {error}"))
        })?;
        let now = chrono::Utc::now();
        let now_text = now.to_rfc3339();
        let active = sqlx::query(
            "SELECT uuid, record_name, proof_sha256, expires_at
             FROM deploy_domain_verification
             WHERE tenant_id = $1 AND domain_id = $2 AND status IN ('PENDING', 'CHECKING')
             ORDER BY id DESC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(domain_internal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("load active deploy_domain_verification", error))?;

        if let Some(active) = active {
            let expires_at: String = active.try_get("expires_at").map_err(|error| {
                DeployServiceError::Internal(format!(
                    "map deploy_domain_verification expiry: {error}"
                ))
            })?;
            let expired = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map(|value| value <= now)
                .unwrap_or(true);
            if !expired {
                return Ok(DomainVerificationChallenge {
                    verification_id: Some(active.try_get("uuid").map_err(|error| {
                        DeployServiceError::Internal(format!(
                            "map deploy_domain_verification uuid: {error}"
                        ))
                    })?),
                    hostname,
                    record_name: Some(active.try_get("record_name").map_err(|error| {
                        DeployServiceError::Internal(format!(
                            "map deploy_domain_verification record: {error}"
                        ))
                    })?),
                    verified: false,
                    proof_sha256: Some(active.try_get("proof_sha256").map_err(|error| {
                        DeployServiceError::Internal(format!(
                            "map deploy_domain_verification proof: {error}"
                        ))
                    })?),
                    token: None,
                    expires_at: Some(expires_at),
                });
            }
            sqlx::query(
                "UPDATE deploy_domain_verification
                 SET status = 'EXPIRED', updated_at = $3, version = version + 1
                 WHERE tenant_id = $1 AND domain_id = $2
                   AND status IN ('PENDING', 'CHECKING')",
            )
            .bind(tenant_id)
            .bind(domain_internal_id)
            .bind(&now_text)
            .execute(&self.pool)
            .await
            .map_err(|error| store_error("expire deploy_domain_verification", error))?;
        }

        let verification_id = new_uuid();
        let verification_internal_id = next_id(self.id_generator())?;
        let token = format!("sdkwork-domain-verification={}", new_uuid());
        let proof_sha256 = sha256_hash(token.as_bytes());
        let record_name = dns_txt_record_name(&hostname)?;
        let expires_at = (now + chrono::Duration::minutes(30)).to_rfc3339();
        sqlx::query(
            "INSERT INTO deploy_domain_verification (
                id, uuid, tenant_id, domain_id, method, record_name, proof_sha256, status,
                attempt_count, next_attempt_at, expires_at, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, 'DNS_TXT', $5, $6, 'PENDING', 0, $7, $8, $7, $7, 1
             )",
        )
        .bind(verification_internal_id)
        .bind(&verification_id)
        .bind(tenant_id)
        .bind(domain_internal_id)
        .bind(&record_name)
        .bind(&proof_sha256)
        .bind(&now_text)
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_domain_verification", error))?;

        Ok(DomainVerificationChallenge {
            verification_id: Some(verification_id),
            hostname,
            record_name: Some(record_name),
            verified: false,
            proof_sha256: Some(proof_sha256),
            token: Some(token),
            expires_at: Some(expires_at),
        })
    }

    pub(super) async fn confirm_domain_verification_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
        verification_id: &str,
        observed_sha256: &str,
        verifier_identity: &str,
    ) -> DeployServiceResult<bool> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let now = now_rfc3339();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin confirm deploy_domain verification", error))?;
        let verification = sqlx::query(
            "UPDATE deploy_domain_verification
             SET status = 'VERIFIED', observed_sha256 = $5, verifier_identity = $6,
                 checked_at = $7, verified_at = $7, attempt_count = attempt_count + 1,
                 updated_at = $7, version = version + 1
             WHERE tenant_id = $1 AND uuid = $4 AND domain_id = (
                 SELECT id FROM deploy_domain
                 WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3
                   AND is_verified = 0 AND deleted_at IS NULL
             ) AND proof_sha256 = $5 AND status IN ('PENDING', 'CHECKING')
               AND expires_at > $7",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .bind(verification_id)
        .bind(observed_sha256)
        .bind(verifier_identity)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("confirm deploy_domain_verification", error))?;

        if verification.rows_affected() != 1 {
            transaction.rollback().await.map_err(|error| {
                store_error("rollback rejected deploy_domain verification", error)
            })?;
            return Ok(false);
        }

        let result = sqlx::query(
            "UPDATE deploy_domain
             SET is_verified = 1, status = 1, updated_at = $4, version = version + 1
             WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3
               AND is_verified = 0 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(domain_id)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("activate verified deploy_domain", error))?;

        if result.rows_affected() != 1 {
            transaction
                .rollback()
                .await
                .map_err(|error| store_error("rollback deploy_domain activation", error))?;
            return Ok(false);
        }
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit deploy_domain verification", error))?;

        Ok(true)
    }
}

fn map_domain_row(row: &PgRow) -> Result<DomainResponse, sqlx::Error> {
    Ok(DomainResponse {
        id: row.try_get("uuid")?,
        hostname: row.try_get("hostname")?,
        is_primary: bool_from_row(row, "is_primary")?,
        is_verified: bool_from_row(row, "is_verified")?,
        ssl_enabled: bool_from_row(row, "ssl_enabled")?,
        ssl_provider: row.try_get("ssl_provider").ok(),
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
