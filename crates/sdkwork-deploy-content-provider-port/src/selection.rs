use std::sync::Arc;

use sdkwork_utils_rust::parse_bool;

use crate::{ContentProviderPort, MemoryContentProviderPort, SdkContentProviderPort};

pub const USE_MEMORY_CONTENT_PROVIDER_ENV: &str = "SDKWORK_DEPLOY_USE_MEMORY_CONTENT_PROVIDER";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContentProviderPortSelection {
    Memory,
    Sdk,
}

fn select_content_provider_port(
    production_like: bool,
    use_memory: Option<bool>,
) -> Result<ContentProviderPortSelection, String> {
    if production_like && use_memory != Some(false) {
        return Err(format!(
            "production content providers require {USE_MEMORY_CONTENT_PROVIDER_ENV}=false"
        ));
    }
    if use_memory == Some(false) {
        Ok(ContentProviderPortSelection::Sdk)
    } else {
        Ok(ContentProviderPortSelection::Memory)
    }
}

pub fn content_provider_port_from_env() -> Result<Arc<dyn ContentProviderPort>, String> {
    let production_like = sdkwork_deploy_core::deploy_is_production_like_environment();
    let use_memory = std::env::var(USE_MEMORY_CONTENT_PROVIDER_ENV)
        .ok()
        .and_then(|value| parse_bool(&value));
    match select_content_provider_port(production_like, use_memory)? {
        ContentProviderPortSelection::Memory => Ok(Arc::new(MemoryContentProviderPort)),
        ContentProviderPortSelection::Sdk => Ok(Arc::new(SdkContentProviderPort::from_env()?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_selection_fails_closed_without_explicit_sdk_mode() {
        for use_memory in [None, Some(true)] {
            assert!(select_content_provider_port(true, use_memory).is_err());
        }
        assert_eq!(
            select_content_provider_port(true, Some(false)).unwrap(),
            ContentProviderPortSelection::Sdk
        );
    }

    #[test]
    fn development_defaults_to_memory_but_can_select_sdk_mode() {
        assert_eq!(
            select_content_provider_port(false, None).unwrap(),
            ContentProviderPortSelection::Memory
        );
        assert_eq!(
            select_content_provider_port(false, Some(false)).unwrap(),
            ContentProviderPortSelection::Sdk
        );
    }
}
