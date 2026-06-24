use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_deploy_contract::DeployServiceError;

use crate::correlation::DeployProblemCorrelation;

pub type DeployApiResult<T> = Result<T, DeployApiError>;

#[derive(Debug, Clone)]
pub struct DeployApiError {
    status: StatusCode,
    code: String,
    detail: String,
}

impl DeployApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
        }
    }
}

impl From<DeployServiceError> for DeployApiError {
    fn from(error: DeployServiceError) -> Self {
        use sdkwork_deploy_contract::DeployServiceErrorKind;
        let (status, code) = match error.kind() {
            DeployServiceErrorKind::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            DeployServiceErrorKind::Conflict => (StatusCode::CONFLICT, "conflict"),
            DeployServiceErrorKind::Validation => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error")
            }
            DeployServiceErrorKind::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            DeployServiceErrorKind::DatabaseUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "database_unavailable")
            }
            DeployServiceErrorKind::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        Self::new(status, code, error.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeployApiProblem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl DeployApiProblem {
    pub fn from_error(error: &DeployApiError) -> Self {
        let correlation = DeployProblemCorrelation::current().unwrap_or_default();
        Self {
            problem_type: format!("https://sdkwork.com/problems/{}", error.code),
            title: error.code.clone(),
            status: error.status.as_u16(),
            detail: error.detail.clone(),
            instance: None,
            request_id: Some(correlation.request_id),
            trace_id: correlation.trace_id,
        }
    }
}

impl IntoResponse for DeployApiError {
    fn into_response(self) -> Response {
        let problem = DeployApiProblem::from_error(&self);
        let mut response = (self.status, Json(problem)).into_response();
        if let Some(correlation) = DeployProblemCorrelation::current() {
            if let Ok(value) = HeaderValue::from_str(&correlation.request_id) {
                response
                    .headers_mut()
                    .insert(header::HeaderName::from_static("x-request-id"), value);
            }
        }
        response
    }
}
