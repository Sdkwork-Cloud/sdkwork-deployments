//! Repository port consumed by the deploy service layer.

use async_trait::async_trait;
use sdkwork_deploy_contract::DeployServiceResult;
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

#[async_trait]
pub trait DeployRepositoryPort: Send + Sync {
    async fn ready_check(&self) -> DeployServiceResult<()>;

    async fn list_sites(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage>;

    async fn create_site(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn retrieve_site(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn update_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn delete_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<()>;

    async fn set_site_status(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> DeployServiceResult<SiteResponse>;

    async fn list_domains(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainPage>;

    async fn create_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<DomainResponse>;

    async fn retrieve_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainResponse>;

    async fn delete_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()>;

    async fn verify_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse>;

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<DeploymentPage>;

    async fn create_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn retrieve_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn rollback_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn list_env_variables(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<EnvVariablePage>;

    async fn create_env_variable(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<EnvVariableResponse>;

    async fn list_certificates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage>;

    async fn create_certificate(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn list_health_checks(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage>;

    async fn create_health_check(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse>;

    async fn list_nginx_configs(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<NginxConfigPage>;

    async fn create_nginx_config(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn retrieve_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn update_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn validate_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxValidateResponse>;

    async fn deploy_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn reload_nginx(&self) -> DeployServiceResult<NginxReloadResponse>;

    async fn retrieve_nginx_status(
        &self,
        tenant_id: Option<i64>,
    ) -> DeployServiceResult<NginxStatusResponse>;

    async fn list_servers(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ServerPage>;

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AuditLogPage>;

    async fn insert_audit_log(
        &self,
        tenant_id: i64,
        organization_id: i64,
        operator_id: i64,
        action: &str,
        target_type: &str,
        target_id: Option<i64>,
        target_uuid: Option<&str>,
    ) -> DeployServiceResult<()>;
}
