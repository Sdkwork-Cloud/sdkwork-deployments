use std::sync::Arc;

use axum::Router;
use sdkwork_deploy_contract::DeployBackendRequestContext;
use sdkwork_router_deploy_common::{
    deploy_web_auth_mode_from_env, with_problem_correlation, DeployWebAuthMode,
    ProductionFailClosedResolver,
};
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, DomainContextInjector, WebRequestContext,
    WebRequestContextProfile,
};

use crate::http_route_manifest::backend_route_manifest;
use crate::paths;

#[derive(Clone, Default)]
struct DeployBackendContextInjector;

impl DomainContextInjector for DeployBackendContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(backend_context) = deploy_backend_context_from_web_request(context) {
            request.extensions_mut().insert(backend_context);
        }
    }
}

fn deploy_backend_context_from_web_request(
    context: &WebRequestContext,
) -> Option<DeployBackendRequestContext> {
    let principal = context.principal.as_ref()?;
    Some(DeployBackendRequestContext {
        operator_id: principal.user_id().parse().ok(),
        tenant_id: principal.tenant_id().parse().ok(),
    })
}

fn build_deploy_backend_api_framework_layer<R>(resolver: R) -> WebFrameworkLayer<R>
where
    R: sdkwork_web_core::WebRequestContextResolver + Clone,
{
    WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            backend_api_prefix: paths::PREFIX.to_owned(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(backend_route_manifest())
        .with_domain_injector(Arc::new(DeployBackendContextInjector))
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    match deploy_web_auth_mode_from_env().await {
        DeployWebAuthMode::DevInline => with_web_request_context(
            with_problem_correlation(router),
            build_deploy_backend_api_framework_layer(DefaultWebRequestContextResolver::default()),
        ),
        DeployWebAuthMode::ProductionFailClosed => with_web_request_context(
            with_problem_correlation(router),
            build_deploy_backend_api_framework_layer(ProductionFailClosedResolver),
        ),
        DeployWebAuthMode::IamDatabase(resolver) => with_web_request_context(
            with_problem_correlation(router),
            build_deploy_backend_api_framework_layer(resolver),
        ),
    }
}
