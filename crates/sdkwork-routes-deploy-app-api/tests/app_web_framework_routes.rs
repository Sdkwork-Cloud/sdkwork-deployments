use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_deploy_contract::{
    DeployAppApi, DeployAppRequestContext, DeployServiceResult, ListSitesQuery, SitePage,
};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_routes_deploy_app_api::{
    build_router_with_shared_app_api, web_bootstrap::wrap_router_with_iam_database_web_framework,
};
use std::sync::Arc;
use tower::util::ServiceExt;

#[tokio::test]
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_iam_database_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_shared_app_api(Arc::new(StubAppApi)),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/sites")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

struct StubAppApi;

#[async_trait]
impl DeployAppApi for StubAppApi {
    async fn list_sites(
        &self,
        _context: &DeployAppRequestContext,
        _query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage> {
        Ok(SitePage::default())
    }

    async fn create_site(
        &self,
        _context: &DeployAppRequestContext,
        _request: &sdkwork_deploy_contract::CreateSiteRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn update_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::UpdateSiteRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn delete_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<()> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn activate_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn pause_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_domains(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainPage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_domain(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::CreateDomainRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_domain(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn delete_domain(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> DeployServiceResult<()> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn verify_domain(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _domain_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DomainVerifyResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_deployments(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _page: i32,
        _page_size: i32,
        _status: Option<i32>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentPage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_deployment(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::CreateDeploymentRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_deployment(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _deployment_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn rollback_deployment(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _deployment_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeploymentResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_env_variables(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _environment: Option<&str>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::EnvVariablePage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_env_variable(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::CreateEnvVariableRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::EnvVariableResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_certificates(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificatePage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_certificate(
        &self,
        _context: &DeployAppRequestContext,
        _request: &sdkwork_deploy_contract::CreateCertificateRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_health_checks(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::HealthCheckPage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_health_check(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::CreateHealthCheckRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::HealthCheckResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }
}
