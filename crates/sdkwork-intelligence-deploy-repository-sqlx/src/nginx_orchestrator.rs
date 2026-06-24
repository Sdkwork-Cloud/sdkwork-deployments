use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::nginx_security::{
    nginx_reload_enabled, run_nginx_reload_command, validate_nginx_config_content,
};

const RUNTIME_BINDING_KEY: &str = "sdkworkDeploy";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdkworkDeployBinding {
    pub app_root: PathBuf,
    pub domain: String,
    pub profile_id: Option<String>,
    pub site_file: Option<PathBuf>,
}

pub fn parse_sdkwork_deploy_binding(
    runtime_config: &Value,
    fallback_domain: Option<&str>,
) -> Option<SdkworkDeployBinding> {
    let binding = runtime_config.get(RUNTIME_BINDING_KEY)?;
    let app_root = binding
        .get("appRoot")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)?;
    let domain = binding
        .get("domain")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| fallback_domain.map(str::to_string))?;
    let profile_id = binding
        .get("profileId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let site_file = binding
        .get("siteFile")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    Some(SdkworkDeployBinding {
        app_root,
        domain,
        profile_id,
        site_file,
    })
}

pub fn orchestration_enabled() -> bool {
    !matches!(
        std::env::var("SDKWORK_DEPLOY_ORCHESTRATE_NGINX").as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE")
    )
}

pub fn resolve_deployctl_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("SDKWORK_DEPLOY_DEPLOYCTL") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(spec_root) = std::env::var("SDKWORK_DEPLOY_SPEC_ROOT") {
        let candidate = PathBuf::from(spec_root).join("tools/deployctl.mjs");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    if let Ok(app_root) = std::env::var("SDKWORK_DEPLOY_APP_ROOT") {
        let candidate = PathBuf::from(app_root).join("../sdkwork-specs/tools/deployctl.mjs");
        if candidate.is_file() {
            return candidate
                .canonicalize()
                .map_err(|error| format!("resolve deployctl path failed: {error}"));
        }
    }

    Err("deployctl not found; set SDKWORK_DEPLOY_DEPLOYCTL or SDKWORK_DEPLOY_SPEC_ROOT".to_string())
}

pub fn run_deployctl_nginx_apply(binding: &SdkworkDeployBinding) -> Result<(), String> {
    if !binding.app_root.join("deployments/deploy.yaml").is_file() {
        return Err(format!(
            "missing deployments/deploy.yaml under {}",
            binding.app_root.display()
        ));
    }

    let deployctl = resolve_deployctl_path()?;
    let node = std::env::var("SDKWORK_NODE_BIN").unwrap_or_else(|_| "node".to_string());
    let mut command = std::process::Command::new(node);
    command.arg(deployctl);
    command.args([
        "nginx",
        "apply",
        "--root",
        binding.app_root.to_string_lossy().as_ref(),
        "--domain",
        &binding.domain,
    ]);
    if let Some(profile_id) = binding.profile_id.as_deref() {
        command.args(["--profile", profile_id]);
    }
    if nginx_reload_enabled() {
        command.env("SDKWORK_DEPLOY_NGINX_RELOAD", "true");
    }
    if let Some(site_file) = binding.site_file.as_ref() {
        command.env(
            "SDKWORK_DEPLOY_NGINX_SITE_FILE",
            site_file.to_string_lossy().as_ref(),
        );
    }

    let output = command
        .output()
        .map_err(|error| format!("failed to spawn deployctl nginx apply: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "deployctl nginx apply failed: {}{}",
        stdout.trim(),
        if stderr.trim().is_empty() {
            String::new()
        } else {
            format!("\n{stderr}")
        }
    ))
}

pub fn default_site_file(domain: &str) -> PathBuf {
    PathBuf::from(format!("/etc/nginx/sites-enabled/sdkwork/{domain}.conf"))
}

pub fn apply_stored_nginx_config(
    content: &str,
    site_file: &Path,
) -> Result<Option<PathBuf>, String> {
    validate_nginx_config_content(content)?;

    if let Some(parent) = site_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create nginx site directory failed: {error}"))?;
    }

    let backup_path = site_file.with_extension("conf.bak");
    let mut backup_created = None;
    if site_file.is_file() {
        std::fs::copy(site_file, &backup_path)
            .map_err(|error| format!("backup nginx site file failed: {error}"))?;
        backup_created = Some(backup_path);
    }

    std::fs::write(site_file, content)
        .map_err(|error| format!("write nginx site file failed: {error}"))?;

    Ok(backup_created)
}

pub fn publish_nginx_config(
    content: &str,
    binding: Option<&SdkworkDeployBinding>,
    fallback_domain: Option<&str>,
) -> Result<(), String> {
    validate_nginx_config_content(content)?;

    if orchestration_enabled() {
        if let Some(binding) = binding {
            if run_deployctl_nginx_apply(binding).is_ok() {
                return Ok(());
            }
        }
    }

    let site_file = std::env::var("SDKWORK_DEPLOY_NGINX_SITE_FILE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| binding.and_then(|item| item.site_file.clone()))
        .or_else(|| {
            binding
                .map(|item| item.domain.as_str())
                .or(fallback_domain)
                .map(default_site_file)
        })
        .ok_or_else(|| "nginx site file target could not be resolved".to_string())?;

    apply_stored_nginx_config(content, &site_file)?;
    if nginx_reload_enabled() {
        run_nginx_reload_command()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{default_site_file, parse_sdkwork_deploy_binding};
    use serde_json::json;

    #[test]
    fn parses_runtime_binding_with_fallback_domain() {
        let runtime = json!({
            "sdkworkDeploy": {
                "appRoot": "/usr/share/sdkwork-space/sdkwork-im",
                "profileId": "cloud.split-services.production"
            }
        });
        let binding = parse_sdkwork_deploy_binding(&runtime, Some("im.sdkwork.com")).unwrap();
        assert_eq!(binding.domain, "im.sdkwork.com");
        assert_eq!(
            binding.profile_id.as_deref(),
            Some("cloud.split-services.production")
        );
    }

    #[test]
    fn default_site_file_matches_spec() {
        assert_eq!(
            default_site_file("im.sdkwork.com"),
            PathBuf::from("/etc/nginx/sites-enabled/sdkwork/im.sdkwork.com.conf")
        );
    }
}
