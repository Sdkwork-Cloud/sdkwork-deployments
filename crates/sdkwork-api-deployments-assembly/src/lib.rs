//! API assembly for sdkwork-deployments.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM: exports beyond the canonical materializer template.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_api_router, assemble_api_router_with_pool, assemble_business_routes,
    assemble_domain_certificate_blocks, migrate_database_from_env, ApiAssembly,
    DomainCertificateBlocks,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
