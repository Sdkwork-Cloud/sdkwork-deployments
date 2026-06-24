pub(crate) fn validate_nginx_config_content(content: &str) -> Result<(), String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("config content is empty".to_string());
    }

    let forbidden = ["lua_", "perl ", "perl\t", "/etc/passwd", "autoindex on;"];
    for pattern in forbidden {
        if trimmed.contains(pattern) {
            return Err(format!("forbidden nginx pattern: {pattern}"));
        }
    }

    if !trimmed.contains("server {") && !trimmed.contains("server{") {
        return Err("config must contain at least one server block".to_string());
    }

    Ok(())
}

pub(crate) fn nginx_reload_enabled() -> bool {
    matches!(
        std::env::var("SDKWORK_DEPLOY_NGINX_RELOAD").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

pub(crate) fn run_nginx_reload_command() -> Result<(), String> {
    let command = std::env::var("SDKWORK_DEPLOY_NGINX_RELOAD_CMD")
        .unwrap_or_else(|_| "nginx -s reload".to_string());
    run_shell_command(&command)
}

fn run_shell_command(command: &str) -> Result<(), String> {
    let output = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", command])
            .output()
    } else {
        std::process::Command::new("sh")
            .args(["-c", command])
            .output()
    }
    .map_err(|error| format!("failed to execute nginx command: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "nginx command failed: {}{}",
        stdout.trim(),
        if stderr.trim().is_empty() {
            String::new()
        } else {
            format!("\n{stderr}")
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::validate_nginx_config_content;

    #[test]
    fn rejects_empty_config() {
        assert!(validate_nginx_config_content("   ").is_err());
    }

    #[test]
    fn rejects_forbidden_lua() {
        assert!(validate_nginx_config_content("server { lua_xxx; }").is_err());
    }

    #[test]
    fn accepts_basic_server_block() {
        assert!(validate_nginx_config_content("server { listen 80; }").is_ok());
    }
}
