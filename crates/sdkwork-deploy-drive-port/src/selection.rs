use std::sync::Arc;

use sdkwork_utils_rust::{parse_bool, string::trim};

use crate::{DeployDrivePort, DeployDrivePortAdapter, MemoryDeployDrivePort, SdkDriveAppFacade};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployDrivePortSelectionInput {
    pub use_memory_drive: Option<bool>,
    pub facade_url: Option<String>,
    pub production_like: bool,
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
    if input.production_like {
        return DeployDrivePortSelection::Unconfigured;
    }
    DeployDrivePortSelection::Memory
}

pub fn deploy_drive_port_from_env() -> Result<Arc<dyn DeployDrivePort>, String> {
    let use_memory_drive = std::env::var("SDKWORK_DEPLOY_USE_MEMORY_DRIVE")
        .ok()
        .and_then(|value| parse_bool(&value));
    let facade_url = std::env::var("SDKWORK_DRIVE_FACADE_URL").ok();
    let production_like = sdkwork_deploy_core::deploy_is_production_like_environment();
    let adapter = match select_deploy_drive_port(DeployDrivePortSelectionInput {
        use_memory_drive,
        facade_url,
        production_like,
    }) {
        DeployDrivePortSelection::Memory => DeployDrivePortAdapter::Memory(MemoryDeployDrivePort),
        DeployDrivePortSelection::Facade { facade_url } => {
            DeployDrivePortAdapter::Facade(SdkDriveAppFacade::from_env(facade_url)?)
        }
        DeployDrivePortSelection::Unconfigured if production_like => {
            return Err(
                "production Drive requires SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false and SDKWORK_DRIVE_FACADE_URL"
                    .to_owned(),
            );
        }
        DeployDrivePortSelection::Unconfigured => DeployDrivePortAdapter::Unconfigured,
    };
    Ok(std::sync::Arc::new(adapter) as std::sync::Arc<dyn DeployDrivePort>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_never_selects_memory_drive() {
        assert_eq!(
            select_deploy_drive_port(DeployDrivePortSelectionInput {
                use_memory_drive: Some(true),
                facade_url: None,
                production_like: true,
            }),
            DeployDrivePortSelection::Unconfigured
        );
        assert_eq!(
            select_deploy_drive_port(DeployDrivePortSelectionInput {
                use_memory_drive: None,
                facade_url: None,
                production_like: true,
            }),
            DeployDrivePortSelection::Unconfigured
        );
    }

    #[test]
    fn production_selects_configured_drive_facade() {
        assert_eq!(
            select_deploy_drive_port(DeployDrivePortSelectionInput {
                use_memory_drive: Some(false),
                facade_url: Some("https://api.sdkwork.com".to_owned()),
                production_like: true,
            }),
            DeployDrivePortSelection::Facade {
                facade_url: "https://api.sdkwork.com".to_owned(),
            }
        );
    }
}
