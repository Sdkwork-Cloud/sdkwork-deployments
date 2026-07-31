use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Executor, PgPool};

const POSTGRES_BASELINE: &str =
    include_str!("../../../../database/ddl/baseline/postgres/0001_deploy_baseline.sql");

pub async fn postgres_pool() -> PgPool {
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
    admin_pool
        .execute(format!("CREATE SCHEMA {schema}").as_str())
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
                connection
                    .execute(format!("SET search_path TO {schema}").as_str())
                    .await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
        .expect("connect isolated PostgreSQL test schema");
    sqlx::raw_sql(POSTGRES_BASELINE)
        .execute(&pool)
        .await
        .expect("apply PostgreSQL baseline");
    pool
}
