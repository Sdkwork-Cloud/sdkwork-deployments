use axum::{http::StatusCode, Extension};

use sdkwork_deploy_contract::DeployBackendRequestContext;
use sdkwork_router_deploy_common::DeployApiError;

pub fn require_backend_context(
    context: Option<Extension<DeployBackendRequestContext>>,
) -> Result<DeployBackendRequestContext, DeployApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        DeployApiError::new(
            StatusCode::UNAUTHORIZED,
            "missing_backend_request_context",
            "authenticated backend request context is required",
        )
    })
}
