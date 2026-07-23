//! Provider SDK adapters for live Drive websites and Knowledgebase Wiki publications.

mod memory;
mod sdk;
mod selection;
mod website_event_delivery;

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    ContentProviderResourceSource, DeployServiceResult, SiteResourceDefinition,
};
use sdkwork_deploy_runtime_compiler::{RuntimeProviderType, RuntimeResourceCapabilities};

pub use memory::MemoryContentProviderPort;
pub use sdk::SdkContentProviderPort;
pub use selection::{
    content_provider_port_from_env, website_provider_event_delivery_port_from_env,
};
pub use website_event_delivery::{
    NoopWebsiteProviderEventDeliveryPort, SdkWebsiteProviderEventDeliveryPort,
    WebsiteProviderEventDeliveryConfig, WebsiteProviderEventDeliveryPort,
    WebsiteProviderEventDeliveryResult,
};

#[derive(Clone, Debug, Default)]
pub struct ProviderRequestCredentials {
    pub auth_token: Option<String>,
    pub access_token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ValidateContentProviderResourceCommand {
    pub tenant_id: i64,
    pub site_uuid: String,
    pub resource: SiteResourceDefinition,
}

#[derive(Clone, Debug)]
pub struct ValidatedContentProviderResource {
    pub key: String,
    pub source: ContentProviderResourceSource,
    pub provider_type: RuntimeProviderType,
    pub provider_resource_uuid: String,
    pub provider_contract_version: String,
    pub capabilities: RuntimeResourceCapabilities,
}

#[async_trait]
pub trait ContentProviderPort: Send + Sync {
    async fn validate_resource(
        &self,
        credentials: &ProviderRequestCredentials,
        command: ValidateContentProviderResourceCommand,
    ) -> DeployServiceResult<ValidatedContentProviderResource>;
}
