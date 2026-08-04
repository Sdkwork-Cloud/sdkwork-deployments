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

    service_router(assembly.contribution.router, service_router_config)
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    sdkwork_api_deployments_assembly::migrate_database_from_env().await?;
    info!("deploy database migration completed");
    Ok(())
}
