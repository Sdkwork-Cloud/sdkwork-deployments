//! Database init/upgrade integration tests (DATABASE_FRAMEWORK_SPEC §7.4,
//! TEST_SPEC §2.0.2.1): databases initialized from earlier consolidated
//! baselines must converge to the current deploy contract through the
//! lifecycle orchestrator's init + migrate pipeline, and drift must be clean.
//!
//! Requires `SDKWORK_DATABASE_TEST_POSTGRES_URL`; tests are ignored by
//! default like the other PostgreSQL integration tests in this crate.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_drift::DriftEngine;
use sdkwork_database_lifecycle::LifecycleOrchestrator;
use sdkwork_database_spi::DefaultDatabaseModule;
use sdkwork_database_sqlx::DatabasePool;
use sqlx::PgPool;

/// Consolidated baseline snapshot as of 2026-06-24 (first 10 tables with the
/// legacy site-bound `deploy_domain` / path-based `deploy_certificate`
/// shapes). Databases initialized from it (or any later pre-2026-07-31
/// snapshot) are exactly the upgrade path the forward migrations serve.
const LEGACY_BASELINE: &str = include_str!("fixtures/legacy_deploy_baseline.sql");

/// Every table of the current deploy contract (contract/schema.yaml).
const CONTRACT_TABLES: &[&str] = &[
    "deploy_acme_account",
    "deploy_artifact",
    "deploy_audit_log",
    "deploy_certificate",
    "deploy_certificate_challenge",
    "deploy_certificate_distribution",
    "deploy_certificate_identifier",
    "deploy_certificate_order",
    "deploy_certificate_version",
    "deploy_deployment",
    "deploy_dns_zone",
    "deploy_domain",
    "deploy_domain_verification",
    "deploy_env_variable",
    "deploy_health_check",
    "deploy_health_result",
    "deploy_listener_certificate_binding",
    "deploy_nginx_config",
    "deploy_node_cluster",
    "deploy_release",
    "deploy_runtime_assignment",
    "deploy_server",
    "deploy_site",
    "deploy_site_binding",
    "deploy_site_mount",
    "deploy_site_resource",
    "deploy_site_revision",
    "deploy_site_target_observation",
    "deploy_site_variant",
    "deploy_site_variant_rule",
    "deploy_tls_policy",
    "deploy_tls_runtime_assignment",
    "deploy_tls_runtime_snapshot",
    "deploy_tls_target_observation",
    "deploy_upload_session_ref",
    "deploy_web_node_target",
    "deploy_app",
    "deploy_app_platform_target",
    "deploy_source_repository",
    "deploy_build_template",
    "deploy_build",
    "deploy_package",
    "deploy_release_channel",
    "deploy_channel_rollout",
    "deploy_signing_identity",
    "deploy_usage_event",
    "deploy_tenant_entitlement_projection",
    "deploy_site_usage_daily",
    "deploy_app_database_profile",
    "deploy_app_database_migration",
];

/// The deploy module lives at the sdkwork-deployments repository root.
fn deploy_module() -> Arc<DefaultDatabaseModule> {
    let app_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    Arc::new(DefaultDatabaseModule::from_app_root(&app_root).expect("load deploy database module"))
}

fn database_pool(pool: PgPool) -> DatabasePool {
    DatabasePool::Postgres(
        pool,
        sdkwork_database_sqlx::PoolContext {
            config: DatabaseConfig::default(),
        },
    )
}

async fn assert_contract_tables_present(pool: &PgPool) {
    for table in CONTRACT_TABLES {
        let present: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = current_schema() AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("query table presence for {table}: {error}"));
        assert!(present, "contract table {table} is missing");
    }
}

async fn assert_column_present(pool: &PgPool, table: &str, column: &str) {
    let present: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = current_schema() AND table_name = $1 AND column_name = $2
        )",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
    .expect("query column presence");
    assert!(present, "column {table}.{column} is missing");
}

async fn assert_migration_count(pool: &PgPool, expected: i64) {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ops_schema_migration_history WHERE module_id = 'deploy'",
    )
    .fetch_one(pool)
    .await
    .expect("query migration history");
    assert_eq!(count, expected, "deploy migration history count");
}

