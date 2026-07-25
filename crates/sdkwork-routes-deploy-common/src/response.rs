use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_deploy_contract::DeployServiceError;
use sdkwork_utils_rust::SdkWorkApiResponse;
use sdkwork_web_core::{
    problem_response, WebFrameworkError, WebFrameworkErrorKind, WebRequestContext,
};
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiProblem>;

pub fn ok_json<T>(data: T) -> ApiResult<T> {
    Ok(data)
}

pub fn success_json<T: Serialize>(
    ctx: &WebRequestContext,
    data: T,
) -> Result<Response, ApiProblem> {
    success_response(ctx, StatusCode::OK, data)
}

pub fn created_json<T: Serialize>(
    ctx: &WebRequestContext,
    data: T,
) -> Result<Response, ApiProblem> {
    success_response(ctx, StatusCode::CREATED, data)
}

pub fn no_content(ctx: &WebRequestContext) -> Result<Response, ApiProblem> {
    let mut response = StatusCode::NO_CONTENT.into_response();
    attach_trace_header(&mut response, &ctx.resolved_trace_id());
    Ok(response)
}

fn success_response<T: Serialize>(
    ctx: &WebRequestContext,
    status: StatusCode,
    data: T,
) -> Result<Response, ApiProblem> {
    let trace_id = ctx.resolved_trace_id();
    let envelope = SdkWorkApiResponse::success(data, trace_id.clone());
    let mut response = (status, Json(envelope)).into_response();
    attach_trace_header(&mut response, &trace_id);
    Ok(response)
}

fn attach_trace_header(response: &mut Response, trace_id: &str) {
    if let Ok(value) = HeaderValue::from_str(trace_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-sdkwork-trace-id"), value);
    }
}

#[derive(Debug)]
pub struct ApiProblem {
    pub message: String,
    status: StatusCode,
}

impl ApiProblem {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::FORBIDDEN,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::NOT_FOUND,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::CONFLICT,
        }
    }

    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    pub fn dependency_unavailable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn framework_error(&self) -> WebFrameworkError {
        let kind = match self.status {
            StatusCode::BAD_REQUEST => WebFrameworkErrorKind::BadRequest,
            StatusCode::UNAUTHORIZED => WebFrameworkErrorKind::MissingCredentials,
            StatusCode::FORBIDDEN => WebFrameworkErrorKind::Forbidden,
            StatusCode::NOT_FOUND => WebFrameworkErrorKind::NotFound,
            StatusCode::CONFLICT => WebFrameworkErrorKind::Conflict,
            StatusCode::UNPROCESSABLE_ENTITY => WebFrameworkErrorKind::BadRequest,
            StatusCode::SERVICE_UNAVAILABLE => WebFrameworkErrorKind::DependencyUnavailable,
            StatusCode::INTERNAL_SERVER_ERROR => WebFrameworkErrorKind::InternalServerError,
            _ => WebFrameworkErrorKind::InternalServerError,
        };
        WebFrameworkError {
            kind,
            message: self.message.clone(),
            retry_after_seconds: None,
            auth_profile: None,
            failed_stage: None,
            reason: None,
        }
    }

    pub fn into_response_for(&self, ctx: &WebRequestContext) -> Response {
        problem_response(&self.framework_error(), ctx.problem_correlation())
    }
}

impl From<DeployServiceError> for ApiProblem {
    fn from(error: DeployServiceError) -> Self {
        use sdkwork_deploy_contract::DeployServiceErrorKind;
        match error.kind() {
            DeployServiceErrorKind::NotFound => Self::not_found(error.to_string()),
            DeployServiceErrorKind::Conflict => Self::conflict(error.to_string()),
            DeployServiceErrorKind::Validation => Self::unprocessable_entity(error.to_string()),
            DeployServiceErrorKind::Forbidden => Self::forbidden(error.to_string()),
            DeployServiceErrorKind::DatabaseUnavailable => {
                Self::dependency_unavailable(error.to_string())
            }
            DeployServiceErrorKind::Internal => Self::internal_server_error(error.to_string()),
        }
    }
}

pub fn finish_api_json<T: Serialize>(ctx: &WebRequestContext, result: ApiResult<T>) -> Response {
    match result {
        Ok(data) => {
            success_json(ctx, data).unwrap_or_else(|problem| problem.into_response_for(ctx))
        }
        Err(problem) => problem.into_response_for(ctx),
    }
}

pub fn finish_created_api_json<T: Serialize>(
    ctx: &WebRequestContext,
    result: ApiResult<T>,
) -> Response {
    match result {
        Ok(data) => {
            created_json(ctx, data).unwrap_or_else(|problem| problem.into_response_for(ctx))
        }
        Err(problem) => problem.into_response_for(ctx),
    }
}

pub fn finish_no_content(ctx: &WebRequestContext, result: ApiResult<()>) -> Response {
    match result {
        Ok(()) => no_content(ctx).unwrap_or_else(|problem| problem.into_response_for(ctx)),
        Err(problem) => problem.into_response_for(ctx),
    }
}

pub fn service_result<T, E: Into<ApiProblem>>(result: Result<T, E>) -> ApiResult<T> {
    result.map_err(Into::into)
}
