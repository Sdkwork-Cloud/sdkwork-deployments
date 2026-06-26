use axum::{http::StatusCode, Extension};

use sdkwork_deploy_contract::DeployAppRequestContext;
use sdkwork_routes_deploy_common::DeployApiError;

pub fn require_app_context(
    context: Option<Extension<DeployAppRequestContext>>,
) -> Result<DeployAppRequestContext, DeployApiError> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        DeployApiError::new(
            StatusCode::UNAUTHORIZED,
            "missing_app_request_context",
            "authenticated app request context is required",
        )
    })
}
