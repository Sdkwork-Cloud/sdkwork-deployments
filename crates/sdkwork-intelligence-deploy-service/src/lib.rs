//! Deploy business service orchestrating repository ports and HTTP API traits.

pub mod app;
pub mod backend;
pub mod repository;

pub use repository::DeployRepositoryPort;

use std::sync::Arc;

use sdkwork_deploy_contract::DeployServiceResult;

/// Application service for SDKWork Deploy control plane operations.
pub struct DeployService {
    pub(crate) repository: Arc<dyn DeployRepositoryPort>,
}

impl DeployService {
    pub fn new(repository: Arc<dyn DeployRepositoryPort>) -> Self {
        Self { repository }
    }

    pub async fn ready_check(&self) -> DeployServiceResult<()> {
        self.repository.ready_check().await
    }
}
