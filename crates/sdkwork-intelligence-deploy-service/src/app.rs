//! App-api service surface implementation.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    is_deploy_package_artifact_type, CancelDeployUploadSessionRequest,
    CompleteDeployUploadSessionRequest, CreateCertificateRequest, CreateDeployUploadSessionRequest,
    CreateDeploymentRequest, CreateDomainRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateReleaseRequest, CreateSiteRequest, DeployAppApi,
    DeployAppRequestContext, DeployServiceResult, DeployUploadSessionResponse, ListSitesQuery,
    UpdateSiteRequest, UploadCustomCertificateRequest, UPLOAD_PACKAGE_TYPE_TLS_CERTIFICATE,
    UPLOAD_PACKAGE_TYPE_TLS_PRIVATE_KEY, UPLOAD_SESSION_STATUS_CANCELLED,
    UPLOAD_SESSION_STATUS_COMPLETED,
};
use sdkwork_deploy_drive_port::{DriveRequestCredentials, PrepareDeployUploadCommand};

use crate::DeployService;

impl DeployService {
    fn require_tenant(context: &DeployAppRequestContext) -> DeployServiceResult<i64> {
        if context.tenant_id <= 0 {
            return Err(sdkwork_deploy_contract::DeployServiceError::Forbidden);
        }
        Ok(context.tenant_id)
    }

    async fn audit_site_action(
        &self,
        context: &DeployAppRequestContext,
        action: &str,
        target_uuid: &str,
    ) -> DeployServiceResult<()> {
        let operator_id = context.actor_id.unwrap_or(0);
        self.repository
            .insert_audit_log(
                context.tenant_id,
                context.organization_id.unwrap_or(0),
                operator_id,
                action,
                "site",
                None,
                Some(target_uuid),
            )
            .await
    }

    fn drive_credentials(context: &DeployAppRequestContext) -> DriveRequestCredentials {
        DriveRequestCredentials {
            auth_token: context.auth_token.clone(),
            access_token: context.access_token.clone(),
        }
    }

    fn validate_upload_request(
        request: &CreateDeployUploadSessionRequest,
    ) -> DeployServiceResult<()> {
        if request.file_name.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "fileName is required",
            ));
        }
        if request.idempotency_key.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "idempotencyKey is required",
            ));
        }
        if request.content_length <= 0 {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "contentLength must be positive",
            ));
        }
        if request.content_type.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "contentType is required",
            ));
        }
        if !(1..=7).contains(&request.package_type) {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "packageType must be between 1 and 7",
            ));
        }
        Ok(())
    }

    fn ensure_upload_session_mutable(
        stored: &DeployUploadSessionResponse,
    ) -> DeployServiceResult<()> {
        match stored.status {
            UPLOAD_SESSION_STATUS_COMPLETED => {
                Err(sdkwork_deploy_contract::DeployServiceError::conflict(
                    "upload session already completed",
                ))
            }
            UPLOAD_SESSION_STATUS_CANCELLED => {
                Err(sdkwork_deploy_contract::DeployServiceError::conflict(
                    "upload session already cancelled",
                ))
            }
            _ => Ok(()),
        }
    }

    fn ensure_completed_upload_for_certificate(
        session: &DeployUploadSessionResponse,
        expected_package_type: i32,
        label: &str,
    ) -> DeployServiceResult<()> {
        if session.package_type != expected_package_type {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                format!("{label} upload session has unexpected packageType"),
            ));
        }
        if session.status != UPLOAD_SESSION_STATUS_COMPLETED {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                format!("{label} upload session is not completed"),
            ));
        }
        if session.drive_node_id.as_deref().unwrap_or("").is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                format!("{label} upload session is missing drive node reference"),
            ));
        }
        Ok(())
    }

    fn validate_upload_certificate_request(
        request: &UploadCustomCertificateRequest,
    ) -> DeployServiceResult<()> {
        if request.cert_name.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "certName is required",
            ));
        }
        if request.certificate_upload_session_id.trim().is_empty()
            || request.private_key_upload_session_id.trim().is_empty()
        {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "certificateUploadSessionId and privateKeyUploadSessionId are required",
            ));
        }
        if request.certificate_upload_session_id == request.private_key_upload_session_id {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "certificate and private key upload sessions must differ",
            ));
        }
        if request.idempotency_key.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "idempotencyKey is required",
            ));
        }
        Ok(())
    }

    fn upload_session_request_matches_stored(
        request: &CreateDeployUploadSessionRequest,
        stored: &DeployUploadSessionResponse,
    ) -> bool {
        stored.site_id == request.site_id
            && stored.package_type == request.package_type
            && stored.file_name == request.file_name
            && stored.content_type == request.content_type
            && stored.content_length == request.content_length
            && stored.checksum == request.checksum
    }
}

