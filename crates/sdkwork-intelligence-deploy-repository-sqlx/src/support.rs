use chrono::{SecondsFormat, Utc};
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::DeployServiceError;
use sqlx::any::AnyRow;
use sqlx::{AnyPool, Error as SqlxError, Row};

pub(crate) fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub(crate) fn store_error(context: &str, error: SqlxError) -> DeployServiceError {
    tracing::error!("{context}: {error}");
    match error {
        SqlxError::Database(db) if db.is_unique_violation() => {
            DeployServiceError::conflict(db.message())
        }
        SqlxError::RowNotFound => DeployServiceError::not_found("resource not found"),
        _ => DeployServiceError::Internal(format!("{context}: {error}")),
    }
}

pub(crate) fn pagination(page: i32, page_size: i32) -> (i32, i32, i64) {
    let (page, page_size) = sdkwork_deploy_core::normalize_pagination(page, page_size);
    let offset = sdkwork_deploy_core::pagination_offset(page, page_size);
    (page, page_size, offset)
}

pub(crate) fn next_id(generator: &SnowflakeIdGenerator) -> Result<i64, DeployServiceError> {
    generator
        .generate()
        .map_err(|error| DeployServiceError::Internal(error.to_string()))
}

pub(crate) fn new_uuid() -> String {
    sdkwork_database_id::uuid_v4()
}

pub(crate) fn sha256_hex(content: &str) -> String {
    sdkwork_utils_rust::sha256_hash(content.as_bytes())
}

pub(crate) fn bool_from_row(row: &AnyRow, column: &str) -> Result<bool, SqlxError> {
    if let Ok(value) = row.try_get::<bool, _>(column) {
        return Ok(value);
    }
    let value: i64 = row.try_get(column)?;
    Ok(value != 0)
}

pub(crate) fn json_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<Option<serde_json::Value>, SqlxError> {
    let raw: Option<String> = row.try_get(column)?;
    Ok(raw.and_then(|text| serde_json::from_str(&text).ok()))
}

pub(crate) async fn resolve_site_internal_id(
    pool: &AnyPool,
    tenant_id: i64,
    site_uuid: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_site
         WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(site_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve deploy_site id", error))?;

    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("site not found"))
}

pub(crate) async fn resolve_site_uuid(
    pool: &AnyPool,
    tenant_id: i64,
    site_internal_id: i64,
) -> Result<String, DeployServiceError> {
    let row = sqlx::query(
        "SELECT uuid FROM deploy_site
         WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(site_internal_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve deploy_site uuid", error))?;

    row.and_then(|row| row.try_get::<String, _>("uuid").ok())
        .ok_or_else(|| DeployServiceError::not_found("site not found"))
}

pub(crate) async fn resolve_domain_internal_id(
    pool: &AnyPool,
    tenant_id: i64,
    domain_uuid: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_domain
         WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(domain_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve deploy_domain id", error))?;

    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("domain not found"))
}

pub(crate) async fn resolve_artifact_internal_id(
    pool: &AnyPool,
    tenant_id: i64,
    artifact_uuid: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_artifact
         WHERE tenant_id = $1 AND uuid = $2 AND status <> 2",
    )
    .bind(tenant_id)
    .bind(artifact_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve deploy_artifact id", error))?;

    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("artifact not found"))
}

pub(crate) async fn resolve_release_internal_id(
    pool: &AnyPool,
    tenant_id: i64,
    site_internal_id: i64,
    release_uuid: &str,
) -> Result<i64, DeployServiceError> {
    let row = sqlx::query(
        "SELECT id FROM deploy_release
         WHERE tenant_id = $1 AND site_id = $2 AND uuid = $3 AND status = 1",
    )
    .bind(tenant_id)
    .bind(site_internal_id)
    .bind(release_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|error| store_error("resolve deploy_release id", error))?;

    row.and_then(|row| row.try_get::<i64, _>("id").ok())
        .ok_or_else(|| DeployServiceError::not_found("release not found"))
}
