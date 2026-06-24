//! Deploy runtime environment helpers shared by routers and the API server.

use sdkwork_utils_rust::parse_bool;

static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[doc(hidden)]
pub fn env_test_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn deploy_environment_name() -> String {
    std::env::var("SDKWORK_DEPLOY_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_DEPLOY_CONFIG_PROFILE"))
        .unwrap_or_else(|_| "development".to_string())
        .to_ascii_lowercase()
}

pub fn deploy_is_production_like_environment() -> bool {
    matches!(
        deploy_environment_name().as_str(),
        "production" | "prod" | "staging" | "stage" | "test"
    )
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false)
}

pub fn deploy_dev_auth_bypass_enabled() -> bool {
    env_truthy("SDKWORK_DEPLOY_DEV_AUTH_BYPASS")
}

pub fn deploy_use_dev_inline_auth_resolver() -> bool {
    !deploy_is_production_like_environment() && deploy_dev_auth_bypass_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env(key: &str, value: Option<&str>, test: impl FnOnce()) {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        test();
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn production_never_uses_dev_inline_auth() {
        let _guard = env_test_lock();
        with_env("SDKWORK_DEPLOY_ENVIRONMENT", Some("production"), || {
            with_env("SDKWORK_DEPLOY_DEV_AUTH_BYPASS", Some("true"), || {
                assert!(!deploy_use_dev_inline_auth_resolver());
            });
        });
    }
}
