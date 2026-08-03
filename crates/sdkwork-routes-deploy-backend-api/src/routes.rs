use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post, put},
    Extension, Json, Router,
};
use sdkwork_deploy_contract::{
    AuditLogQuery, CreateNginxConfigRequest, CreateNodeClusterRequest, CreateServerRequest,
    DeployBackendApi, DeployBackendRequestContext, ListNginxConfigsQuery,
    UpdateNginxConfigRequest, UpdateNodeClusterRequest, UpdateServerRequest,
};
use sdkwork_routes_deploy_common::{envelope, finish_api_json, finish_created_api_json, ok_json};
use sdkwork_web_core::WebRequestContext;
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::require_backend_context, paths};

#[derive(Clone)]
struct BackendState {
    api: Arc<dyn DeployBackendApi>,
}

pub fn build_router_with_backend_api<A>(api: A) -> Router
where
    A: DeployBackendApi + 'static,
{
    build_router_with_shared_backend_api(Arc::new(api))
}

pub fn build_router_with_shared_backend_api(api: Arc<dyn DeployBackendApi>) -> Router {
    Router::new()
        .route(
            paths::NGINX_CONFIGS,
            get(list_nginx_configs).post(create_nginx_config),
        )
        .route(
            paths::NGINX_CONFIG,
            get(retrieve_nginx_config).put(update_nginx_config),
        )
        .route(paths::NGINX_CONFIG_VALIDATE, post(validate_nginx_config))
        .route(paths::NGINX_CONFIG_DEPLOY, post(deploy_nginx_config))
        .route(paths::NGINX_RELOAD, post(reload_nginx))
        .route(paths::NGINX_STATUS, get(retrieve_nginx_status))
        .route(paths::SERVERS, get(list_servers).post(create_server))
        .route(paths::SERVER, put(update_server))
        .route(
            paths::NODE_CLUSTERS,
            get(list_node_clusters).post(create_node_cluster),
        )
        .route(paths::NODE_CLUSTER, put(update_node_cluster))
        .route(paths::AUDIT_LOGS, get(list_audit_logs))
        .layer(axum::middleware::from_fn(
            sdkwork_routes_deploy_common::pagination::validate_pagination_query,
        ))
        .with_state(BackendState { api })
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_page_size")]
    page_size: i32,
    // 规范 wire 词汇（PAGINATION_SPEC §3）：lower_snake_case，无 camelCase 别名。
    #[serde(default)]
    cluster_id: Option<String>,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

async fn list_nginx_configs(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Query(query): Query<ListNginxConfigsQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let page = state.api.list_nginx_configs(&context, &query).await?;
            ok_json(envelope::nginx_config_page(page))
        }
        .await,
    )
}

async fn create_nginx_config(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Json(request): Json<CreateNginxConfigRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.create_nginx_config(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn retrieve_nginx_config(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state
                .api
                .retrieve_nginx_config(&context, &config_id)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_nginx_config(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(config_id): Path<String>,
    Json(request): Json<UpdateNginxConfigRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state
                .api
                .update_nginx_config(&context, &config_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn validate_nginx_config(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state
                .api
                .validate_nginx_config(&context, &config_id)
                .await?;
            ok_json(envelope::nginx_validate(item))
        }
        .await,
    )
}

async fn deploy_nginx_config(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(config_id): Path<String>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.deploy_nginx_config(&context, &config_id).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn reload_nginx(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.reload_nginx(&context).await?;
            ok_json(envelope::nginx_reload(item))
        }
        .await,
    )
}

async fn retrieve_nginx_status(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.retrieve_nginx_status(&context).await?;
            ok_json(envelope::nginx_status(item))
        }
        .await,
    )
}

async fn list_servers(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let page = state
                .api
                .list_servers(&context, query.page, query.page_size, query.cluster_id)
                .await?;
            ok_json(envelope::server_page(page, query.page, query.page_size))
        }
        .await,
    )
}

async fn create_server(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Json(request): Json<CreateServerRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.create_server(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_server(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(server_id): Path<String>,
    Json(request): Json<UpdateServerRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state
                .api
                .update_server(&context, &server_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_node_clusters(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Query(query): Query<PageQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let page = state
                .api
                .list_node_clusters(&context, query.page, query.page_size)
                .await?;
            ok_json(envelope::node_cluster_page(
                page,
                query.page,
                query.page_size,
            ))
        }
        .await,
    )
}

async fn create_node_cluster(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Json(request): Json<CreateNodeClusterRequest>,
) -> Response {
    finish_created_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state.api.create_node_cluster(&context, &request).await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn update_node_cluster(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Path(cluster_id): Path<String>,
    Json(request): Json<UpdateNodeClusterRequest>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let item = state
                .api
                .update_node_cluster(&context, &cluster_id, &request)
                .await?;
            ok_json(envelope::resource(item))
        }
        .await,
    )
}

async fn list_audit_logs(
    ctx: WebRequestContext,
    State(state): State<BackendState>,
    context: Option<Extension<DeployBackendRequestContext>>,
    Query(query): Query<AuditLogQuery>,
) -> Response {
    finish_api_json(
        &ctx,
        async {
            let context = require_backend_context(context)?;
            let page = state.api.list_audit_logs(&context, &query).await?;
            ok_json(envelope::audit_log_page(page))
        }
        .await,
    )
}