/// Fresh database: `init` bootstraps the consolidated baseline, `migrate`
/// applies the forward migrations as idempotent no-ops, and the drift gate
/// reports a clean schema.
#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn fresh_database_init_and_migrate_converge_to_contract() {
    let pool = common::postgres_schema_pool().await;
    let module = deploy_module();
    let orchestrator = LifecycleOrchestrator::new(database_pool(pool.clone()), module.clone())
        .with_applied_by("sdkwork-deploy-test");

    orchestrator
        .init()
        .await
        .expect("init on an empty schema must bootstrap the baseline");
    orchestrator
        .migrate()
        .await
        .expect("migrate on a fresh schema must apply idempotently");

    assert_contract_tables_present(&pool).await;
    assert_migration_count(&pool, 9).await;

    let drift = DriftEngine::new(database_pool(pool.clone()), module)
        .analyze()
        .await
        .expect("drift analyze");
    assert_eq!(
        drift.summary.error,
        0,
        "fresh schema must be drift-clean: {}",
        drift
            .diffs
            .iter()
            .map(|diff| diff.message.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    );
}

/// Legacy database: the anchor table exists so `init` skips the baseline
/// (orchestrator short-circuit), and the forward migrations must create the
/// missing tables, converge the redesigned `deploy_domain` /
/// `deploy_certificate` shapes, and leave the schema drift-clean.
#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn legacy_database_upgrades_through_forward_migrations() {
    let pool = common::postgres_schema_pool().await;
    sqlx::raw_sql(LEGACY_BASELINE)
        .execute(&pool)
        .await
        .expect("apply 2026-06-24 legacy baseline");

    let module = deploy_module();
    let orchestrator = LifecycleOrchestrator::new(database_pool(pool.clone()), module.clone())
        .with_applied_by("sdkwork-deploy-test");

    orchestrator
        .init()
        .await
        .expect("init on a legacy schema must skip the baseline and record state");
    orchestrator
        .migrate()
        .await
        .expect("migrate must converge the legacy schema");

    assert_contract_tables_present(&pool).await;
    assert_migration_count(&pool, 9).await;

    // The redesigned tables carry the current contract columns...
    assert_column_present(&pool, "deploy_domain", "hostname_ascii").await;
    assert_column_present(&pool, "deploy_domain", "verification_status").await;
    assert_column_present(&pool, "deploy_domain", "zone_id").await;
    assert_column_present(&pool, "deploy_certificate", "certificate_source").await;
    assert_column_present(&pool, "deploy_certificate", "idempotency_key").await;
    assert_column_present(&pool, "deploy_certificate", "current_version_id").await;
    assert_column_present(&pool, "deploy_site", "default_variant_id").await;
    assert_column_present(&pool, "deploy_server", "node_role").await;
    // ...and the legacy-only columns are gone.
    assert_column_absent(&pool, "deploy_domain", "hostname").await;
    assert_column_absent(&pool, "deploy_domain", "site_id").await;
    assert_column_absent(&pool, "deploy_certificate", "cert_type").await;
    assert_column_absent(&pool, "deploy_certificate", "san_list").await;

    let drift = DriftEngine::new(database_pool(pool.clone()), module)
        .analyze()
        .await
        .expect("drift analyze");
    assert_eq!(
        drift.summary.error,
        0,
        "upgraded legacy schema must be drift-clean: {}",
        drift
            .diffs
            .iter()
            .map(|diff| diff.message.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    );
}

/// The convergence migration is idempotent: re-running the pipeline on an
/// already-current schema must not fail and must stay drift-clean.
#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn convergence_is_idempotent_on_current_schema() {
    let pool = common::postgres_schema_pool().await;
    let module = deploy_module();
    let orchestrator = LifecycleOrchestrator::new(database_pool(pool.clone()), module.clone())
        .with_applied_by("sdkwork-deploy-test");

    orchestrator.init().await.expect("first init");
    orchestrator.migrate().await.expect("first migrate");
    orchestrator
        .init()
        .await
        .expect("second init must be a no-op");
    orchestrator
        .migrate()
        .await
        .expect("second migrate must be a no-op");

    let drift = DriftEngine::new(database_pool(pool.clone()), module)
        .analyze()
        .await
        .expect("drift analyze");
    assert_eq!(
        drift.summary.error, 0,
        "re-init schema must stay drift-clean"
    );
}

async fn assert_column_absent(pool: &PgPool, table: &str, column: &str) {
    let present: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = current_schema() AND table_name = $1 AND column_name = $2
        )",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
    .expect("query column presence");
    assert!(!present, "column {table}.{column} must have been dropped");
}
