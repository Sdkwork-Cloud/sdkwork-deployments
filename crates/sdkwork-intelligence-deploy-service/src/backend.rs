//! Backend-api service surface implementation.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    next_order_transition, AcmeAccountPage, AcmeAccountResponse, BuildQueuePage,
    CertificateChallengePage, CertificateOrderPage, CertificateOrderResponse,
    CreateNginxConfigRequest, CreateNodeClusterRequest, CreateServerRequest, DeployBackendApi,
    DeployBackendRequestContext, DeployServiceError, DeployServiceResult,
    EntitlementProjectionPage, ListNginxConfigsQuery, RequestCertificateOrderRequest,
    RunnerHealthPage, StoreCertificateVersionRequest, UpdateNginxConfigRequest,
    UpdateNodeClusterRequest, UpdateServerRequest,
};

use crate::DeployService;

impl DeployService {
    fn backend_tenant_scope(
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<Option<i64>> {
        Ok(context.tenant_id)
    }

    fn backend_write_tenant(context: &DeployBackendRequestContext) -> DeployServiceResult<i64> {
        context
            .tenant_id
            .filter(|tenant_id| *tenant_id > 0)
            .ok_or(DeployServiceError::validation(
                "tenant context is required for backend write operations",
            ))
    }
}

#[async_trait]
impl DeployBackendApi for DeployService {
    async fn list_nginx_configs(
        &self,
        context: &DeployBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigPage> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.list_nginx_configs(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .create_nginx_config(tenant_id, request)
            .await
    }

    async fn retrieve_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .retrieve_nginx_config(tenant_id, config_id)
            .await
    }

    async fn update_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .update_nginx_config(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxValidateResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .validate_nginx_config(tenant_id, config_id)
            .await
    }

    async fn deploy_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .deploy_nginx_config(tenant_id, config_id)
            .await
    }

    async fn reload_nginx(
        &self,
        _context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxReloadResponse> {
        self.repository.reload_nginx().await
    }

    async fn retrieve_nginx_status(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxStatusResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.retrieve_nginx_status(tenant_id).await
    }

    async fn list_servers(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
        cluster_id: Option<String>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ServerPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_servers(tenant_id, page, page_size, cluster_id)
            .await
    }

    async fn create_server(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ServerResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository.create_server(tenant_id, request).await
    }

    async fn update_server(
        &self,
        context: &DeployBackendRequestContext,
        server_id: &str,
        request: &UpdateServerRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ServerResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .update_server(tenant_id, server_id, request)
            .await
    }

    async fn list_node_clusters(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NodeClusterPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_node_clusters(tenant_id, page, page_size)
            .await
    }

    async fn create_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNodeClusterRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NodeClusterResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .create_node_cluster(tenant_id, request)
            .await
    }

