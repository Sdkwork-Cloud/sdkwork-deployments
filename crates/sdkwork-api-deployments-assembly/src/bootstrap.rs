//! Business-only gateway bootstrap for sdkwork-deployments.

use axum::{Extension, Router};
use sdkwork_intelligence_deploy_repository_sqlx::bootstrap_deploy_runtime_from_env;
use sdkwork_intelligence_deploy_service::DeployService;
use sdkwork_routes_deploy_app_api::{
    gateway_mount as mount_app, wrap_router_with_web_framework_from_env as wrap_app,
};
use sdkwork_routes_deploy_backend_api::{
    gateway_mount as mount_backend, wrap_router_with_web_framework_from_env as wrap_backend,
};
use std::sync::Arc;

pub struct ApiAssembly {
    pub router: Router,
    pub service: Arc<DeployService>,
}

pub async fn assemble_business_routes() -> Result<ApiAssembly, String> {
    let runtime = bootstrap_deploy_runtime_from_env().await?;
    let service = Arc::new(runtime.service);
    let app = wrap_app(mount_app(service.clone())).await;
    let backend = wrap_backend(mount_backend(service.clone())).await;
    Ok(ApiAssembly {
        router: Router::new()
            .merge(app)
            .merge(backend)
            .layer(Extension(service.clone())),
        service,
    })
}

pub async fn assemble_api_router() -> Result<ApiAssembly, String> {
    assemble_business_routes().await
}
