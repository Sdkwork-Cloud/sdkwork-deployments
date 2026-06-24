//! `DeployRepositoryPort` trait implementation delegating to SQLx repository modules.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    AuditLogPage, CertificatePage, CertificateResponse, CreateCertificateRequest,
    CreateDeploymentRequest, CreateDomainRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateNginxConfigRequest, CreateServerRequest, CreateSiteRequest,
    DeploymentPage, DeploymentResponse, DomainPage, DomainResponse, DomainVerifyResponse,
    EnvVariablePage, EnvVariableResponse, HealthCheckPage, HealthCheckResponse,
    ListNginxConfigsQuery, ListSitesQuery, NginxConfigPage, NginxConfigResponse,
    NginxReloadResponse, NginxStatusResponse, NginxValidateResponse, ServerPage, ServerResponse,
    SitePage, SiteResponse, UpdateNginxConfigRequest, UpdateSiteRequest,
};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

use crate::DeployRepository;

#[async_trait]
impl DeployRepositoryPort for DeployRepository {
    async fn ready_check(&self) -> DeployServiceResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|_| DeployServiceError::DatabaseUnavailable)?;
        Ok(())
    }

    async fn list_sites(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage> {
        self.list_sites_repo(tenant_id, query).await
    }

    async fn create_site(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<SiteResponse> {
        self.create_site_repo(tenant_id, organization_id, actor_id, request)
            .await
    }

    async fn retrieve_site(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse> {
        self.retrieve_site_repo(tenant_id, site_id).await
    }

    async fn update_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse> {
        self.update_site_repo(tenant_id, site_id, request).await
    }

    async fn delete_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<()> {
        self.delete_site_repo(tenant_id, site_id, actor_id).await
    }

    async fn set_site_status(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> DeployServiceResult<SiteResponse> {
        self.set_site_status_repo(tenant_id, site_id, status).await
    }

    async fn list_domains(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainPage> {
        self.list_domains_repo(tenant_id, site_id, page, page_size)
            .await
    }

    async fn create_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<DomainResponse> {
        self.create_domain_repo(tenant_id, site_id, request).await
    }

    async fn retrieve_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainResponse> {
        self.retrieve_domain_repo(tenant_id, site_id, domain_id)
            .await
    }

    async fn delete_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()> {
        self.delete_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn verify_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse> {
        self.verify_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<DeploymentPage> {
        self.list_deployments_repo(tenant_id, site_id, page, page_size, status)
            .await
    }

    async fn create_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.create_deployment_repo(tenant_id, site_id, actor_id, request)
            .await
    }

    async fn retrieve_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.retrieve_deployment_repo(tenant_id, site_id, deployment_id)
            .await
    }

    async fn rollback_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.rollback_deployment_repo(tenant_id, site_id, deployment_id, actor_id)
            .await
    }

    async fn list_env_variables(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<EnvVariablePage> {
        self.list_env_variables_repo(tenant_id, site_id, environment)
            .await
    }

    async fn create_env_variable(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<EnvVariableResponse> {
        self.create_env_variable_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_certificates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage> {
        self.list_certificates_repo(tenant_id, page, page_size)
            .await
    }

    async fn create_certificate(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse> {
        self.create_certificate_repo(tenant_id, request).await
    }

    async fn list_health_checks(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage> {
        self.list_health_checks_repo(tenant_id, site_id).await
    }

    async fn create_health_check(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse> {
        self.create_health_check_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_nginx_configs(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<NginxConfigPage> {
        self.list_nginx_configs_repo(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.create_nginx_config_repo(tenant_id, request).await
    }

    async fn retrieve_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.retrieve_nginx_config_repo(tenant_id, config_id).await
    }

    async fn update_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.update_nginx_config_repo(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxValidateResponse> {
        self.validate_nginx_config_repo(tenant_id, config_id).await
    }

    async fn deploy_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.deploy_nginx_config_repo(tenant_id, config_id).await
    }

    async fn reload_nginx(&self) -> DeployServiceResult<NginxReloadResponse> {
        self.reload_nginx_repo().await
    }

    async fn retrieve_nginx_status(
        &self,
        tenant_id: Option<i64>,
    ) -> DeployServiceResult<NginxStatusResponse> {
        self.retrieve_nginx_status_repo(tenant_id).await
    }

    async fn list_servers(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ServerPage> {
        self.list_servers_repo(tenant_id, page, page_size).await
    }

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        self.create_server_repo(tenant_id, request).await
    }

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AuditLogPage> {
        self.list_audit_logs_repo(tenant_id, page, page_size).await
    }

    async fn insert_audit_log(
        &self,
        tenant_id: i64,
        organization_id: i64,
        operator_id: i64,
        action: &str,
        target_type: &str,
        target_id: Option<i64>,
        target_uuid: Option<&str>,
    ) -> DeployServiceResult<()> {
        self.insert_audit_log_repo(
            tenant_id,
            organization_id,
            operator_id,
            action,
            target_type,
            target_id,
            target_uuid,
        )
        .await
    }
}