    async fn update_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        cluster_id: &str,
        request: &UpdateNodeClusterRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NodeClusterResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .update_node_cluster(tenant_id, cluster_id, request)
            .await
    }

    async fn list_audit_logs(
        &self,
        context: &DeployBackendRequestContext,
        query: &sdkwork_deploy_contract::AuditLogQuery,
        cursor: Option<&str>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AuditLogPage> {
        // 审计日志是租户私有数据：无租户上下文的令牌必须被拒绝，绝不回退为
        // 全库枚举（repository 层的 None 分支同时 fail-closed）。
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_audit_logs(Some(tenant_id), query, cursor)
            .await
    }

    async fn list_entitlement_projections(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<EntitlementProjectionPage> {
        // Platform management surface: tenant-scoped when the token carries a
        // tenant, platform-wide otherwise (platform operators).
        self.repository
            .list_entitlement_projections(context.tenant_id, page, page_size)
            .await
    }

    async fn list_build_queue(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildQueuePage> {
        self.repository
            .list_build_queue(context.tenant_id, page, page_size)
            .await
    }

    async fn list_runner_health(
        &self,
        _context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<RunnerHealthPage> {
        self.repository.list_runner_health(page, page_size).await
    }
    // -- TLS control plane orchestration (TECH-cloud-site-publishing §4.5) ---------

    async fn create_acme_account(
        &self,
        context: &DeployBackendRequestContext,
        request: &sdkwork_deploy_contract::CreateAcmeAccountRequest,
    ) -> DeployServiceResult<AcmeAccountResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        if !matches!(
            request.ca_profile.as_str(),
            "LETS_ENCRYPT_STAGING" | "LETS_ENCRYPT_PRODUCTION"
        ) {
            return Err(DeployServiceError::validation(
                "caProfile must be LETS_ENCRYPT_STAGING or LETS_ENCRYPT_PRODUCTION",
            ));
        }
        if !request.directory_url.starts_with("https://") || request.directory_url.len() > 2048 {
            return Err(DeployServiceError::validation(
                "directoryUrl must be an https URL (max 2048 characters)",
            ));
        }
        if request.contact_email.is_empty() || request.contact_email.len() > 320 {
            return Err(DeployServiceError::validation(
                "contactEmail must be 1..=320 characters",
            ));
        }
        if let Some(digest) = request.external_account_digest.as_deref() {
            if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(DeployServiceError::validation(
                    "externalAccountDigest must be 64 hexadecimal characters",
                ));
            }
        }
        self.repository
            .create_acme_account(tenant_id, request)
            .await
    }

    async fn list_acme_accounts(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AcmeAccountPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_acme_accounts(tenant_id, page, page_size)
            .await
    }

    async fn request_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        request: &RequestCertificateOrderRequest,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        if request.idempotency_key.trim().is_empty() {
            return Err(DeployServiceError::validation("idempotencyKey is required"));
        }
        if let Some(challenge_type) = request.challenge_type.as_deref() {
            if !matches!(challenge_type, "HTTP_01" | "DNS_01") {
                return Err(DeployServiceError::validation(
                    "challengeType must be HTTP_01 or DNS_01",
                ));
            }
        }
        self.repository
            .request_certificate_order(tenant_id, request)
            .await
    }

    async fn advance_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        // Discover the current state, then advance exactly one canonical step.
        // The retry loop absorbs a concurrent worker that advanced the order
        // between the read and the optimistic UPDATE.
        for _ in 0..2 {
            let current = self
                .repository
                .retrieve_certificate_order(tenant_id, order_id)
                .await?
                .status;
            let Some(next) = next_order_transition(&current) else {
                return Err(DeployServiceError::conflict(format!(
                    "certificate order {order_id} is at terminal state {current}"
                )));
            };
            let applied = self
                .repository
                .advance_certificate_order(tenant_id, order_id, &current, next)
                .await?;
            if applied == next {
                return self
                    .repository
                    .retrieve_certificate_order(tenant_id, order_id)
                    .await;
            }
        }
        Err(DeployServiceError::conflict(format!(
            "certificate order {order_id} is contended; retry"
        )))
    }

    async fn fail_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        error_code: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::backend_write_tenant(context)?;
        if error_code.is_empty() || error_code.len() > 64 {
            return Err(DeployServiceError::validation(
                "errorCode must be 1..=64 characters",
            ));
        }
        self.repository
            .fail_certificate_order(tenant_id, order_id, error_code)
            .await
    }

    async fn record_challenge_result(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        challenge_id: Option<&str>,
        valid: bool,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .record_challenge_result(tenant_id, order_id, challenge_id, valid, None)
            .await
    }

    async fn store_certificate_version(
        &self,
        context: &DeployBackendRequestContext,
        request: &StoreCertificateVersionRequest,
    ) -> DeployServiceResult<CertificateOrderResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        if request.version_no <= 0 {
            return Err(DeployServiceError::validation("versionNo must be positive"));
        }
        for (value, field) in [
            (&request.serial_sha256, "serialSha256"),
            (&request.fingerprint_sha256, "fingerprintSha256"),
            (&request.spki_sha256, "spkiSha256"),
            (&request.chain_sha256, "chainSha256"),
        ] {
            if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(DeployServiceError::validation(format!(
                    "{field} must be 64 hexadecimal characters"
                )));
            }
        }
        if !matches!(request.key_algorithm.as_str(), "RSA" | "ECDSA") {
            return Err(DeployServiceError::validation(
                "keyAlgorithm must be RSA or ECDSA",
            ));
        }
        if !(request.secret_bundle_ref.starts_with("secret://")
            || request.secret_bundle_ref.starts_with("file:"))
        {
            return Err(DeployServiceError::validation(
                "secretBundleRef must be a secret:// or file: reference",
            ));
        }
        if chrono::DateTime::parse_from_rfc3339(&request.not_before).is_err()
            || chrono::DateTime::parse_from_rfc3339(&request.not_after).is_err()
        {
            return Err(DeployServiceError::validation(
                "notBefore/notAfter must be RFC3339 timestamps",
            ));
        }
        self.repository
            .store_certificate_version(
                tenant_id,
                &request.order_id,
                request.version_no,
                &request.serial_sha256,
                &request.fingerprint_sha256,
                &request.spki_sha256,
                &request.chain_sha256,
                &request.issuer,
                &request.subject,
                &request.key_algorithm,
                &request.not_before,
                &request.not_after,
                &request.secret_bundle_ref,
            )
            .await
    }

    async fn list_certificate_orders(
        &self,
        context: &DeployBackendRequestContext,
        certificate_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateOrderPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_certificate_orders(tenant_id, certificate_id, page, page_size)
            .await
    }

    async fn list_certificate_challenges(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateChallengePage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_certificate_challenges(tenant_id, order_id, page, page_size)
            .await
    }
}
