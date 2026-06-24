use axum::{extract::Extension, http::StatusCode, routing::get, Router};
use sdkwork_intelligence_deploy_repository_sqlx::bootstrap_deploy_runtime_from_env;
use sdkwork_intelligence_deploy_service::DeployService;
use sdkwork_router_deploy_app_api::{
    build_router_with_shared_app_api, wrap_router_with_web_framework_from_env as wrap_app_router,
};
use sdkwork_router_deploy_backend_api::{
    build_router_with_shared_backend_api,
    wrap_router_with_web_framework_from_env as wrap_backend_router,
};
use std::sync::Arc;
use tracing::info;

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(
    Extension(service): Extension<Arc<DeployService>>,
) -> Result<&'static str, StatusCode> {
    service
        .ready_check()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok("ok")
}

pub async fn build_router() -> Result<Router, String> {
    let runtime = bootstrap_deploy_runtime_from_env().await?;
    info!("deploy runtime ready");
    let service = Arc::new(runtime.service);

    let app_business_router = build_router_with_shared_app_api(service.clone());
    let backend_business_router = build_router_with_shared_backend_api(service.clone());

    let app_router = wrap_app_router(app_business_router).await;
    let backend_router = wrap_backend_router(backend_business_router).await;

    Ok(Router::new()
        .merge(app_router)
        .merge(backend_router)
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .layer(Extension(service)))
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_DEPLOY_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_deploy_database_host::bootstrap_deploy_database_from_env().await?;
    info!("deploy database migration completed");
    Ok(())
}
