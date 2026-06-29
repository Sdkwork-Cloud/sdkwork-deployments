use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_deploy_contract::{
    CancelDeployUploadSessionRequest, CompleteDeployUploadSessionRequest, CreateCertificateRequest,
    CreateDeployUploadSessionRequest, CreateDeploymentRequest, CreateDomainRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreateSiteRequest, DeployAppApi,
    DeployAppRequestContext, ListSitesQuery, UpdateSiteRequest,
};
use sdkwork_routes_deploy_common::{
    envelope, finish_api_json, finish_created_api_json, finish_no_content, ok_json, service_result,
};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_app_context, paths};

#[derive(Clone)]
struct AppState {
    api: Arc<dyn DeployAppApi>,
}

pub fn build_router_with_app_api<A>(api: A) -> Router
where
    A: DeployAppApi + 'static,
{
    build_router_with_shared_app_api(Arc::new(api))
}

pub fn build_router_with_shared_app_api(api: Arc<dyn DeployAppApi>) -> Router {
    Router::new()
        .route(paths::SITES, get(list_sites).post(create_site))
        .route(
            paths::SITE,
            get(retrieve_site).patch(update_site).delete(delete_site),
        )
        .route(paths::SITE_ACTIVATE, post(activate_site))
        .route(paths::SITE_PAUSE, post(pause_site))
        .route(paths::SITE_DOMAINS, get(list_domains).post(create_domain))
        .route(
            paths::SITE_DOMAIN,
            get(retrieve_domain).delete(delete_domain),
        )
        .route(paths::SITE_DOMAIN_VERIFY, post(verify_domain))
        .route(
            paths::SITE_DEPLOYMENTS,
            get(list_deployments).post(create_deployment),
        )
        .route(paths::SITE_DEPLOYMENT, get(retrieve_deployment))
        .route(paths::SITE_DEPLOYMENT_ROLLBACK, post(rollback_deployment))
        .route(
            paths::SITE_ENV_VARIABLES,
            get(list_env_variables).post(create_env_variable),
        )
        .route(
            paths::CERTIFICATES,
            get(list_certificates).post(create_certificate),
        )
        .route(paths::UPLOAD_SESSIONS, post(create_upload_session))
        .route(paths::UPLOAD_SESSION, get(retrieve_upload_session))
        .route(
            paths::UPLOAD_SESSION_COMPLETE,
            post(complete_upload_session),
        )
        .route(paths::UPLOAD_SESSION_CANCEL, post(cancel_upload_session))
        .route(
            paths::SITE_HEALTH_CHECKS,
            get(list_health_checks).post(create_health_check),
        )
        .with_state(AppState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i32,
}

#[derive(Debug, Deserialize)]
struct DeploymentListQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size", rename = "pageSize")]
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

async fn list_domains(
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
                .list_domains(&context, &site_id, query.page, query.page_size)
                .await?;
            ok_json(envelope::domain_page(page, query.page, query.page_size))
        }
        .await,
    )
}

async fn create_domain(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path(site_id): Path<String>,
    Json(request): Json<CreateDomainRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .create_domain(&context, &site_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_domain(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .retrieve_domain(&context, &site_id, &domain_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn delete_domain(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Response {
    finish_no_content(
        &ctx,
        async {
            let context = require_app_context(context)?;
            service_result(
                state
                    .api
                    .delete_domain(&context, &site_id, &domain_id)
                    .await,
            )
        }
        .await,
    )
}

async fn verify_domain(
    ctx: WebRequestContext,
    State(state): State<AppState>,
    context: Option<Extension<DeployAppRequestContext>>,
    Path((site_id, domain_id)): Path<(String, String)>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .verify_domain(&context, &site_id, &domain_id)
                .await?;
            ok_json(envelope::domain_verify(item))
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
    Json(request): Json<CreateCertificateRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state.api.create_certificate(&context, &request).await?;
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
    Json(request): Json<CancelDeployUploadSessionRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_app_context(context)?;
            let item = state
                .api
                .cancel_upload_session(&context, &upload_session_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}
