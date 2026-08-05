//! Deploy service error model aligned with OpenAPI problem responses.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployServiceErrorKind {
    NotFound,
    Conflict,
    Validation,
    Forbidden,
    QuotaExceeded,
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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),
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
            Self::Forbidden(_) => DeployServiceErrorKind::Forbidden,
            Self::QuotaExceeded(_) => DeployServiceErrorKind::QuotaExceeded,
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

    pub fn forbidden(detail: impl Into<String>) -> Self {
        Self::Forbidden(detail.into())
    }

    /// Entitlement enforcement failure (429 Too Many Requests semantics):
    /// the tenant's plan limit for the dimension has been reached.
    pub fn quota_exceeded(detail: impl Into<String>) -> Self {
        Self::QuotaExceeded(detail.into())
    }
}

pub type DeployServiceResult<T> = Result<T, DeployServiceError>;
