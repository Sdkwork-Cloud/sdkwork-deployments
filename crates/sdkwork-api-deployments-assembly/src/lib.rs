//! API assembly for sdkwork-deployments.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_api_router, ApiAssembly, assemble_business_routes, DomainCertificateBlocks, assemble_domain_certificate_blocks, migrate_database_from_env};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
