//! Application database structure contract rules (REQ-2026-0002 extension):
//! engine/catalog/migration-definition validation that binds an app release
//! to its declared data structure.

pub const DATABASE_ENGINES: &[&str] = &["POSTGRES", "MYSQL", "SQLITE"];
pub const MIGRATION_STRATEGIES: &[&str] = &["VERSIONED", "REPEATABLE"];
pub const PROFILE_STATUSES: &[&str] = &["DRAFT", "READY", "ACTIVE", "SUPERSEDED", "ARCHIVED"];
pub const MIGRATION_STATUSES: &[&str] = &["PENDING", "APPLIED", "FAILED", "SUPERSEDED"];

/// Validates the declared database engine.
pub fn validate_database_engine(engine: &str) -> Result<(), String> {
    if DATABASE_ENGINES.contains(&engine) {
        Ok(())
    } else {
        Err(format!(
            "dbEngine must be one of {}",
            DATABASE_ENGINES.join(", ")
        ))
    }
}

/// Validates the migration strategy.
pub fn validate_migration_strategy(strategy: &str) -> Result<(), String> {
    if MIGRATION_STRATEGIES.contains(&strategy) {
        Ok(())
    } else {
        Err(format!(
            "migrationStrategy must be one of {}",
            MIGRATION_STRATEGIES.join(", ")
        ))
    }
}

/// Validates a profile status transition value.
pub fn validate_profile_status(status: &str) -> Result<(), String> {
    if PROFILE_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(format!(
            "profileStatus must be one of {}",
            PROFILE_STATUSES.join(", ")
        ))
    }
}

/// Validates the catalog/schema name (bounded identifier, no quotes).
pub fn validate_catalog_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 {
        return Err("catalogName must be 1..=128 characters".into());
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err("catalogName contains invalid characters".into());
    }
    Ok(())
}

/// Validates a migration version (numeric dotted or plain, 1..=64 chars).
pub fn validate_migration_version(version: &str) -> Result<(), String> {
    if version.is_empty() || version.len() > 64 {
        return Err("migrationVersion must be 1..=64 characters".into());
    }
    if !version
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err("migrationVersion contains invalid characters".into());
    }
    if version.starts_with('.') || version.ends_with('.') {
        return Err("migrationVersion must not start or end with '.'".into());
    }
    Ok(())
}

/// Validates a migration name (bounded, no path traversal).
pub fn validate_migration_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 200 {
        return Err("migrationName must be 1..=200 characters".into());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("migrationName must not contain path separators".into());
    }
    Ok(())
}

/// Validates a profile key (slug style).
pub fn validate_profile_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 120 {
        return Err("profileKey must be 1..=120 characters".into());
    }
    if !key
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err("profileKey must contain only letters, digits, '_' or '-'".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_engine_is_validated() {
        assert!(validate_database_engine("POSTGRES").is_ok());
        assert!(validate_database_engine("MYSQL").is_ok());
        assert!(validate_database_engine("SQLITE").is_ok());
        assert!(validate_database_engine("ORACLE").is_err());
        assert!(validate_database_engine("").is_err());
    }

    #[test]
    fn migration_strategy_is_validated() {
        assert!(validate_migration_strategy("VERSIONED").is_ok());
        assert!(validate_migration_strategy("REPEATABLE").is_ok());
        assert!(validate_migration_strategy("TIMESTAMPED").is_err());
    }

    #[test]
    fn migration_version_rules_apply() {
        assert!(validate_migration_version("0001").is_ok());
        assert!(validate_migration_version("1.2.3").is_ok());
        assert!(validate_migration_version("20260804_001").is_ok());
        assert!(validate_migration_version(".1").is_err());
        assert!(validate_migration_version("1.").is_err());
        assert!(validate_migration_version("a/b").is_err());
        assert!(validate_migration_version(&"x".repeat(65)).is_err());
    }

    #[test]
    fn catalog_and_names_are_bounded() {
        assert!(validate_catalog_name("sdkwork_app_prod").is_ok());
        assert!(validate_catalog_name("app-prod.v1").is_ok());
        assert!(validate_catalog_name("bad;drop").is_err());
        assert!(validate_catalog_name(&"x".repeat(129)).is_err());
        assert!(validate_profile_key("primary").is_ok());
        assert!(validate_profile_key("a b").is_err());
        assert!(validate_migration_name("create_users_table").is_ok());
        assert!(validate_migration_name("../escape").is_err());
    }
}
