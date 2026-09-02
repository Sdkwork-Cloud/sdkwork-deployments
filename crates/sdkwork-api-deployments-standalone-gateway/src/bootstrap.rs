use axum::Router;
use sdkwork_api_deployments_assembly::ApiAssembly;
use sdkwork_iam_web_adapter::{
    build_web_framework_builder, iam_web_request_context_resolver_from_env,
};
use sdkwork_web_bootstrap::{infra_public_path_prefixes, ApiModuleRegistry};

use tracing::info;

pub async fn build_router(assembly: ApiAssembly) -> Result<Router, String> {
    info!("deploy runtime ready");
    let contribution = assembly.contribution;
    let framework = build_web_framework_builder(
        iam_web_request_context_resolver_from_env().await,
        contribution.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    let mut module_registry = ApiModuleRegistry::new();
    module_registry.add_modules(vec![contribution]);
    Ok(module_registry
        .try_compose("SDKWork Deployments API")?
        .into_hosted(framework)
        .router)
}

pub async fn run_database_migrate_only() -> Result<(), String> {
    sdkwork_api_deployments_assembly::migrate_database_from_env().await?;
    info!("deploy database migration completed");
    Ok(())
}
