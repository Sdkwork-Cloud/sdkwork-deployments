use std::sync::Arc;

use sdkwork_utils_rust::parse_bool;

use crate::{ContentProviderPort, MemoryContentProviderPort, SdkContentProviderPort};

pub const USE_MEMORY_CONTENT_PROVIDER_ENV: &str = "SDKWORK_DEPLOY_USE_MEMORY_CONTENT_PROVIDER";

pub fn content_provider_port_from_env() -> Result<Arc<dyn ContentProviderPort>, String> {
    let production_like = sdkwork_deploy_core::deploy_is_production_like_environment();
    let use_memory = std::env::var(USE_MEMORY_CONTENT_PROVIDER_ENV)
        .ok()
        .and_then(|value| parse_bool(&value));
    if production_like && use_memory != Some(false) {
        return Err(format!(
            "production content providers require {USE_MEMORY_CONTENT_PROVIDER_ENV}=false"
        ));
    }
    if use_memory == Some(false) {
        return Ok(Arc::new(SdkContentProviderPort::from_env()?));
    }
    Ok(Arc::new(MemoryContentProviderPort))
}
