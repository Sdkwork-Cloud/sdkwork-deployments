//! Deploy business service orchestrating repository ports and HTTP API traits.

pub mod app;
pub mod backend;
pub mod repository;

pub use repository::DeployRepositoryPort;

use std::sync::Arc;

use sdkwork_deploy_contract::DeployServiceResult;
use sdkwork_deploy_drive_port::DeployDrivePort;

/// Application service for SDKWork Deploy control plane operations.
pub struct DeployService {
    pub(crate) repository: Arc<dyn DeployRepositoryPort>,
    pub(crate) drive: Arc<dyn DeployDrivePort>,
}

impl DeployService {
    pub fn new(repository: Arc<dyn DeployRepositoryPort>, drive: Arc<dyn DeployDrivePort>) -> Self {
        Self { repository, drive }
    }

    pub async fn ready_check(&self) -> DeployServiceResult<()> {
        self.repository.ready_check().await
    }
}
