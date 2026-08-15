use axum::Router;
use sdkwork_api_deployments_assembly::ApiAssembly;
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use tracing::info;

pub fn build_router(assembly: ApiAssembly) -> Router {
    info!("deploy runtime ready");
    let service_router_config = ServiceRouterConfig::default()
        .with_readiness_check(assembly.contribution.readiness_check.clone());

    service_router(assembly.contribution.router, service_router_config)
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    sdkwork_api_deployments_assembly::migrate_database_from_env().await?;
    info!("deploy database migration completed");
    Ok(())
}
