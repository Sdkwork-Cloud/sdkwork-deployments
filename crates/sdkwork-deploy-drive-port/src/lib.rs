//! Drive-backed upload delegation for SDKWork Deploy package artifacts.

mod facade;
mod memory;
mod selection;

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    CompleteDeployUploadSessionRequest, CreateDeployUploadSessionRequest, DeployServiceError,
    DeployServiceResult, DeployUploadSessionResponse,
};

pub use facade::SdkDriveAppFacade;
pub use memory::MemoryDeployDrivePort;
pub use selection::{deploy_drive_port_from_env, DeployDrivePortSelection};

#[derive(Clone, Debug, Default)]
pub struct DriveRequestCredentials {
    pub auth_token: Option<String>,
    pub access_token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PrepareDeployUploadCommand {
    pub tenant_id: i64,
    pub request: CreateDeployUploadSessionRequest,
}

#[async_trait]
pub trait DeployDrivePort: Send + Sync {
    async fn prepare_package_upload(
        &self,
        credentials: &DriveRequestCredentials,
        command: PrepareDeployUploadCommand,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn retrieve_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn complete_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
        request: &CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn cancel_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;
}

#[derive(Clone)]
pub enum DeployDrivePortAdapter {
    Memory(MemoryDeployDrivePort),
    Facade(SdkDriveAppFacade),
    Unconfigured,
}

#[async_trait]
impl DeployDrivePort for DeployDrivePortAdapter {
    async fn prepare_package_upload(
        &self,
        credentials: &DriveRequestCredentials,
        command: PrepareDeployUploadCommand,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        match self {
            Self::Memory(port) => port.prepare_package_upload(credentials, command).await,
            Self::Facade(port) => port.prepare_package_upload(credentials, command).await,
            Self::Unconfigured => Err(DeployServiceError::Internal(
                "Drive facade is not configured; set SDKWORK_DRIVE_FACADE_URL and SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false"
                    .into(),
            )),
        }
    }

    async fn retrieve_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        match self {
            Self::Memory(port) => {
                port.retrieve_upload_session(credentials, drive_session_id)
                    .await
            }
            Self::Facade(port) => {
                port.retrieve_upload_session(credentials, drive_session_id)
                    .await
            }
            Self::Unconfigured => Err(DeployServiceError::Internal(
                "Drive facade is not configured; set SDKWORK_DRIVE_FACADE_URL and SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false"
                    .into(),
            )),
        }
    }

    async fn complete_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
        request: &CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        match self {
            Self::Memory(port) => {
                port.complete_upload_session(credentials, drive_session_id, request)
                    .await
            }
            Self::Facade(port) => {
                port.complete_upload_session(credentials, drive_session_id, request)
                    .await
            }
            Self::Unconfigured => Err(DeployServiceError::Internal(
                "Drive facade is not configured; set SDKWORK_DRIVE_FACADE_URL and SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false"
                    .into(),
            )),
        }
    }

    async fn cancel_upload_session(
        &self,
        credentials: &DriveRequestCredentials,
        drive_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        match self {
            Self::Memory(port) => {
                port.cancel_upload_session(credentials, drive_session_id)
                    .await
            }
            Self::Facade(port) => {
                port.cancel_upload_session(credentials, drive_session_id)
                    .await
            }
            Self::Unconfigured => Err(DeployServiceError::Internal(
                "Drive facade is not configured; set SDKWORK_DRIVE_FACADE_URL and SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false"
                    .into(),
            )),
        }
    }
}
