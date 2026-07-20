use axum::Router;
use sdkwork_api_deployments_assembly::ApiAssembly;
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use std::sync::Arc;
use tracing::info;

use crate::readiness::DeployServiceReadinessCheck;

pub fn build_router(assembly: ApiAssembly) -> Router {
    info!("deploy runtime ready");
    let service_router_config = ServiceRouterConfig::default()
        .with_readiness_check(Arc::new(DeployServiceReadinessCheck::new(assembly.service)));

    service_router(assembly.router, service_router_config)
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_DEPLOY_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_deploy_database_host::bootstrap_deploy_database_from_env().await?;
    info!("deploy database migration completed");
    Ok(())
}
