//! Deploy business service orchestrating repository ports and HTTP API traits.

pub mod app;
pub mod app_composition;
pub mod app_delivery;
pub mod app_domains;
pub mod backend;
pub mod domain_verification;
pub mod entitlement;
pub mod repository;
pub mod runtime_publication;

pub use app_composition::{AppCompositionRepositoryPort, ReplaceAppCompositionCommand};
pub use domain_verification::{
    dns_txt_record_name, normalize_domain_hostname, normalize_zone_apex,
    DomainOwnershipVerifierPort, DomainVerificationChallenge, DomainVerificationObservation,
    UnconfiguredDomainOwnershipVerifier,
};
pub use repository::DeployRepositoryPort;
pub use runtime_publication::{
    DeployRuntimeAssignmentMutationPort, DeployRuntimeAssignmentRepositoryPort,
    RuntimeObservationEvidence, RuntimeObservationPersistenceResult, RuntimeObservationState,
    RuntimePublicationBatchResult, RuntimePublicationService,
};

use std::sync::Arc;

use sdkwork_deploy_content_provider_port::{ContentProviderPort, MemoryContentProviderPort};
use sdkwork_deploy_contract::DeployServiceResult;
use sdkwork_deploy_drive_port::DeployDrivePort;

/// Application service for SDKWork Deploy control plane operations.
pub struct DeployService {
    pub(crate) repository: Arc<dyn DeployRepositoryPort>,
    pub(crate) drive: Arc<dyn DeployDrivePort>,
    pub(crate) content_provider: Arc<dyn ContentProviderPort>,
    pub(crate) domain_ownership_verifier: Arc<dyn DomainOwnershipVerifierPort>,
    runtime_publication: Option<Arc<RuntimePublicationService>>,
}

impl DeployService {
    pub fn new(repository: Arc<dyn DeployRepositoryPort>, drive: Arc<dyn DeployDrivePort>) -> Self {
        Self {
            repository,
            drive,
            content_provider: Arc::new(MemoryContentProviderPort),
            domain_ownership_verifier: Arc::new(UnconfiguredDomainOwnershipVerifier),
            runtime_publication: None,
        }
    }

    pub fn new_with_runtime_publication(
        repository: Arc<dyn DeployRepositoryPort>,
        drive: Arc<dyn DeployDrivePort>,
        content_provider: Arc<dyn ContentProviderPort>,
        domain_ownership_verifier: Arc<dyn DomainOwnershipVerifierPort>,
        runtime_publication: Arc<RuntimePublicationService>,
    ) -> Self {
        Self {
            repository,
            drive,
            content_provider,
            domain_ownership_verifier,
            runtime_publication: Some(runtime_publication),
        }
    }

    pub fn runtime_publication(&self) -> Option<&Arc<RuntimePublicationService>> {
        self.runtime_publication.as_ref()
    }

    pub async fn ready_check(&self) -> DeployServiceResult<()> {
        self.repository.ready_check().await
    }
}
