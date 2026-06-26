//! App API route boundary for SDKWork Deploy.

pub mod auth;
pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod web_bootstrap;

pub use http_route_manifest::app_route_manifest;
pub use routes::{build_router_with_app_api, build_router_with_shared_app_api};
pub use sdkwork_deploy_contract::{DeployAppApi, DeployAppRequestContext};
pub use web_bootstrap::{
    deploy_app_api_prefixes, deploy_app_api_public_path_prefixes,
    wrap_router_with_web_framework_from_env,
};
