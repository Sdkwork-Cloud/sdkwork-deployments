use async_trait::async_trait;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use sdkwork_deploy_contract::{
    ArtifactResponse, CreateArtifactRequest, DeployAppApi, DeployAppRequestContext,
    DeployServiceResult, ListSitesQuery, SitePage, SiteResponse,
};
use sdkwork_routes_deploy_app_api::{
    build_router_with_shared_app_api, web_bootstrap::wrap_router_with_web_framework,
};
use sdkwork_web_core::{
    access_token_jwt, auth_token_jwt_with_permissions, DefaultWebRequestContextResolver,
};
use std::sync::Arc;
use tower::util::ServiceExt;

fn authorized_request(path: &str, permission: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(
            header::AUTHORIZATION,
            format!(
                "Bearer {}",
                auth_token_jwt_with_permissions("42", "7", "session-1", "web", permission,)
            ),
        )
        .header(
            "access-token",
            access_token_jwt("42", "7", "session-1", "web"),
        )
        .body(Body::empty())
        .unwrap()
}

/// API_ASSEMBLY_SPEC §4: when a host selects a dependency assembly, tests must
/// prove that a matched dependency error carries the dependency operation's
/// `instance` and `operationId` (an HTTP status alone is not integration
/// evidence). The Deployments blocks keep that contract under the Web
/// Framework layer that also serves them when composed into the Web Server
/// standalone gateway.
#[tokio::test]
async fn matched_dependency_error_preserves_operation_identity() {
    let app = wrap_router_with_web_framework(
        DefaultWebRequestContextResolver::default(),
        build_router_with_shared_app_api(Arc::new(StubAppApi)),
    );

    let response = app
        .oneshot(authorized_request(
            "/app/v3/api/domain_zones",
            "deploy.domainZones.read",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["instance"], "GET /app/v3/api/domain_zones");
    assert_eq!(payload["operationId"], "domainZones.list");
}

#[tokio::test]
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_web_framework(
        DefaultWebRequestContextResolver::default(),
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
    async fn create_artifact(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateArtifactRequest,
    ) -> DeployServiceResult<ArtifactResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

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
    ) -> DeployServiceResult<SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn update_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn update_site_composition(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _expected_site_version: i64,
        _idempotency_key: &str,
        _request: &sdkwork_deploy_contract::UpdateSiteCompositionRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::SiteCompositionResponse> {
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
    ) -> DeployServiceResult<SiteResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn pause_site(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
    ) -> DeployServiceResult<SiteResponse> {
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
        _cursor: Option<&str>,
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

    async fn list_artifacts(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ArtifactPage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_artifact(
        &self,
        _context: &DeployAppRequestContext,
        _artifact_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ArtifactResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retain_artifact(
        &self,
        _context: &DeployAppRequestContext,
        _artifact_id: &str,
    ) -> DeployServiceResult<()> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn list_releases(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleasePage> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_release(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _release_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleaseResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn create_release(
        &self,
        _context: &DeployAppRequestContext,
        _site_id: &str,
        _request: &sdkwork_deploy_contract::CreateReleaseRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ReleaseResponse> {
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
        _idempotency_key: &str,
        _request: &sdkwork_deploy_contract::CreateCertificateRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_certificate(
        &self,
        _context: &DeployAppRequestContext,
        _certificate_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::CertificateResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn delete_certificate(
        &self,
        _context: &DeployAppRequestContext,
        _certificate_id: &str,
    ) -> DeployServiceResult<()> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn renew_certificate(
        &self,
        _context: &DeployAppRequestContext,
        _certificate_id: &str,
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

    async fn create_upload_session(
        &self,
        _context: &DeployAppRequestContext,
        _request: &sdkwork_deploy_contract::CreateDeployUploadSessionRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeployUploadSessionResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn retrieve_upload_session(
        &self,
        _context: &DeployAppRequestContext,
        _upload_session_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeployUploadSessionResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn complete_upload_session(
        &self,
        _context: &DeployAppRequestContext,
        _upload_session_id: &str,
        _request: &sdkwork_deploy_contract::CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeployUploadSessionResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }

    async fn cancel_upload_session(
        &self,
        _context: &DeployAppRequestContext,
        _upload_session_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::DeployUploadSessionResponse> {
        Err(sdkwork_deploy_contract::DeployServiceError::Internal(
            "not implemented".into(),
        ))
    }
}
