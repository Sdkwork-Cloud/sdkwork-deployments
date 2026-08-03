use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post, put},
    Extension, Json, Router,
};
use sdkwork_deploy_contract::{
    CompleteDeployUploadSessionRequest, CreateArtifactRequest, CreateCertificateRequest,
    CreateDeployUploadSessionRequest, CreateDeploymentRequest, CreateDomainHostnameRequest,
    CreateDomainZoneRequest, CreateEnvVariableRequest,
    CreateHealthCheckRequest, CreateReleaseRequest, CreateSiteRequest, DeployAppApi,
    DeployAppRequestContext, ListDomainZonesQuery, ListSitesQuery, UpdateDomainZoneRequest,
    UpdateSiteCompositionRequest, UpdateSiteRequest,
};
use sdkwork_routes_deploy_common::{
    envelope, finish_api_json, finish_created_api_json, finish_no_content, ok_json, service_result,
};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_app_context, paths};

#[derive(Clone)]
pub struct AppState {
    pub api: Arc<dyn DeployAppApi>,
}

pub fn build_router_with_app_api<A>(api: A) -> Router
where
    A: DeployAppApi + 'static,
{
    build_router_with_shared_app_api(Arc::new(api))
}

/// Composable domain management block: root-zone and hostname routes. The
/// block is mounted by the Deployments standalone gateway as part of the full
/// app API and by consuming hosts (for example the Web Server standalone
/// gateway) as an independent same-origin dependency contribution.
pub fn build_domain_management_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            paths::DOMAIN_ZONES,
            get(list_domain_zones).post(create_domain_zone),
        )
        .route(
            paths::DOMAIN_ZONE,
            get(retrieve_domain_zone)
                .patch(update_domain_zone)
                .delete(delete_domain_zone),
        )
        .route(
            paths::DOMAIN_ZONE_HOSTNAMES,
            get(list_domain_hostnames).post(create_domain_hostname),
        )
        .route(
            paths::DOMAIN_ZONE_HOSTNAME,
            get(retrieve_domain_hostname).delete(delete_domain_hostname),
        )
        .route(
            paths::DOMAIN_ZONE_HOSTNAME_VERIFY,
            post(verify_domain_hostname),
        )
        .layer(axum::middleware::from_fn(
            sdkwork_routes_deploy_common::pagination::validate_pagination_query,
        ))
}

/// Composable certificate management block: certificate and renewal routes.
pub fn build_certificate_management_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            paths::CERTIFICATES,
            get(list_certificates).post(create_certificate),
        )
        .route(
            paths::CERTIFICATE,
            get(retrieve_certificate).delete(delete_certificate),
        )
        .route(paths::CERTIFICATE_RENEW, post(renew_certificate))
        .layer(axum::middleware::from_fn(
            sdkwork_routes_deploy_common::pagination::validate_pagination_query,
        ))
}

pub fn build_router_with_shared_app_api(api: Arc<dyn DeployAppApi>) -> Router {
    Router::<AppState>::new()
        .merge(build_domain_management_router())
        .merge(build_certificate_management_router())
        .route(paths::SITES, get(list_sites).post(create_site))
        .route(
            paths::SITE,
            get(retrieve_site).patch(update_site).delete(delete_site),
        )
        .route(paths::SITE_COMPOSITION, put(update_site_composition))
        .route(paths::SITE_ACTIVATE, post(activate_site))
        .route(paths::SITE_PAUSE, post(pause_site))
        .route(
            paths::SITE_DEPLOYMENTS,
            get(list_deployments).post(create_deployment),
        )
        .route(paths::SITE_DEPLOYMENT, get(retrieve_deployment))
        .route(paths::SITE_DEPLOYMENT_ROLLBACK, post(rollback_deployment))
        .route(
            paths::SITE_RELEASES,
            get(list_releases).post(create_release),
        )
        .route(paths::SITE_RELEASE, get(retrieve_release))
        .route(
            paths::SITE_ENV_VARIABLES,
            get(list_env_variables).post(create_env_variable),
        )
        .route(paths::UPLOAD_SESSIONS, post(create_upload_session))
        .route(paths::UPLOAD_SESSION, get(retrieve_upload_session))
        .route(
            paths::UPLOAD_SESSION_COMPLETE,
            post(complete_upload_session),
        )
        .route(paths::UPLOAD_SESSION_CANCEL, post(cancel_upload_session))
        .route(paths::ARTIFACTS, get(list_artifacts).post(create_artifact))
        .route(
            paths::ARTIFACT,
            get(retrieve_artifact).delete(retain_artifact),
        )
        .route(
            paths::SITE_HEALTH_CHECKS,
            get(list_health_checks).post(create_health_check),
        )
        .layer(axum::middleware::from_fn(
            sdkwork_routes_deploy_common::pagination::validate_pagination_query,
        ))
        .with_state(AppState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
}

#[derive(Debug, Deserialize)]
struct DeploymentListQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
    status: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct EnvVariableListQuery {
    environment: Option<String>,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

async fn list_domain_zones(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Query(query): Query<ListDomainZonesQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state.api.list_domain_zones(&context, &query).await?;
            ok_json(envelope::domain_zone_page(page))
        }
        .await,
    )
}