#[async_trait]
impl DeployAppApi for DeployService {
    async fn list_sites(
        &self,
        context: &DeployAppRequestContext,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SitePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.list_sites(tenant_id, query).await
    }

    async fn create_site(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .create_site(
                tenant_id,
                context.organization_id,
                context.actor_id,
                request,
            )
            .await?;
        let _ = self
            .audit_site_action(context, "sites.create", &site.id)
            .await;
        Ok(site)
    }

    async fn retrieve_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.retrieve_site(tenant_id, site_id).await
    }

    async fn update_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .update_site(tenant_id, site_id, request)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.update", site_id)
            .await;
        Ok(site)
    }

    async fn delete_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_site(tenant_id, site_id, context.actor_id)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.delete", site_id)
            .await;
        Ok(())
    }

    async fn activate_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .set_site_status(tenant_id, site_id, 1)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.activate", site_id)
            .await;
        Ok(site)
    }

    async fn pause_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let site = self
            .repository
            .set_site_status(tenant_id, site_id, 2)
            .await?;
        let _ = self
            .audit_site_action(context, "sites.pause", site_id)
            .await;
        Ok(site)
    }

    async fn list_domains(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_domains(tenant_id, site_id, page, page_size)
            .await
    }

    async fn create_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_domain(tenant_id, site_id, request)
            .await
    }

    async fn retrieve_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn delete_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn verify_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainVerifyResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .verify_domain(tenant_id, site_id, domain_id)
            .await
    }

    async fn list_deployments(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_deployments(tenant_id, site_id, page, page_size, status)
            .await
    }

    async fn create_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_deployment(tenant_id, site_id, context.actor_id, request)
            .await
    }

    async fn retrieve_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_deployment(tenant_id, site_id, deployment_id)
            .await
    }

    async fn rollback_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .rollback_deployment(tenant_id, site_id, deployment_id, context.actor_id)
            .await
    }

    async fn list_artifacts(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ArtifactPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_artifacts(tenant_id, page, page_size)
            .await
    }

    async fn retrieve_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ArtifactResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_artifact(tenant_id, artifact_id)
            .await
    }

    async fn retain_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retain_artifact(tenant_id, artifact_id)
            .await
    }

    async fn list_releases(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleasePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_releases(tenant_id, site_id, page, page_size)
            .await
    }

    async fn retrieve_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleaseResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_release(tenant_id, site_id, release_id)
            .await
    }

    async fn create_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateReleaseRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleaseResponse> {
        let tenant_id = Self::require_tenant(context)?;
        if request.artifact_id.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "artifactId is required",
            ));
        }
        if request.idempotency_key.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "idempotencyKey is required",
            ));
        }
        if let Some(existing) = self
            .repository
            .find_release_by_idempotency_key(tenant_id, site_id, &request.idempotency_key)
            .await?
        {
            return Ok(existing);
        }
        match self
            .repository
            .create_release(tenant_id, site_id, request)
            .await
        {
            Ok(response) => Ok(response),
            Err(error @ sdkwork_deploy_contract::DeployServiceError::Conflict(_)) => {
                if let Some(existing) = self
                    .repository
                    .find_release_by_idempotency_key(tenant_id, site_id, &request.idempotency_key)
                    .await?
                {
                    return Ok(existing);
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    async fn list_env_variables(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::EnvVariablePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_env_variables(tenant_id, site_id, environment)
            .await
    }

    async fn create_env_variable(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::EnvVariableResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_env_variable(tenant_id, site_id, request)
            .await
    }

    async fn list_certificates(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificatePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_certificates(tenant_id, page, page_size)
            .await
    }

    async fn create_certificate(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.create_certificate(tenant_id, request).await
    }

    async fn upload_custom_certificate(
        &self,
        context: &DeployAppRequestContext,
        request: &UploadCustomCertificateRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        let tenant_id = Self::require_tenant(context)?;
        Self::validate_upload_certificate_request(request)?;
        if let Some(existing) = self
            .repository
            .find_certificate_by_idempotency_key(tenant_id, &request.idempotency_key)
            .await?
        {
            return Ok(existing);
        }
        let certificate_upload = self
            .repository
            .retrieve_upload_session_ref(tenant_id, &request.certificate_upload_session_id)
            .await?;
        let private_key_upload = self
            .repository
            .retrieve_upload_session_ref(tenant_id, &request.private_key_upload_session_id)
            .await?;
        Self::ensure_completed_upload_for_certificate(
            &certificate_upload,
            UPLOAD_PACKAGE_TYPE_TLS_CERTIFICATE,
            "certificate",
        )?;
        Self::ensure_completed_upload_for_certificate(
            &private_key_upload,
            UPLOAD_PACKAGE_TYPE_TLS_PRIVATE_KEY,
            "private key",
        )?;
        match self
            .repository
            .upload_custom_certificate(tenant_id, request, &certificate_upload, &private_key_upload)
            .await
        {
            Ok(response) => Ok(response),
            Err(error @ sdkwork_deploy_contract::DeployServiceError::Conflict(_)) => {
                if let Some(existing) = self
                    .repository
                    .find_certificate_by_idempotency_key(tenant_id, &request.idempotency_key)
                    .await?
                {
                    return Ok(existing);
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    async fn retrieve_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_certificate(tenant_id, certificate_id)
            .await
    }

    async fn delete_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_certificate(tenant_id, certificate_id)
            .await
    }

    async fn renew_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .renew_certificate(tenant_id, certificate_id)
            .await
    }

    async fn list_health_checks(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::HealthCheckPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.list_health_checks(tenant_id, site_id).await
    }

    async fn create_health_check(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::HealthCheckResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_health_check(tenant_id, site_id, request)
            .await
    }

    async fn create_upload_session(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let tenant_id = Self::require_tenant(context)?;
        Self::validate_upload_request(request)?;
        if let Some(existing) = self
            .repository
            .find_upload_session_by_idempotency_key(tenant_id, &request.idempotency_key)
            .await?
        {
            if Self::upload_session_request_matches_stored(request, &existing) {
                return Ok(existing);
            }
            return Err(sdkwork_deploy_contract::DeployServiceError::conflict(
                "idempotencyKey already used with a different upload payload",
            ));
        }
        let drive_response = self
            .drive
            .prepare_package_upload(
                &Self::drive_credentials(context),
                PrepareDeployUploadCommand {
                    tenant_id,
                    organization_id: context.organization_id,
                    operator_id: context.actor_id,
                    request: request.clone(),
                },
            )
            .await?;
        match self
            .repository
            .create_upload_session_ref(tenant_id, context, request, &drive_response)
            .await
        {
            Ok(response) => Ok(response),
            Err(error @ sdkwork_deploy_contract::DeployServiceError::Conflict(_)) => {
                if let Some(existing) = self
                    .repository
                    .find_upload_session_by_idempotency_key(tenant_id, &request.idempotency_key)
                    .await?
                {
                    if Self::upload_session_request_matches_stored(request, &existing) {
                        return Ok(existing);
                    }
                }
                Err(error)
            }
            Err(error) => Err(error),
        }
    }

    async fn retrieve_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let stored = self
            .repository
            .retrieve_upload_session_ref(tenant_id, upload_session_id)
            .await?;
        let refreshed = self
            .drive
            .retrieve_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
            )
            .await?;
        self.repository
            .update_upload_session_status(
                tenant_id,
                upload_session_id,
                refreshed.status,
                refreshed.drive_node_id.as_deref(),
            )
            .await
    }

    async fn complete_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
        request: &CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let tenant_id = Self::require_tenant(context)?;
        if request.checksum_sha256_hex.trim().is_empty() {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "checksumSha256Hex is required",
            ));
        }
        let stored = self
            .repository
            .retrieve_upload_session_ref(tenant_id, upload_session_id)
            .await?;
        Self::ensure_upload_session_mutable(&stored)?;
        let drive_response = self
            .drive
            .complete_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
                request,
            )
            .await?;
        let updated = self
            .repository
            .update_upload_session_status(
                tenant_id,
                upload_session_id,
                drive_response.status,
                drive_response.drive_node_id.as_deref(),
            )
            .await?;
        if updated.status == UPLOAD_SESSION_STATUS_COMPLETED
            && is_deploy_package_artifact_type(updated.package_type)
        {
            self.repository
                .create_artifact_from_upload_session(
                    tenant_id,
                    upload_session_id,
                    &request.checksum_sha256_hex,
                )
                .await?;
        }
        Ok(updated)
    }

    async fn cancel_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
        request: &CancelDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let stored = self
            .repository
            .retrieve_upload_session_ref(tenant_id, upload_session_id)
            .await?;
        Self::ensure_upload_session_mutable(&stored)?;
        let mut cancel_request = request.clone();
        if cancel_request.operator_id.is_none() {
            cancel_request.operator_id = context.actor_id.map(|value| value.to_string());
        }
        let drive_response = self
            .drive
            .cancel_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
                &cancel_request,
            )
            .await?;
        self.repository
            .update_upload_session_status(
                tenant_id,
                upload_session_id,
                drive_response.status,
                drive_response.drive_node_id.as_deref(),
            )
            .await
    }
}

