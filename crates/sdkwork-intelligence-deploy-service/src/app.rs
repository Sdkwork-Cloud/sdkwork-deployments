//! App-api service surface implementation.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    is_deploy_package_artifact_type, AppDeploymentPage, AppDeploymentResponse, AppPage,
    AppReleasePage, AppReleaseResponse, AppResponse, BuildPage, BuildResponse, BuildTemplatePage,
    BuildTemplateResponse, ChannelPage, ChannelResponse, ChannelRolloutPage,
    ChannelRolloutResponse, CompleteDeployUploadSessionRequest, CreateAppDeploymentRequest,
    CreateAppReleaseRequest, CreateAppRequest, CreateArtifactRequest, CreateBuildRequest,
    CreateBuildTemplateRequest, CreateCertificateRequest, CreateDeployUploadSessionRequest,
    CreateDeploymentRequest, CreateDomainHostnameRequest, CreateDomainZoneRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreatePlatformTargetRequest,
    CreateReleaseRequest, CreateSigningIdentityRequest, CreateSiteRequest,
    CreateSourceRepositoryRequest, DeployAppApi, DeployAppRequestContext, DeployServiceResult,
    DeployUploadSessionResponse, ListDomainZonesQuery, ListSitesQuery, PackagePage,
    PackageResponse, PlatformTargetPage, PlatformTargetResponse, PromoteChannelRequest,
    RegisterPackageRequest, ReleaseStatus, SigningIdentityPage, SigningIdentityResponse,
    SourceRepositoryPage, SourceRepositoryResponse, UpdateAppRequest, UpdateBuildStateRequest,
    UpdateDomainHostnameRequest, UpdateDomainZoneRequest, UpdateSiteRequest,
    UPLOAD_SESSION_STATUS_CANCELLED, UPLOAD_SESSION_STATUS_COMPLETED,
};
use sdkwork_deploy_drive_port::{DriveRequestCredentials, PrepareDeployUploadCommand};

use crate::{repository::InsertAuditLogCommand, DeployService};

impl DeployService {
    pub(crate) fn require_tenant(context: &DeployAppRequestContext) -> DeployServiceResult<i64> {
        if context.tenant_id <= 0 {
            return Err(sdkwork_deploy_contract::DeployServiceError::forbidden(
                "site operations require tenant authorization",
            ));
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
            .insert_audit_log(InsertAuditLogCommand {
                tenant_id: context.tenant_id,
                organization_id: context.organization_id.unwrap_or(0),
                operator_id,
                action: action.to_owned(),
                target_type: "site".to_owned(),
                target_id: None,
                target_uuid: Some(target_uuid.to_owned()),
            })
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
        if !is_deploy_package_artifact_type(request.package_type) {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "packageType must be a deployable artifact type between 1 and 5",
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

fn normalize_optional_text(
    value: Option<String>,
    maximum_length: usize,
    field: &str,
) -> DeployServiceResult<Option<String>> {
    value
        .map(|value| {
            let value = value.trim().to_owned();
            if value.is_empty() || value.len() > maximum_length {
                return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                    format!("{field} must contain 1 to {maximum_length} characters"),
                ));
            }
            Ok(value)
        })
        .transpose()
}

fn normalize_relative_hostname(
    relative_name: &str,
    apex_hostname: &str,
) -> DeployServiceResult<String> {
    let relative_name = relative_name.trim();
    if relative_name == "@" {
        return Ok("@".to_owned());
    }
    if relative_name.is_empty() || relative_name.ends_with('.') {
        return Err(sdkwork_deploy_contract::DeployServiceError::validation(
            "relativeName is invalid",
        ));
    }
    let hostname = crate::normalize_domain_hostname(&format!("{relative_name}.{apex_hostname}"))?;
    let suffix = format!(".{apex_hostname}");
    hostname
        .strip_suffix(&suffix)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            sdkwork_deploy_contract::DeployServiceError::validation(
                "relativeName must remain inside the selected domain zone",
            )
        })
}