async fn create_domain_zone(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Json(request): Json<CreateDomainZoneRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.create_domain_zone(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_domain_zone(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(zone_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.retrieve_domain_zone(&context, &zone_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_domain_zone(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(zone_id): Path<String>,
    Json(request): Json<UpdateDomainZoneRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .update_domain_zone(&context, &zone_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn delete_domain_zone(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(zone_id): Path<String>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(state.api.delete_domain_zone(&context, &zone_id).await)
        }
        .await,
    )
}

async fn list_domain_hostnames(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(zone_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_domain_hostnames(&context, &zone_id, query.page, query.page_size)
                .await?;
            ok_json(envelope::domain_hostname_page(page))
        }
        .await,
    )
}

async fn create_domain_hostname(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(zone_id): Path<String>,
    Json(request): Json<CreateDomainHostnameRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_domain_hostname(&context, &zone_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_domain_hostname(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((zone_id, hostname_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_domain_hostname(&context, &zone_id, &hostname_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn delete_domain_hostname(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((zone_id, hostname_id)): Path<(String, String)>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(
                state
                    .api
                    .delete_domain_hostname(&context, &zone_id, &hostname_id)
                    .await,
            )
        }
        .await,
    )
}

async fn verify_domain_hostname(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((zone_id, hostname_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .verify_domain_hostname(&context, &zone_id, &hostname_id)
                .await?;
            ok_json(envelope::domain_verify(item))
        }
        .await,
    )
}

async fn list_sites(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Query(query): Query<ListSitesQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state.api.list_sites(&context, &query).await?;
            ok_json(envelope::site_page(page))
        }
        .await,
    )
}

async fn create_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Json(request): Json<CreateSiteRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.create_site(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.retrieve_site(&context, &site_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<UpdateSiteRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.update_site(&context, &site_id, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_site_composition(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<UpdateSiteCompositionRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let expected_site_version = parse_if_match(&headers)?;
            let idempotency_key = required_header(&headers, "idempotency-key")?;
            let item = state
                .api
                .update_site_composition(
                    &context,
                    &site_id,
                    expected_site_version,
                    &idempotency_key,
                    &request,
                )
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

fn parse_if_match(headers: &HeaderMap) -> Result<i64, sdkwork_deploy_contract::DeployServiceError> {
    let value = required_header(headers, "if-match")?;
    if value == "*" || value.starts_with("W/") {
        return Err(sdkwork_deploy_contract::DeployServiceError::validation(
            "If-Match must contain one strong decimal site version",
        ));
    }
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(&value);
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(sdkwork_deploy_contract::DeployServiceError::validation(
            "If-Match must contain one strong decimal site version",
        ));
    }
    value.parse::<i64>().map_err(|_| {
        sdkwork_deploy_contract::DeployServiceError::validation(
            "If-Match site version is outside the supported range",
        )
    })
}

fn required_header(
    headers: &HeaderMap,
    name: &'static str,
) -> Result<String, sdkwork_deploy_contract::DeployServiceError> {
    let value = headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            sdkwork_deploy_contract::DeployServiceError::validation(format!(
                "{name} header is required"
            ))
        })?;
    if value.len() > 128 || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(sdkwork_deploy_contract::DeployServiceError::validation(
            format!("{name} header is invalid"),
        ));
    }
    Ok(value.to_owned())
}

async fn delete_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(state.api.delete_site(&context, &site_id).await)
        }
        .await,
    )
}

async fn activate_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.activate_site(&context, &site_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn pause_site(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.pause_site(&context, &site_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_deployments(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<DeploymentListQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_deployments(
                    &context,
                    &site_id,
                    query.page,
                    query.page_size,
                    query.status,
                )
                .await?;
            ok_json(envelope::deployment_page(page))
        }
        .await,
    )
}

async fn create_deployment(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateDeploymentRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_deployment(&context, &site_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_deployment(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, deployment_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_deployment(&context, &site_id, &deployment_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn rollback_deployment(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, deployment_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .rollback_deployment(&context, &site_id, &deployment_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_releases(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_releases(&context, &site_id, query.page, query.page_size)
                .await?;
            ok_json(envelope::release_page(page))
        }
        .await,
    )
}

async fn create_release(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateReleaseRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_release(&context, &site_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_release(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, release_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_release(&context, &site_id, &release_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_artifacts(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_artifacts(&context, query.page, query.page_size)
                .await?;
            ok_json(envelope::artifact_page(page, query.page, query.page_size))
        }
        .await,
    )
}

async fn create_artifact(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Json(request): Json<CreateArtifactRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.create_artifact(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_artifact(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(artifact_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.retrieve_artifact(&context, &artifact_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retain_artifact(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(artifact_id): Path<String>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(state.api.retain_artifact(&context, &artifact_id).await)
        }
        .await,
    )
}

async fn list_env_variables(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Query(query): Query<EnvVariableListQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_env_variables(&context, &site_id, query.environment.as_deref())
                .await?;
            ok_json(envelope::env_variable_page(page))
        }
        .await,
    )
}

async fn create_env_variable(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateEnvVariableRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_env_variable(&context, &site_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_certificates(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state
                .api
                .list_certificates(&context, query.page, query.page_size)
                .await?;
            ok_json(envelope::certificate_page(
                page,
                query.page,
                query.page_size,
            ))
        }
        .await,
    )
}

async fn create_certificate(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    headers: HeaderMap,
    Json(request): Json<CreateCertificateRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let idempotency_key = required_header(&headers, "idempotency-key")?;
            let item = state
                .api
                .create_certificate(&context, &idempotency_key, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_certificate(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(certificate_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_certificate(&context, &certificate_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn delete_certificate(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(certificate_id): Path<String>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(
                state
                    .api
                    .delete_certificate(&context, &certificate_id)
                    .await,
            )
        }
        .await,
    )
}

async fn renew_certificate(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(certificate_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .renew_certificate(&context, &certificate_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_health_checks(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let page = state.api.list_health_checks(&context, &site_id).await?;
            ok_json(envelope::health_check_page(page))
        }
        .await,
    )
}

async fn create_health_check(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateHealthCheckRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_health_check(&context, &site_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn create_upload_session(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Json(request): Json<CreateDeployUploadSessionRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.create_upload_session(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_upload_session(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(upload_session_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_upload_session(&context, &upload_session_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn complete_upload_session(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(upload_session_id): Path<String>,
    Json(request): Json<CompleteDeployUploadSessionRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .complete_upload_session(&context, &upload_session_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn cancel_upload_session(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(upload_session_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .cancel_upload_session(&context, &upload_session_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_match_accepts_decimal_strong_entity_tag() {
        let mut headers = HeaderMap::new();
        headers.insert("if-match", "\"42\"".parse().unwrap());
        assert_eq!(parse_if_match(&headers).unwrap(), 42);
    }

    #[test]
    fn if_match_rejects_wildcard_and_weak_entity_tags() {
        for value in ["*", "W/\"42\"", "not-a-version"] {
            let mut headers = HeaderMap::new();
            headers.insert("if-match", value.parse().unwrap());
            assert!(parse_if_match(&headers).is_err());
        }
    }

    #[test]
    fn composition_precondition_headers_are_required() {
        let headers = HeaderMap::new();
        assert!(parse_if_match(&headers)
            .expect_err("If-Match must be required")
            .to_string()
            .contains("if-match header is required"));
        assert!(required_header(&headers, "idempotency-key")
            .expect_err("Idempotency-Key must be required")
            .to_string()
            .contains("idempotency-key header is required"));
    }
}
