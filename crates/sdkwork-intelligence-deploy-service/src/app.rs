//! App-api service surface implementation.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    CancelDeployUploadSessionRequest, CompleteDeployUploadSessionRequest, CreateCertificateRequest,
    CreateDeployUploadSessionRequest, CreateDeploymentRequest, CreateDomainRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreateSiteRequest, DeployAppApi,
    DeployAppRequestContext, DeployServiceResult, DeployUploadSessionResponse, ListSitesQuery,
    UpdateSiteRequest,
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
        Ok(())
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
        self.repository
            .create_upload_session_ref(tenant_id, context, request, &drive_response)
            .await
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
        self.drive
            .complete_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
                request,
            )
            .await?;
        self.repository
            .update_upload_session_status(tenant_id, upload_session_id, 1, None)
            .await
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
        let mut cancel_request = request.clone();
        if cancel_request.operator_id.is_none() {
            cancel_request.operator_id = context.actor_id.map(|value| value.to_string());
        }
        self.drive
            .cancel_upload_session(
                &Self::drive_credentials(context),
                &stored.drive_upload_session_id,
                &cancel_request,
            )
            .await?;
        self.repository
            .update_upload_session_status(tenant_id, upload_session_id, 2, None)
            .await
    }
}