#[async_trait]
impl DeployAppApi for DeployService {
    async fn list_domain_zones(
        &self,
        context: &DeployAppRequestContext,
        query: &ListDomainZonesQuery,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainZonePage> {
        let tenant_id = Self::require_tenant(context)?;
        if let Some(status) = query.status.as_deref() {
            if !matches!(status, "ACTIVE" | "PAUSED") {
                return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                    "domain zone status is invalid",
                ));
            }
        }
        self.repository.list_domain_zones(tenant_id, query).await
    }

    async fn create_domain_zone(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateDomainZoneRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainZoneResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let mut request = request.clone();
        request.apex_hostname = crate::normalize_zone_apex(&request.apex_hostname)?;
        request.display_name = normalize_optional_text(request.display_name, 200, "displayName")?;
        request.dns_provider = normalize_optional_text(request.dns_provider, 64, "dnsProvider")?;
        request.provider_zone_ref =
            normalize_optional_text(request.provider_zone_ref, 512, "providerZoneRef")?;
        self.repository
            .create_domain_zone(
                tenant_id,
                context.organization_id,
                context.actor_id,
                &request,
            )
            .await
    }

    async fn retrieve_domain_zone(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainZoneResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_domain_zone(tenant_id, zone_id)
            .await
    }

    async fn update_domain_zone(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        request: &UpdateDomainZoneRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainZoneResponse> {
        let tenant_id = Self::require_tenant(context)?;
        if let Some(status) = request.status.as_deref() {
            if !matches!(status, "ACTIVE" | "PAUSED") {
                return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                    "domain zone status is invalid",
                ));
            }
        }
        let mut request = request.clone();
        request.display_name = normalize_optional_text(request.display_name, 200, "displayName")?;
        request.dns_provider = normalize_optional_text(request.dns_provider, 64, "dnsProvider")?;
        request.provider_zone_ref =
            normalize_optional_text(request.provider_zone_ref, 512, "providerZoneRef")?;
        self.repository
            .update_domain_zone(tenant_id, context.actor_id, zone_id, &request)
            .await
    }

    async fn delete_domain_zone(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository.delete_domain_zone(tenant_id, zone_id).await
    }

    async fn list_domain_hostnames(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainHostnamePage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_domain_hostnames(tenant_id, zone_id, page, page_size)
            .await
    }

    async fn create_domain_hostname(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        request: &CreateDomainHostnameRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainHostnameResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let zone = self
            .repository
            .retrieve_domain_zone(tenant_id, zone_id)
            .await?;
        let relative_name =
            normalize_relative_hostname(&request.relative_name, &zone.apex_hostname)?;
        self.repository
            .create_domain_hostname(
                tenant_id,
                context.actor_id,
                zone_id,
                &CreateDomainHostnameRequest { relative_name },
            )
            .await
    }

    async fn retrieve_domain_hostname(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainHostnameResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .retrieve_domain_hostname(tenant_id, zone_id, hostname_id)
            .await
    }

    async fn delete_domain_hostname(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<()> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .delete_domain_hostname(tenant_id, zone_id, hostname_id)
            .await
    }

    async fn update_domain_hostname(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        hostname_id: &str,
        request: &UpdateDomainHostnameRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainHostnameResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let zone = self
            .repository
            .retrieve_domain_zone(tenant_id, zone_id)
            .await?;
        let relative_name =
            normalize_relative_hostname(&request.relative_name, &zone.apex_hostname)?;
        self.repository
            .update_domain_hostname(
                tenant_id,
                context.actor_id,
                zone_id,
                hostname_id,
                &UpdateDomainHostnameRequest { relative_name },
            )
            .await
    }

    async fn verify_domain_hostname(
        &self,
        context: &DeployAppRequestContext,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainVerifyResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let challenge = self
            .repository
            .domain_hostname_verification_challenge(tenant_id, zone_id, hostname_id)
            .await?;
        if challenge.verified || challenge.token.is_some() {
            return Ok(challenge.response());
        }
        let verification_id = challenge.verification_id.as_deref().ok_or_else(|| {
            sdkwork_deploy_contract::DeployServiceError::Internal(
                "pending domain has no verification attempt".to_owned(),
            )
        })?;
        let proof_sha256 = challenge.proof_sha256.as_deref().ok_or_else(|| {
            sdkwork_deploy_contract::DeployServiceError::Internal(
                "pending domain verification has no proof digest".to_owned(),
            )
        })?;
        let observation = self
            .domain_ownership_verifier
            .verify_dns_txt(&challenge.hostname, proof_sha256)
            .await?;
        if !observation.matched {
            return Ok(challenge.response());
        }
        let observed_sha256 = observation.observed_sha256.as_deref().ok_or_else(|| {
            sdkwork_deploy_contract::DeployServiceError::Internal(
                "matched domain verification has no observed digest".to_owned(),
            )
        })?;
        if self
            .repository
            .confirm_domain_hostname_verification(
                tenant_id,
                zone_id,
                hostname_id,
                verification_id,
                observed_sha256,
                &observation.verifier_identity,
            )
            .await?
        {
            return Ok(sdkwork_deploy_contract::DomainVerifyResponse {
                verified: true,
                method: crate::domain_verification::DOMAIN_VERIFICATION_METHOD_DNS_TXT.to_owned(),
                verification_id: Some(verification_id.to_owned()),
                record_name: challenge.record_name,
                token: None,
                expires_at: challenge.expires_at,
            });
        }
        let current = self
            .repository
            .domain_hostname_verification_challenge(tenant_id, zone_id, hostname_id)
            .await?;
        if current.verified {
            Ok(current.response())
        } else {
            Err(sdkwork_deploy_contract::DeployServiceError::conflict(
                "domain verification challenge changed; retry with the current token",
            ))
        }
    }

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

    async fn update_site_composition(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        expected_site_version: i64,
        idempotency_key: &str,
        request: &sdkwork_deploy_contract::UpdateSiteCompositionRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteCompositionResponse> {
        self.update_composition(
            context,
            site_id,
            expected_site_version,
            idempotency_key,
            request,
        )
        .await
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

    async fn list_deployments(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
        cursor: Option<&str>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentPage> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .list_deployments(tenant_id, site_id, page, page_size, status, cursor)
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

    async fn create_artifact(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateArtifactRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ArtifactResponse> {
        let tenant_id = Self::require_tenant(context)?;
        if !is_deploy_package_artifact_type(request.package_type) {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "packageType must identify a deployable package",
            ));
        }
        if request.file_name.trim().is_empty()
            || request.content_type.trim().is_empty()
            || request.content_length <= 0
            || request.drive_upload_session_id.trim().is_empty()
            || request.drive_space_id.trim().is_empty()
            || request.drive_node_id.trim().is_empty()
            || request.idempotency_key.trim().is_empty()
        {
            return Err(sdkwork_deploy_contract::DeployServiceError::validation(
                "file metadata, stable Drive references, and idempotencyKey are required",
            ));
        }
        self.repository
            .create_artifact_from_drive(tenant_id, request)
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
        idempotency_key: &str,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        let tenant_id = Self::require_tenant(context)?;
        self.repository
            .create_certificate(
                tenant_id,
                context.organization_id,
                context.actor_id,
                idempotency_key,
                request,
            )
            .await
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
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        let tenant_id = Self::require_tenant(context)?;
        let stored = self
            .repository
            .retrieve_upload_session_ref(tenant_id, upload_session_id)
            .await?;
        Self::ensure_upload_session_mutable(&stored)?;
        let drive_response = self
            .drive
            .cancel_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
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

    // -- unified app delivery (REQ-2026-0002) ------------------------------------

    async fn list_apps(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppPage> {
        self.list_apps(context, page, page_size).await
    }

    async fn create_app(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        self.create_app(context, request).await
    }

    async fn retrieve_app(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<AppResponse> {
        self.retrieve_app(context, app_id).await
    }

    async fn update_app(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &UpdateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        self.update_app(context, app_id, request).await
    }

    async fn create_platform_target(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &CreatePlatformTargetRequest,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        self.create_platform_target(context, app_id, request).await
    }

    async fn list_platform_targets(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<PlatformTargetPage> {
        self.list_platform_targets(context, app_id).await
    }

    async fn retrieve_platform_target(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        target_id: &str,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        self.retrieve_platform_target(context, app_id, target_id)
            .await
    }

    async fn create_source_repository(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &CreateSourceRepositoryRequest,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        self.create_source_repository(context, app_id, request)
            .await
    }

    async fn list_source_repositories(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<SourceRepositoryPage> {
        self.list_source_repositories(context, app_id).await
    }

    async fn retrieve_source_repository(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        repo_id: &str,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        self.retrieve_source_repository(context, app_id, repo_id)
            .await
    }

    async fn create_build_template(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateBuildTemplateRequest,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        self.create_build_template(context, request).await
    }

    async fn list_build_templates(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildTemplatePage> {
        self.list_build_templates(context, page, page_size).await
    }

    async fn retrieve_build_template(
        &self,
        context: &DeployAppRequestContext,
        template_id: &str,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        self.retrieve_build_template(context, template_id).await
    }

    async fn create_build(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateBuildRequest,
    ) -> DeployServiceResult<BuildResponse> {
        self.create_build(context, request).await
    }

    async fn list_builds(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildPage> {
        self.list_builds(context, app_id, page, page_size).await
    }

    async fn retrieve_build(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        build_id: &str,
    ) -> DeployServiceResult<BuildResponse> {
        self.retrieve_build(context, app_id, build_id).await
    }

    async fn update_build_state(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        build_id: &str,
        request: &UpdateBuildStateRequest,
    ) -> DeployServiceResult<BuildResponse> {
        self.update_build_state(context, app_id, build_id, request)
            .await
    }

    async fn register_package(
        &self,
        context: &DeployAppRequestContext,
        request: &RegisterPackageRequest,
    ) -> DeployServiceResult<PackageResponse> {
        self.register_package(context, request).await
    }

    async fn list_packages(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<PackagePage> {
        self.list_packages(context, app_id, page, page_size).await
    }

    async fn retrieve_package(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        package_id: &str,
    ) -> DeployServiceResult<PackageResponse> {
        self.retrieve_package(context, app_id, package_id).await
    }

    async fn create_app_release(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppReleaseRequest,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.create_app_release(context, request).await
    }

    async fn list_app_releases(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppReleasePage> {
        self.list_app_releases(context, app_id, page, page_size)
            .await
    }

    async fn retrieve_app_release(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.retrieve_app_release(context, app_id, release_id).await
    }

    async fn update_app_release_status(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        release_id: &str,
        release_status: ReleaseStatus,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.update_app_release_status(context, app_id, release_id, release_status)
            .await
    }

    async fn list_channels(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<ChannelPage> {
        self.list_channels(context, app_id).await
    }

    async fn retrieve_channel(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
    ) -> DeployServiceResult<ChannelResponse> {
        self.retrieve_channel(context, app_id, channel_id).await
    }

    async fn promote_channel(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
        request: &PromoteChannelRequest,
    ) -> DeployServiceResult<ChannelRolloutResponse> {
        self.promote_channel(context, app_id, channel_id, request)
            .await
    }

    async fn list_channel_rollouts(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ChannelRolloutPage> {
        self.list_channel_rollouts(context, app_id, channel_id, page, page_size)
            .await
    }

    async fn create_app_deployment(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppDeploymentRequest,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        self.create_app_deployment(context, request).await
    }

    async fn list_app_deployments(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppDeploymentPage> {
        self.list_app_deployments(context, app_id, page, page_size)
            .await
    }

    async fn retrieve_app_deployment(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        self.retrieve_app_deployment(context, app_id, deployment_id)
            .await
    }

    async fn create_signing_identity(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateSigningIdentityRequest,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        self.create_signing_identity(context, request).await
    }

    async fn list_signing_identities(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SigningIdentityPage> {
        self.list_signing_identities(context, page, page_size).await
    }

    async fn retrieve_signing_identity(
        &self,
        context: &DeployAppRequestContext,
        identity_id: &str,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        self.retrieve_signing_identity(context, identity_id).await
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

#[cfg(test)]
mod domain_zone_tests {
    use super::normalize_relative_hostname;

    #[test]
    fn relative_names_support_apex_multi_level_and_wildcard() {
        assert_eq!(
            normalize_relative_hostname("@", "example.com").unwrap(),
            "@"
        );
        assert_eq!(
            normalize_relative_hostname("WWW", "example.com").unwrap(),
            "www"
        );
        assert_eq!(
            normalize_relative_hostname("api.eu", "example.com").unwrap(),
            "api.eu"
        );
        assert_eq!(
            normalize_relative_hostname("*", "example.com").unwrap(),
            "*"
        );
        assert_eq!(
            normalize_relative_hostname("*.a", "example.com").unwrap(),
            "*.a"
        );
    }

    #[test]
    fn relative_names_stay_inside_the_zone() {
        for name in [
            "", ".", "@.x", "a..b", "a.b.", "foo.*", "*x", "*.a.*", "*.x..y",
        ] {
            assert!(
                normalize_relative_hostname(name, "example.com").is_err(),
                "{name}"
            );
        }
    }
}

// -- unified app delivery (REQ-2026-0002) ------------------------------------

impl DeployService {
    pub async fn list_apps_api(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppPage> {
        self.list_apps(context, page, page_size).await
    }
}
