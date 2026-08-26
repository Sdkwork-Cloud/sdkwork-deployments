//! TLS control plane repository operations (TECH-cloud-app-publishing §4.5):
//! ACME accounts, certificate order/challenge state machines, and certificate
//! version storage. The table schema (migration 0004) is the state machine
//! authority; every transition is an optimistic UPDATE guarded by the current
//! status column.

use sdkwork_deploy_contract::{
    AcmeAccountPage, AcmeAccountResponse, CertificateChallengePage, CertificateChallengeResponse,
    CertificateOrderPage, CertificateOrderResponse, CreateAcmeAccountRequest, DeployServiceError,
    DeployServiceResult, ORDER_STATUS_FAILED, ORDER_STATUS_FINALIZING, ORDER_STATUS_VERSION_STORED,
};
use sqlx::Row;

use crate::support::{
    new_uuid, next_id, now_rfc3339, optional_datetime, pagination, required_datetime, sha256_hex,
    store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn create_acme_account_repo(
        &self,
        tenant_id: i64,
        request: &CreateAcmeAccountRequest,
    ) -> DeployServiceResult<AcmeAccountResponse> {
        let account_id = next_id(self.id_generator())?;
        let account_uuid = new_uuid();
        let now = now_rfc3339();
        sqlx::query(
            "INSERT INTO deploy_acme_account
                (id, uuid, tenant_id, ca_profile, directory_url, contact_email,
                 external_account_digest, account_key_secret_ref, status,
                 created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'secret://acme-account-key/' || $2, 'ACTIVE', $8, $8)",
        )
        .bind(account_id)
        .bind(&account_uuid)
        .bind(tenant_id)
        .bind(&request.ca_profile)
        .bind(&request.directory_url)
        .bind(&request.contact_email)
        .bind(request.external_account_digest.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_acme_account", error))?;
        self.retrieve_acme_account_internal_repo(tenant_id, &account_uuid)
            .await
    }

    pub(super) async fn list_acme_accounts_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AcmeAccountPage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_acme_account
             WHERE tenant_id = $1 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_acme_account", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, tenant_id, ca_profile, directory_url, contact_email,
                    external_account_digest, status, created_at, updated_at, version
             FROM deploy_acme_account
             WHERE tenant_id = $1 AND deleted_at IS NULL
             ORDER BY updated_at DESC, id DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_acme_account", error))?;

        let items = rows
            .iter()
            .map(map_acme_account_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AcmeAccountPage {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn retrieve_acme_account_internal_repo(
        &self,
        tenant_id: i64,
        account_id: &str,
    ) -> DeployServiceResult<AcmeAccountResponse> {
        let row = sqlx::query(
            "SELECT uuid, tenant_id, ca_profile, directory_url, contact_email,
                    external_account_digest, status, created_at, updated_at, version
             FROM deploy_acme_account
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_acme_account", error))?;
        let Some(row) = row else {
            return Err(DeployServiceError::not_found("acme account not found"));
        };
        map_acme_account_row(&row)
    }

    /// Creates a certificate order with one challenge per identifier.
    /// Idempotent on `(tenant_id, idempotency_key)`: a replay returns the
    /// existing order. `requested_version_no` is the next version of the
    /// certificate being requested; the caller computes it.
    pub(super) async fn request_certificate_order_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        idempotency_key: &str,
        challenge_type: &str,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let certificate_row = sqlx::query(
            "SELECT c.id, c.uuid, c.renewal_status, COALESCE(c.current_version_id, 0) AS current_version_id
             FROM deploy_certificate c
             WHERE c.tenant_id = $1 AND c.uuid = $2 AND c.status <> 'REVOKED' AND c.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(certificate_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve deploy_certificate for order", error))?;
        let Some(certificate_row) = certificate_row else {
            return Err(DeployServiceError::not_found("certificate not found"));
        };
        let certificate_internal_id: i64 = certificate_row.try_get("id").unwrap_or(0);

        // Idempotency replay: return the existing order when present.
        let existing = sqlx::query(
            "SELECT o.uuid, o.tenant_id, c.uuid AS certificate_uuid, a.uuid AS account_uuid,
                    o.requested_version_no, o.request_sha256, o.idempotency_key,
                    o.external_order_digest, o.status, o.attempt_count, o.last_error_code,
                    o.deadline_at, o.created_at, o.updated_at, o.version
             FROM deploy_certificate_order o
             JOIN deploy_certificate c ON c.id = o.certificate_id
             JOIN deploy_acme_account a ON a.id = o.acme_account_id
             WHERE o.tenant_id = $1 AND o.idempotency_key = $2
             ORDER BY o.created_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find certificate order by idempotency", error))?;
        if let Some(row) = existing {
            return map_certificate_order_row(&row);
        }

        // Resolve the tenant's active ACME account (prefer production profile,
        // fall back to any ACTIVE account).
        let account_row = sqlx::query(
            "SELECT uuid FROM deploy_acme_account
             WHERE tenant_id = $1 AND status = 'ACTIVE' AND deleted_at IS NULL
             ORDER BY (ca_profile = 'LETS_ENCRYPT_PRODUCTION') DESC, updated_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve acme account for order", error))?;
        let Some(account_row) = account_row else {
            return Err(DeployServiceError::validation(
                "no active ACME account for this tenant; create one first",
            ));
        };
        let account_uuid: String = account_row.try_get("uuid").unwrap_or_default();

        // Next version of the certificate being requested.
        let next_version_row = sqlx::query(
            "SELECT COALESCE(MAX(version_no), 0) + 1 AS next_version_no
             FROM deploy_certificate_version WHERE certificate_id = $1",
        )
        .bind(certificate_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("compute next certificate version", error))?;
        let requested_version_no: i64 = next_version_row.try_get("next_version_no").unwrap_or(1);

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin certificate order", error))?;
        let order_id = next_id(self.id_generator())?;
        let order_uuid = new_uuid();
        let now = now_rfc3339();
        let request_sha256 = sha256_hex(&format!("{certificate_id}:{idempotency_key}"));
        let deadline = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
            .unwrap_or(now.clone());
        let account_internal_id =
            resolve_acme_account_internal_id(&mut transaction, &account_uuid).await?;
        let inserted = sqlx::query(
            "INSERT INTO deploy_certificate_order
                (id, uuid, tenant_id, certificate_id, acme_account_id, requested_version_no,
                 request_sha256, idempotency_key, status, attempt_count, deadline_at,
                 created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'REQUESTED', 0, $9, $10, $10)
             ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
        )
        .bind(order_id)
        .bind(&order_uuid)
        .bind(tenant_id)
        .bind(certificate_internal_id)
        .bind(account_internal_id)
        .bind(requested_version_no)
        .bind(&request_sha256)
        .bind(idempotency_key)
        .bind(&deadline)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert deploy_certificate_order", error))?;
        if inserted.rows_affected() == 0 {
            // Concurrent replay of the same idempotency key: commit and return
            // the winner's order.
            transaction
                .commit()
                .await
                .map_err(|error| store_error("commit certificate order replay", error))?;
            return self
                .find_certificate_order_by_idempotency_repo(tenant_id, idempotency_key)
                .await?
                .ok_or_else(|| {
                    DeployServiceError::Internal(
                        "certificate order disappeared after concurrent insert".into(),
                    )
                });
        }

        // One challenge per certificate identifier.
        let identifiers = sqlx::query(
            "SELECT id, hostname_ascii FROM deploy_certificate_identifier
             WHERE certificate_id = $1 ORDER BY position ASC",
        )
        .bind(certificate_internal_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|error| store_error("list certificate identifiers", error))?;
        for identifier in &identifiers {
            let identifier_id: i64 = identifier.try_get("id").unwrap_or(0);
            let hostname: String = identifier.try_get("hostname_ascii").unwrap_or_default();
            let challenge_id = next_id(self.id_generator())?;
            let challenge_uuid = new_uuid();
            // Deterministic proof placeholder: the key authorization hash is
            // produced by the ACME client boundary once credentials exist; the
            // placeholder keeps the state machine exercisable end to end.
            let proof = sha256_hex(&format!("{order_uuid}:{hostname}:{challenge_type}"));
            sqlx::query(
                "INSERT INTO deploy_certificate_challenge
                    (id, uuid, tenant_id, order_id, identifier_id, challenge_type,
                     proof_sha256, status, attempt_count, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 'PENDING', 0, $8, $8)",
            )
            .bind(challenge_id)
            .bind(&challenge_uuid)
            .bind(tenant_id)
            .bind(order_id)
            .bind(identifier_id)
            .bind(challenge_type)
            .bind(&proof)
            .bind(&now)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert deploy_certificate_challenge", error))?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit certificate order", error))?;
        self.retrieve_certificate_order_internal_repo(tenant_id, &order_uuid)
            .await
    }

    pub(super) async fn find_certificate_order_by_idempotency_repo(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<CertificateOrderResponse>> {
        let row = sqlx::query(
            "SELECT o.uuid, o.tenant_id, c.uuid AS certificate_uuid, a.uuid AS account_uuid,
                    o.requested_version_no, o.request_sha256, o.idempotency_key,
                    o.external_order_digest, o.status, o.attempt_count, o.last_error_code,
                    o.deadline_at, o.created_at, o.updated_at, o.version
             FROM deploy_certificate_order o
             JOIN deploy_certificate c ON c.id = o.certificate_id
             JOIN deploy_acme_account a ON a.id = o.acme_account_id
             WHERE o.tenant_id = $1 AND o.idempotency_key = $2",
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find certificate order by idempotency", error))?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(map_certificate_order_row(&row)?))
    }

    /// Optimistically advances the order state machine one step. Returns the
    /// previous status when the transition is not applicable (no-op) and the
    /// new status on success.
    pub(super) async fn advance_certificate_order_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
        from_status: &str,
        to_status: &str,
    ) -> DeployServiceResult<String> {
        let order_internal_id =
            resolve_certificate_order_internal_id(&self.pool, tenant_id, order_id).await?;
        let result = sqlx::query(
            "UPDATE deploy_certificate_order
                SET status = $1, attempt_count = attempt_count + 1, updated_at = NOW(),
                    version = version + 1
             WHERE id = $2 AND status = $3",
        )
        .bind(to_status)
        .bind(order_internal_id)
        .bind(from_status)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("advance deploy_certificate_order", error))?;
        if result.rows_affected() == 0 {
            return Ok(from_status.to_owned());
        }
        Ok(to_status.to_owned())
    }

    pub(super) async fn fail_certificate_order_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
        error_code: &str,
    ) -> DeployServiceResult<()> {
        let order_internal_id =
            resolve_certificate_order_internal_id(&self.pool, tenant_id, order_id).await?;
        sqlx::query(
            "UPDATE deploy_certificate_order
                SET status = $1, last_error_code = $2, updated_at = NOW(), version = version + 1
             WHERE id = $3 AND status NOT IN ($1, 'VERSION_STORED', 'CANCELLED')",
        )
        .bind(ORDER_STATUS_FAILED)
        .bind(error_code)
        .bind(order_internal_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("fail deploy_certificate_order", error))?;
        Ok(())
    }

    /// Records a challenge validation result. A VALID challenge advances the
    /// order to FINALIZING; a FAILED challenge fails the order. `challenge_id`
    /// may be `None` when the order has no challenges (all-order validation).
    pub(super) async fn record_challenge_result_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
        challenge_id: Option<&str>,
        valid: bool,
        error_code: Option<&str>,
    ) -> DeployServiceResult<()> {
        let order_internal_id =
            resolve_certificate_order_internal_id(&self.pool, tenant_id, order_id).await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin challenge result", error))?;

        if let Some(challenge_id) = challenge_id {
            let challenge_internal_id = sqlx::query(
                "SELECT id FROM deploy_certificate_challenge
                 WHERE tenant_id = $1 AND uuid = $2 AND order_id = $3",
            )
            .bind(tenant_id)
            .bind(challenge_id)
            .bind(order_internal_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("resolve challenge", error))?
            .and_then(|row| row.try_get::<i64, _>("id").ok())
            .ok_or_else(|| DeployServiceError::not_found("challenge not found"))?;
            let (challenge_status, validated_at) = if valid {
                ("VALID", Some(now_rfc3339()))
            } else {
                ("FAILED", None)
            };
            sqlx::query(
                "UPDATE deploy_certificate_challenge
                    SET status = $1, validated_at = COALESCE($2, validated_at),
                        last_error_code = $3, checked_at = NOW(),
                        updated_at = NOW(), version = version + 1
                 WHERE id = $4 AND status NOT IN ('VALID', 'FAILED', 'CLEANED')",
            )
            .bind(challenge_status)
            .bind(validated_at)
            .bind(error_code)
            .bind(challenge_internal_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("update challenge result", error))?;
        }

        if valid {
            sqlx::query(
                "UPDATE deploy_certificate_order
                    SET status = $1, updated_at = NOW(), version = version + 1
                 WHERE id = $2 AND status = 'CHALLENGE_VALIDATING'",
            )
            .bind(ORDER_STATUS_FINALIZING)
            .bind(order_internal_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("advance order to finalizing", error))?;
        } else {
            sqlx::query(
                "UPDATE deploy_certificate_order
                    SET status = $1, last_error_code = COALESCE($2, last_error_code),
                        updated_at = NOW(), version = version + 1
                 WHERE id = $3 AND status NOT IN ('VERSION_STORED', 'CANCELLED')",
            )
            .bind(ORDER_STATUS_FAILED)
            .bind(error_code)
            .bind(order_internal_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("fail order on challenge result", error))?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit challenge result", error))?;
        Ok(())
    }

    /// Stores the issued certificate version (CANDIDATE), activates it, and
    /// advances the order to VERSION_STORED in one transaction.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn store_certificate_version_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
        version_no: i64,
        serial_sha256: &str,
        fingerprint_sha256: &str,
        spki_sha256: &str,
        chain_sha256: &str,
        issuer: &str,
        subject: &str,
        key_algorithm: &str,
        not_before: &str,
        not_after: &str,
        secret_bundle_ref: &str,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let order_internal_id =
            resolve_certificate_order_internal_id(&self.pool, tenant_id, order_id).await?;
        let order_row = sqlx::query(
            "SELECT o.certificate_id, o.uuid, o.status FROM deploy_certificate_order o
             WHERE o.id = $1",
        )
        .bind(order_internal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve order for version storage", error))?;
        let Some(order_row) = order_row else {
            return Err(DeployServiceError::not_found("certificate order not found"));
        };
        let certificate_internal_id: i64 = order_row.try_get("certificate_id").unwrap_or(0);
        let order_uuid: String = order_row.try_get("uuid").unwrap_or_default();
        let order_status: String = order_row.try_get("status").unwrap_or_default();
        if order_status != ORDER_STATUS_FINALIZING {
            return Err(DeployServiceError::conflict(format!(
                "certificate order must be FINALIZING to store a version, got {order_status}"
            )));
        }

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin certificate version storage", error))?;
        let version_id = next_id(self.id_generator())?;
        let version_uuid = new_uuid();
        let now = now_rfc3339();
        let inserted = sqlx::query(
            "INSERT INTO deploy_certificate_version
                (id, uuid, tenant_id, certificate_id, version_no, serial_sha256,
                 fingerprint_sha256, spki_sha256, chain_sha256, issuer, subject,
                 key_algorithm, not_before, not_after, secret_bundle_ref, source_order_id,
                 status, created_by, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                     'CANDIDATE', 0, $17)
             ON CONFLICT (certificate_id, version_no) DO NOTHING",
        )
        .bind(version_id)
        .bind(&version_uuid)
        .bind(tenant_id)
        .bind(certificate_internal_id)
        .bind(version_no)
        .bind(serial_sha256)
        .bind(fingerprint_sha256)
        .bind(spki_sha256)
        .bind(chain_sha256)
        .bind(issuer)
        .bind(subject)
        .bind(key_algorithm)
        .bind(not_before)
        .bind(not_after)
        .bind(secret_bundle_ref)
        .bind(order_internal_id)
        .bind(&now)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert deploy_certificate_version", error))?;
        if inserted.rows_affected() > 0 {
            // Activate the new version and supersede the previous one.
            sqlx::query(
                "UPDATE deploy_certificate_version SET status = 'SUPERSEDED'
                 WHERE certificate_id = $1 AND version_no <> $2 AND status = 'ACTIVE'",
            )
            .bind(certificate_internal_id)
            .bind(version_no)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("supersede certificate version", error))?;
            sqlx::query(
                "UPDATE deploy_certificate_version SET status = 'ACTIVE'
                 WHERE id = $1",
            )
            .bind(version_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("activate certificate version", error))?;
            sqlx::query(
                "UPDATE deploy_certificate
                    SET current_version_id = $1, renewal_status = 'NONE', status = 'ACTIVE',
                        updated_at = NOW(), version = version + 1
                 WHERE id = $2 AND deleted_at IS NULL",
            )
            .bind(version_id)
            .bind(certificate_internal_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("update certificate current version", error))?;
        }
        sqlx::query(
            "UPDATE deploy_certificate_order
                SET status = $1, updated_at = NOW(), version = version + 1
             WHERE id = $2",
        )
        .bind(ORDER_STATUS_VERSION_STORED)
        .bind(order_internal_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("complete certificate order", error))?;
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit certificate version storage", error))?;
        self.retrieve_certificate_order_internal_repo(tenant_id, &order_uuid)
            .await
    }

    pub(super) async fn retrieve_certificate_order_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        self.retrieve_certificate_order_internal_repo(tenant_id, order_id)
            .await
    }

    pub(super) async fn list_certificate_orders_repo(
        &self,
        tenant_id: i64,
        certificate_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateOrderPage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let certificate_internal_id =
            resolve_certificate_internal_id_light(&self.pool, tenant_id, certificate_id).await?;
        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_certificate_order
             WHERE tenant_id = $1 AND certificate_id = $2",
        )
        .bind(tenant_id)
        .bind(certificate_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count certificate orders", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT o.uuid, o.tenant_id, c.uuid AS certificate_uuid, a.uuid AS account_uuid,
                    o.requested_version_no, o.request_sha256, o.idempotency_key,
                    o.external_order_digest, o.status, o.attempt_count, o.last_error_code,
                    o.deadline_at, o.created_at, o.updated_at, o.version
             FROM deploy_certificate_order o
             JOIN deploy_certificate c ON c.id = o.certificate_id
             JOIN deploy_acme_account a ON a.id = o.acme_account_id
             WHERE o.tenant_id = $1 AND o.certificate_id = $2
             ORDER BY o.created_at DESC, o.id DESC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(certificate_internal_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list certificate orders", error))?;

        let items = rows
            .iter()
            .map(map_certificate_order_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CertificateOrderPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn list_certificate_challenges_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateChallengePage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let order_internal_id =
            resolve_certificate_order_internal_id(&self.pool, tenant_id, order_id).await?;
        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_certificate_challenge
             WHERE tenant_id = $1 AND order_id = $2",
        )
        .bind(tenant_id)
        .bind(order_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count certificate challenges", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT ch.uuid, ch.tenant_id, o.uuid AS order_uuid, i.uuid AS identifier_uuid,
                    i.hostname_ascii, ch.challenge_type, ch.proof_sha256, ch.presentation_ref,
                    ch.status, ch.attempt_count, ch.checked_at, ch.validated_at,
                    ch.last_error_code, ch.created_at, ch.updated_at, ch.version
             FROM deploy_certificate_challenge ch
             JOIN deploy_certificate_order o ON o.id = ch.order_id
             JOIN deploy_certificate_identifier i ON i.id = ch.identifier_id
             WHERE ch.tenant_id = $1 AND ch.order_id = $2
             ORDER BY ch.id ASC LIMIT $3 OFFSET $4",
        )
        .bind(tenant_id)
        .bind(order_internal_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list certificate challenges", error))?;

        let items = rows
            .iter()
            .map(map_certificate_challenge_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CertificateChallengePage {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn retrieve_certificate_order_internal_repo(
        &self,
        tenant_id: i64,
        order_id: &str,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let row = sqlx::query(
            "SELECT o.uuid, o.tenant_id, c.uuid AS certificate_uuid, a.uuid AS account_uuid,
                    o.requested_version_no, o.request_sha256, o.idempotency_key,
                    o.external_order_digest, o.status, o.attempt_count, o.last_error_code,
                    o.deadline_at, o.created_at, o.updated_at, o.version
             FROM deploy_certificate_order o
             JOIN deploy_certificate c ON c.id = o.certificate_id
             JOIN deploy_acme_account a ON a.id = o.acme_account_id
             WHERE o.tenant_id = $1 AND o.uuid = $2",
        )
        .bind(tenant_id)
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve certificate order", error))?;
        let Some(row) = row else {
            return Err(DeployServiceError::not_found("certificate order not found"));
        };
        map_certificate_order_row(&row)
    }
}

async fn resolve_acme_account_internal_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_uuid: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_acme_account WHERE uuid = $1 AND status = 'ACTIVE' AND deleted_at IS NULL",
    )
    .bind(account_uuid)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| store_error("resolve acme account id", error))?;
    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("acme account not found"))
}

async fn resolve_certificate_internal_id_light(
    pool: &sqlx::PgPool,
    tenant_id: i64,
    certificate_id: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_certificate
         WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(certificate_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve certificate id", error))?;
    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("certificate not found"))
}

async fn resolve_certificate_order_internal_id(
    pool: &sqlx::PgPool,
    tenant_id: i64,
    order_id: &str,
) -> Result<i64, DeployServiceError> {
    let row =
        sqlx::query("SELECT id FROM deploy_certificate_order WHERE tenant_id = $1 AND uuid = $2")
            .bind(tenant_id)
            .bind(order_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| store_error("resolve certificate order id", error))?;
    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("certificate order not found"))
}

fn map_acme_account_row(
    row: &sqlx::postgres::PgRow,
) -> Result<AcmeAccountResponse, DeployServiceError> {
    let created_at = required_datetime(row, "created_at")?;
    let updated_at = required_datetime(row, "updated_at")?;
    let version: i64 = row.try_get("version").unwrap_or(1);
    Ok(AcmeAccountResponse {
        id: row.try_get("uuid").unwrap_or_default(),
        tenant_id: row.try_get("tenant_id").unwrap_or(0),
        ca_profile: row.try_get("ca_profile").unwrap_or_default(),
        directory_url: row.try_get("directory_url").unwrap_or_default(),
        contact_email: row.try_get("contact_email").unwrap_or_default(),
        external_account_digest: row.try_get("external_account_digest").ok(),
        status: row.try_get("status").unwrap_or_default(),
        created_at,
        updated_at,
        version: version.to_string(),
    })
}

fn map_certificate_order_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CertificateOrderResponse, DeployServiceError> {
    let deadline_at = required_datetime(row, "deadline_at")?;
    let created_at = required_datetime(row, "created_at")?;
    let updated_at = required_datetime(row, "updated_at")?;
    let version: i64 = row.try_get("version").unwrap_or(1);
    Ok(CertificateOrderResponse {
        id: row.try_get("uuid").unwrap_or_default(),
        tenant_id: row.try_get("tenant_id").unwrap_or(0),
        certificate_id: row.try_get("certificate_uuid").unwrap_or_default(),
        acme_account_id: row.try_get("account_uuid").unwrap_or_default(),
        requested_version_no: row.try_get("requested_version_no").unwrap_or(0),
        request_sha256: row.try_get("request_sha256").unwrap_or_default(),
        idempotency_key: row.try_get("idempotency_key").unwrap_or_default(),
        external_order_digest: row.try_get("external_order_digest").ok(),
        status: row.try_get("status").unwrap_or_default(),
        attempt_count: row.try_get("attempt_count").unwrap_or(0),
        last_error_code: row.try_get("last_error_code").ok(),
        deadline_at,
        created_at,
        updated_at,
        version: version.to_string(),
    })
}

fn map_certificate_challenge_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CertificateChallengeResponse, DeployServiceError> {
    let checked_at = optional_datetime(row, "checked_at")?;
    let validated_at = optional_datetime(row, "validated_at")?;
    let created_at = required_datetime(row, "created_at")?;
    let updated_at = required_datetime(row, "updated_at")?;
    let version: i64 = row.try_get("version").unwrap_or(1);
    Ok(CertificateChallengeResponse {
        id: row.try_get("uuid").unwrap_or_default(),
        tenant_id: row.try_get("tenant_id").unwrap_or(0),
        order_id: row.try_get("order_uuid").unwrap_or_default(),
        identifier_id: row.try_get("identifier_uuid").unwrap_or_default(),
        hostname: row.try_get("hostname_ascii").unwrap_or_default(),
        challenge_type: row.try_get("challenge_type").unwrap_or_default(),
        proof_sha256: row.try_get("proof_sha256").unwrap_or_default(),
        presentation_ref: row.try_get("presentation_ref").ok(),
        status: row.try_get("status").unwrap_or_default(),
        attempt_count: row.try_get("attempt_count").unwrap_or(0),
        checked_at,
        validated_at,
        last_error_code: row.try_get("last_error_code").ok(),
        created_at,
        updated_at,
        version: version.to_string(),
    })
}

#[cfg(test)]
mod tls_state_machine_tests {
    use sdkwork_deploy_contract::next_order_transition;

    #[test]
    fn order_state_machine_is_forward_only_and_terminates() {
        assert_eq!(next_order_transition("REQUESTED"), Some("ACCOUNT_READY"));
        assert_eq!(
            next_order_transition("ACCOUNT_READY"),
            Some("ORDER_PENDING")
        );
        assert_eq!(
            next_order_transition("ORDER_PENDING"),
            Some("CHALLENGE_PRESENTING")
        );
        assert_eq!(
            next_order_transition("CHALLENGE_PRESENTING"),
            Some("CHALLENGE_VALIDATING")
        );
        assert_eq!(
            next_order_transition("CHALLENGE_VALIDATING"),
            Some("FINALIZING")
        );
        // Terminal states have no forward transition.
        assert_eq!(next_order_transition("FINALIZING"), None);
        assert_eq!(next_order_transition("VERSION_STORED"), None);
        assert_eq!(next_order_transition("FAILED"), None);
        assert_eq!(next_order_transition("CANCELLED"), None);
        // Unknown states never advance.
        assert_eq!(next_order_transition("GARBAGE"), None);
    }

    #[test]
    fn terminal_statuses_are_recognized() {
        for terminal in ["VERSION_STORED", "FAILED", "CANCELLED"] {
            assert_eq!(next_order_transition(terminal), None);
        }
    }
}
