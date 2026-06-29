//! Legacy re-exports — prefer [`crate::response::ApiProblem`].

pub use crate::response::{ApiProblem as DeployApiError, ApiResult as DeployApiResult};

#[deprecated(note = "use sdkwork-web-framework problem_response via ApiProblem instead")]
pub type DeployApiProblem = crate::response::ApiProblem;
