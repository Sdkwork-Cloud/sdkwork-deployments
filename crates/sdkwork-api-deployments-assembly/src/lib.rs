//! API assembly for sdkwork-deployments.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_api_router_with_pool, assemble_app_api_contribution_from_env,
    assemble_business_routes, assemble_domain_certificate_blocks, migrate_database_from_env,
    ApiAssembly, DomainCertificateBlocks,
};
// Route-manifest accessor for the composed standalone gateway inventory:
// the Web Server gateway combines this dependency's domain/certificate blocks
// into one route manifest before framework installation (API_ASSEMBLY_SPEC §6.1).
pub use sdkwork_routes_deploy_app_api::domain_certificate_route_manifest;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