#[cfg(test)]
mod upload_session_tests {
    use super::*;
    use sdkwork_deploy_contract::CreateDeployUploadSessionRequest;

    fn sample_request() -> CreateDeployUploadSessionRequest {
        CreateDeployUploadSessionRequest {
            site_id: None,
            package_type: 1,
            file_name: "app.zip".to_string(),
            content_type: "application/zip".to_string(),
            content_length: 1024,
            checksum: None,
            idempotency_key: "idem-1".to_string(),
        }
    }

    #[test]
    fn validate_upload_request_rejects_invalid_package_type() {
        let mut request = sample_request();
        request.package_type = 9;
        assert!(DeployService::validate_upload_request(&request).is_err());
    }

    #[test]
    fn upload_session_request_matches_stored_compares_payload_fields() {
        let request = sample_request();
        let stored = DeployUploadSessionResponse {
            id: "sess-1".to_string(),
            site_id: None,
            package_type: 1,
            file_name: "app.zip".to_string(),
            content_type: "application/zip".to_string(),
            content_length: 1024,
            checksum: None,
            status: 0,
            drive_upload_session_id: "drive-1".to_string(),
            drive_upload_item_id: None,
            drive_space_id: None,
            drive_node_id: None,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        };
        assert!(DeployService::upload_session_request_matches_stored(
            &request, &stored
        ));
    }

    #[test]
    fn ensure_upload_session_mutable_rejects_terminal_states() {
        let completed = DeployUploadSessionResponse {
            id: "sess-1".to_string(),
            site_id: None,
            package_type: 1,
            file_name: "app.zip".to_string(),
            content_type: "application/zip".to_string(),
            content_length: 1024,
            checksum: None,
            status: UPLOAD_SESSION_STATUS_COMPLETED,
            drive_upload_session_id: "drive-1".to_string(),
            drive_upload_item_id: None,
            drive_space_id: None,
            drive_node_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert!(DeployService::ensure_upload_session_mutable(&completed).is_err());
    }
}
