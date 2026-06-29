use axum::Extension;

use sdkwork_deploy_contract::DeployBackendRequestContext;
use sdkwork_routes_deploy_common::ApiProblem;

pub fn require_backend_context(
    context: Option<Extension<DeployBackendRequestContext>>,
) -> Result<DeployBackendRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::unauthorized("authenticated backend request context is required")
    })
}
