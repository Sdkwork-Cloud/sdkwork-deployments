//! API assembly for sdkwork-deployments.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_domain_certificate_blocks, migrate_database_from_env,
    ApiAssembly, DomainCertificateBlocks,
};
/// Route inventory of the composable domain/certificate contribution, exported
/// for consuming hosts that prove same-origin mount coverage without
/// importing the dependency application's route crates (API_ASSEMBLY_SPEC
/// §3/§6.1).
pub use sdkwork_routes_deploy_app_api::domain_certificate_route_manifest;

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
