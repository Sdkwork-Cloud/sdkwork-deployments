//! Deploy service error model aligned with OpenAPI problem responses.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployServiceErrorKind {
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    DatabaseUnavailable,
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum DeployServiceError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("forbidden")]
    Forbidden,
    #[error("database unavailable")]
    DatabaseUnavailable,
    #[error("internal error: {0}")]
    Internal(String),
}

impl DeployServiceError {
    pub fn kind(&self) -> DeployServiceErrorKind {
        match self {
            Self::NotFound(_) => DeployServiceErrorKind::NotFound,
            Self::Conflict(_) => DeployServiceErrorKind::Conflict,
            Self::Validation(_) => DeployServiceErrorKind::Validation,
            Self::Forbidden => DeployServiceErrorKind::Forbidden,
            Self::DatabaseUnavailable => DeployServiceErrorKind::DatabaseUnavailable,
            Self::Internal(_) => DeployServiceErrorKind::Internal,
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::NotFound(detail.into())
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::Conflict(detail.into())
    }

    pub fn validation(detail: impl Into<String>) -> Self {
        Self::Validation(detail.into())
    }
}

pub type DeployServiceResult<T> = Result<T, DeployServiceError>;
