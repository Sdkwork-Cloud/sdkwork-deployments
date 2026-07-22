//! Generated Web Internal SDK adapter for runtime assignment publication.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_deploy_runtime_compiler::CompiledRuntimeSet;
use sdkwork_utils_rust::string::trim;
use sdkwork_web_internal_sdk::{
    PublishRuntimeAssignmentRequest, RuntimeAssignment, SdkworkCustomClient, SdkworkError,
    WebsiteRuntimeSetSnapshot,
};

pub const WEB_INTERNAL_API_URL_ENV: &str = "SDKWORK_DEPLOY_WEB_INTERNAL_API_URL";
pub const WEB_INTERNAL_API_INGRESS_TOKEN_FILE_ENV: &str =
    "SDKWORK_DEPLOY_WEB_INTERNAL_API_INGRESS_TOKEN_FILE";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAssignmentReceipt {
    pub assignment_uuid: String,
    pub node_uuid: String,
    pub environment: String,
    pub generation: String,
    pub snapshot_uuid: String,
    pub snapshot_sha256: String,
    pub assigned_at: String,
}

#[async_trait]
pub trait DeployWebRuntimePort: Send + Sync {
    async fn publish_runtime_assignment(
        &self,
        runtime_set: &CompiledRuntimeSet,
    ) -> DeployServiceResult<RuntimeAssignmentReceipt>;
}

#[derive(Clone, Debug, Default)]
pub struct UnconfiguredWebRuntimePort;

#[async_trait]
impl DeployWebRuntimePort for UnconfiguredWebRuntimePort {
    async fn publish_runtime_assignment(
        &self,
        _runtime_set: &CompiledRuntimeSet,
    ) -> DeployServiceResult<RuntimeAssignmentReceipt> {
        Err(DeployServiceError::Internal(
            "Web runtime publication is not configured".to_owned(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct SdkWebRuntimeFacade {
    base_url: String,
    ingress_token_file: PathBuf,
}

impl SdkWebRuntimeFacade {
    pub fn new(base_url: String, ingress_token_file: PathBuf) -> Result<Self, String> {
        let base_url = trim(&base_url);
        if base_url.is_empty() {
            return Err("Web Internal API URL must not be blank".to_owned());
        }
        if ingress_token_file.as_os_str().is_empty() {
            return Err("Web Internal API ingress token file must not be blank".to_owned());
        }
        Ok(Self {
            base_url: base_url.to_owned(),
            ingress_token_file,
        })
    }

    pub fn from_env() -> Result<Self, String> {
        let base_url = std::env::var(WEB_INTERNAL_API_URL_ENV)
            .map_err(|_| format!("{WEB_INTERNAL_API_URL_ENV} is required"))?;
        let token_file = std::env::var(WEB_INTERNAL_API_INGRESS_TOKEN_FILE_ENV)
            .map_err(|_| format!("{WEB_INTERNAL_API_INGRESS_TOKEN_FILE_ENV} is required"))?;
        Self::new(base_url, PathBuf::from(token_file))
    }

    fn client(&self) -> DeployServiceResult<SdkworkCustomClient> {
        let token = read_secret_file(&self.ingress_token_file)?;
        let client =
            SdkworkCustomClient::new_with_base_url(&self.base_url).map_err(map_web_sdk_error)?;
        client.set_api_key(token);
        Ok(client)
    }
}

#[async_trait]
impl DeployWebRuntimePort for SdkWebRuntimeFacade {
    async fn publish_runtime_assignment(
        &self,
        runtime_set: &CompiledRuntimeSet,
    ) -> DeployServiceResult<RuntimeAssignmentReceipt> {
        let snapshot: WebsiteRuntimeSetSnapshot =
            serde_json::from_value(runtime_set.snapshot.clone()).map_err(|error| {
                DeployServiceError::Internal(format!(
                    "map compiled runtime-set to generated Web SDK model: {error}"
                ))
            })?;
        let node_uuid = snapshot.node_uuid.clone();
        let environment = snapshot
            .environment
            .as_str()
            .ok_or_else(|| {
                DeployServiceError::Internal(
                    "compiled runtime-set environment is not a string".to_owned(),
                )
            })?
            .to_owned();
        let response = self
            .client()?
            .runtime()
            .assignments_update(
                &node_uuid,
                &environment,
                &PublishRuntimeAssignmentRequest {
                    runtime_set: snapshot,
                },
            )
            .await
            .map_err(map_web_sdk_error)?;
        Ok(response.into())
    }
}

impl From<RuntimeAssignment> for RuntimeAssignmentReceipt {
    fn from(value: RuntimeAssignment) -> Self {
        Self {
            assignment_uuid: value.assignment_uuid,
            node_uuid: value.node_uuid,
            environment: value.environment,
            generation: value.generation,
            snapshot_uuid: value.snapshot_uuid,
            snapshot_sha256: value.snapshot_sha256,
            assigned_at: value.assigned_at,
        }
    }
}

fn read_secret_file(path: &Path) -> DeployServiceResult<String> {
    let value = std::fs::read_to_string(path).map_err(|error| {
        DeployServiceError::Internal(format!(
            "read Web Internal API ingress token file failed: {error}"
        ))
    })?;
    let value = trim(&value);
    if !(16..=4_096).contains(&value.len()) || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DeployServiceError::Internal(
            "Web Internal API ingress token file contains an invalid credential".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn map_web_sdk_error(error: SdkworkError) -> DeployServiceError {
    let message = error.to_string();
    if message.contains("404") || message.contains("not found") {
        DeployServiceError::not_found("Web runtime assignment target was not found")
    } else if message.contains("409") || message.contains("conflict") {
        DeployServiceError::conflict("Web runtime assignment generation conflicts")
    } else if message.contains("400") || message.contains("validation") {
        DeployServiceError::validation("Web rejected the compiled runtime assignment")
    } else if message.contains("401") || message.contains("403") {
        DeployServiceError::Forbidden
    } else {
        DeployServiceError::Internal("Web runtime assignment publication failed".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn secret_file_validation_accepts_bounded_rotatable_credential() {
        let path = std::env::temp_dir().join(format!(
            "sdkwork-deploy-web-token-{}",
            sdkwork_utils_rust::sha256_hash(b"web-port-test")
        ));
        fs::write(&path, "0123456789abcdef\n").unwrap();
        assert_eq!(read_secret_file(&path).unwrap(), "0123456789abcdef");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn sdk_errors_are_redacted_at_the_provider_boundary() {
        let error = SdkworkError::HttpStatus {
            status: 500,
            body: "upstream token=do-not-leak".to_owned(),
        };
        let mapped = map_web_sdk_error(error);
        assert_eq!(
            mapped.to_string(),
            "internal error: Web runtime assignment publication failed"
        );
    }
}
