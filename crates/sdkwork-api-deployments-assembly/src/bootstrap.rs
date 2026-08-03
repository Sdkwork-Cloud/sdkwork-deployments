//! Business-only gateway bootstrap for sdkwork-deployments.

use axum::{Extension, Router};
use sdkwork_deploy_service_host::bootstrap_deploy_service_host_from_env;
use sdkwork_intelligence_deploy_service::DeployService;
use sdkwork_routes_deploy_app_api::{
    build_certificate_management_router, build_domain_management_router,
    deploy_app_api_domain_context_injectors, domain_certificate_route_manifest,
    gateway_mount as mount_app, wrap_router_with_web_framework_from_env as wrap_app, AppState,
};
use sdkwork_routes_deploy_backend_api::{
    gateway_mount as mount_backend, wrap_router_with_web_framework_from_env as wrap_backend,
};
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};
use sdkwork_web_core::{DomainContextInjector, HttpRouteManifest};
use std::sync::Arc;

pub struct ApiAssembly {
    pub router: Router,
    pub service: Arc<DeployService>,
}

pub async fn assemble_business_routes() -> Result<ApiAssembly, String> {
    let service = bootstrap_deploy_service_host_from_env().await?.service;
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

/// Composable domain management + certificate management contribution for
/// consuming hosts (for example the Web Server standalone gateway).
///
/// The blocks are host-neutral business routers plus their complete route
/// inventory (manifest, domain context injectors, readiness). Consuming
/// gateways merge them before installing their single Web Framework layer, so
/// the blocks stay un-wrapped and authenticate through the host framework.
pub struct DomainCertificateBlocks {
    pub router: Router,
    pub route_manifest: HttpRouteManifest,
    pub domain_context_injectors: Vec<Arc<dyn DomainContextInjector>>,
    pub readiness_check: Arc<dyn ReadinessCheck>,
}

pub async fn assemble_domain_certificate_blocks() -> Result<DomainCertificateBlocks, String> {
    let service = bootstrap_deploy_service_host_from_env().await?.service;
    let router = Router::new()
        .merge(build_domain_management_router())
        .merge(build_certificate_management_router())
        .with_state(AppState { api: service.clone() });
    let route_manifest = domain_certificate_route_manifest();
    let domain_context_injectors = deploy_app_api_domain_context_injectors();
    let readiness_check: Arc<dyn ReadinessCheck> =
        Arc::new(DeployServiceReadinessCheck { service });
    Ok(DomainCertificateBlocks {
        router,
        route_manifest,
        domain_context_injectors,
        readiness_check,
    })
}

/// Migrate-only entrypoint for the Deployments database module, reusable by
/// consuming hosts (for example the Web Server standalone gateway's
/// `db-migrate` command). Baseline applies on empty databases; versioned
/// forward migrations converge existing databases; the drift gate then
/// fails loudly if the schema still diverges from the contract.
pub async fn migrate_database_from_env() -> Result<(), String> {
    std::env::set_var("SDKWORK_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_deploy_database_host::bootstrap_deploy_database_from_env()
        .await
        .map(|_| ())
        .map_err(|detail| format!("deploy database migration failed: {detail}"))
}

struct DeployServiceReadinessCheck {
    service: Arc<DeployService>,
}

impl ReadinessCheck for DeployServiceReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let service = self.service.clone();
        Box::pin(async move {
            service
                .ready_check()
                .await
                .map_err(|error| error.to_string())
        })
    }
}
