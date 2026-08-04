use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, PgPool};

/// Deterministic AES-256 key for repository integration tests. Production
/// derives the key from `SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY`; tests only
/// exercise encryption round-trips and response masking, so a fixed key is
/// sufficient and keeps assertions stable.
#[allow(dead_code)]
pub fn test_secret_key() -> [u8; 32] {
    *b"sdkwork-deploy-test-secret-00000"
}

const POSTGRES_BASELINE: &str =
    include_str!("../../../../database/ddl/baseline/postgres/0001_deploy_baseline.sql");

// `postgres_pool` is only used by the shared-module tests; upgrade tests use
// `postgres_schema_pool` directly, so silence the per-test dead-code warning.
#[allow(dead_code)]
pub async fn postgres_pool() -> PgPool {
    let pool = postgres_schema_pool().await;
    sqlx::raw_sql(POSTGRES_BASELINE)
        .execute(&pool)
        .await
        .expect("apply PostgreSQL baseline");
    pool
}

/// Creates an isolated random test schema and returns a pool pinned to it
/// without applying any baseline DDL. Callers apply the baseline or legacy
/// fixtures themselves (for example database upgrade tests).
pub async fn postgres_schema_pool() -> PgPool {
    let database_url = std::env::var("SDKWORK_DATABASE_TEST_POSTGRES_URL")
        .expect("SDKWORK_DATABASE_TEST_POSTGRES_URL is required for PostgreSQL integration tests");
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect PostgreSQL integration database");
    let schema = format!(
        "sdkwork_deploy_test_{}",
        sdkwork_database_id::uuid_v4().replace('-', "")
    );
    // schema is derived from a random UUID; the assertion is an audit of that derivation
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
        .execute(&admin_pool)
        .await
        .expect("create isolated PostgreSQL test schema");
    admin_pool.close().await;

    let connect_options = PgConnectOptions::from_str(&database_url)
        .expect("parse PostgreSQL integration database URL");
    let connection_schema = schema.clone();
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .after_connect(move |connection, _metadata| {
            let schema = connection_schema.clone();
            Box::pin(async move {
                sqlx::raw_sql(AssertSqlSafe(format!("SET search_path TO {schema}")))
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
        .expect("connect isolated PostgreSQL test schema");
    pool
}
