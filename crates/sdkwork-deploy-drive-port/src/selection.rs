use std::sync::Arc;

use sdkwork_utils_rust::{parse_bool, string::trim};

use crate::{DeployDrivePort, DeployDrivePortAdapter, MemoryDeployDrivePort, SdkDriveAppFacade};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployDrivePortSelectionInput {
    pub use_memory_drive: Option<bool>,
    pub facade_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployDrivePortSelection {
    Memory,
    Facade { facade_url: String },
    Unconfigured,
}

pub fn select_deploy_drive_port(input: DeployDrivePortSelectionInput) -> DeployDrivePortSelection {
    if input.use_memory_drive == Some(false) {
        if let Some(facade_url) = input
            .facade_url
            .as_ref()
            .map(|value| trim(value))
            .filter(|value| !value.is_empty())
        {
            return DeployDrivePortSelection::Facade { facade_url };
        }
        return DeployDrivePortSelection::Unconfigured;
    }
    DeployDrivePortSelection::Memory
}

pub fn deploy_drive_port_from_env() -> Result<Arc<dyn DeployDrivePort>, String> {
    let use_memory_drive = std::env::var("SDKWORK_DEPLOY_USE_MEMORY_DRIVE")
        .ok()
        .and_then(|value| parse_bool(&value));
    let facade_url = std::env::var("SDKWORK_DRIVE_FACADE_URL").ok();
    let adapter = match select_deploy_drive_port(DeployDrivePortSelectionInput {
        use_memory_drive,
        facade_url,
    }) {
        DeployDrivePortSelection::Memory => {
            DeployDrivePortAdapter::Memory(MemoryDeployDrivePort::default())
        }
        DeployDrivePortSelection::Facade { facade_url } => {
            DeployDrivePortAdapter::Facade(SdkDriveAppFacade::from_env(facade_url)?)
        }
        DeployDrivePortSelection::Unconfigured => DeployDrivePortAdapter::Unconfigured,
    };
    Ok(std::sync::Arc::new(adapter) as std::sync::Arc<dyn DeployDrivePort>)
}
