use axum::Extension;

use sdkwork_deploy_contract::DeployAppRequestContext;
use sdkwork_routes_deploy_common::ApiProblem;

pub fn require_app_context(
    context: Option<Extension<DeployAppRequestContext>>,
) -> Result<DeployAppRequestContext, ApiProblem> {
    context
        .map(|Extension(context)| context)
        .ok_or_else(|| ApiProblem::unauthorized("authenticated app request context is required"))
}
