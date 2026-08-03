//! Domain management and certificate management route inventory subset.
//!
//! The full app API manifest stays the generated single source of truth
//! (`http_route_manifest.rs`). This file derives the composable domain
//! management + certificate management inventory from it so the same
//! normalized routes back the executable blocks
//! (`build_domain_management_router` / `build_certificate_management_router`)
//! in every host that mounts them.

use sdkwork_web_core::HttpRouteManifest;

use crate::http_route_manifest::app_route_manifest;

pub fn domain_certificate_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::from_owned_routes(
        app_route_manifest()
            .routes()
            .iter()
            .filter(|route| {
                route.path.starts_with("/app/v3/api/domain_zones")
                    || route.path.starts_with("/app/v3/api/certificates")
            })
            .copied()
            .collect(),
    )
}
