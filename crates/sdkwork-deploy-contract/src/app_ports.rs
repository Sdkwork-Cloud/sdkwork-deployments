use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::dto::*;
use crate::problem::DeployServiceResult;
use crate::site_composition::{SiteCompositionResponse, UpdateSiteCompositionRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAppRequestContext {
    pub tenant_id: i64,
    pub actor_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub session_id: Option<String>,
    #[serde(default, skip_serializing)]
    pub auth_token: Option<String>,
    #[serde(default, skip_serializing)]
    pub access_token: Option<String>,
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
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    pub status: Option<i32>,
    #[serde(rename = "siteType")]
    pub site_type: Option<i32>,
    pub keyword: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListDomainZonesQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[async_trait]
pub trait DeployAppApi: Send + Sync {
    async fn list_domain_zones(
        &self,
        _context: &DeployAppRequestContext,
        _query: &ListDomainZonesQuery,
    ) -> DeployServiceResult<DomainZonePage> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn create_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn update_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _request: &UpdateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn delete_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
    ) -> DeployServiceResult<()> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn list_domain_hostnames(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<DomainHostnamePage> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn create_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _request: &CreateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn delete_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<()> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn verify_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname verification API is not implemented".to_owned(),
        ))
    }

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

    async fn update_site_composition(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        expected_site_version: i64,
        idempotency_key: &str,
        request: &UpdateSiteCompositionRequest,
    ) -> DeployServiceResult<SiteCompositionResponse>;

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

    async fn list_artifacts(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ArtifactPage>;

    async fn create_artifact(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateArtifactRequest,
    ) -> DeployServiceResult<ArtifactResponse>;

    async fn retrieve_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<ArtifactResponse>;

    async fn retain_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<()>;

    async fn list_releases(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ReleasePage>;

    async fn retrieve_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<ReleaseResponse>;

    async fn create_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateReleaseRequest,
    ) -> DeployServiceResult<ReleaseResponse>;

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
        idempotency_key: &str,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn retrieve_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn delete_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<()>;

    async fn renew_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
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

    async fn create_upload_session(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn retrieve_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn complete_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
        request: &CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn cancel_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;
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
        cluster_id: Option<String>,
    ) -> DeployServiceResult<ServerPage>;

    async fn create_server(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn update_server(
        &self,
        context: &DeployBackendRequestContext,
        server_id: &str,
        request: &UpdateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn list_node_clusters(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<NodeClusterPage>;

    async fn create_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse>;

    async fn update_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        cluster_id: &str,
        request: &UpdateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse>;

    async fn list_audit_logs(
        &self,
        context: &DeployBackendRequestContext,
        query: &AuditLogQuery,
    ) -> DeployServiceResult<AuditLogPage>;
}
