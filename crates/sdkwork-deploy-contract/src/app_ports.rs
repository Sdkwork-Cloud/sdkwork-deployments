use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::dto::*;
use crate::problem::DeployServiceResult;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAppRequestContext {
    pub tenant_id: i64,
    pub actor_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployBackendRequestContext {
    pub operator_id: Option<i64>,
    pub tenant_id: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListSitesQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size", rename = "pageSize")]
    pub page_size: i32,
    pub status: Option<i32>,
    #[serde(rename = "siteType")]
    pub site_type: Option<i32>,
    pub keyword: Option<String>,
}

#[async_trait]
pub trait DeployAppApi: Send + Sync {
    async fn list_sites(
        &self,
        context: &DeployAppRequestContext,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage>;

    async fn create_site(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn retrieve_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn update_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn delete_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<()>;

    async fn activate_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn pause_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn list_domains(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainPage>;

    async fn create_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<DomainResponse>;

    async fn retrieve_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainResponse>;

    async fn delete_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()>;

    async fn verify_domain(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse>;

    async fn list_deployments(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<DeploymentPage>;

    async fn create_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn retrieve_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn rollback_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn list_env_variables(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<EnvVariablePage>;

    async fn create_env_variable(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<EnvVariableResponse>;

    async fn list_certificates(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage>;

    async fn create_certificate(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn list_health_checks(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage>;

    async fn create_health_check(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse>;
}

#[async_trait]
pub trait DeployBackendApi: Send + Sync {
    async fn list_nginx_configs(
        &self,
        context: &DeployBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<NginxConfigPage>;

    async fn create_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn retrieve_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn update_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn validate_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxValidateResponse>;

    async fn deploy_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn reload_nginx(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<NginxReloadResponse>;

    async fn retrieve_nginx_status(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<NginxStatusResponse>;

    async fn list_servers(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ServerPage>;

    async fn create_server(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn list_audit_logs(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AuditLogPage>;
}
