//! Retention, usage reconciliation, and signing identity health integration
//! tests (PRD §5.8, TECH §8): dry-run and real retention runs, the
//! idempotent daily aggregate rebuild from retained usage facts, and the
//! expiry health surface.
//!
//! Requires `SDKWORK_DATABASE_TEST_POSTGRES_URL`; ignored by default like the
//! other PostgreSQL integration tests in this crate.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_database_lifecycle::LifecycleOrchestrator;
use sdkwork_database_spi::DefaultDatabaseModule;
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;
use sqlx::PgPool;

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

async fn migrated_repository() -> DeployRepository {
    let pool = common::postgres_schema_pool().await;
    let module = deploy_module();
    let orchestrator = LifecycleOrchestrator::new(database_pool(pool.clone()), module.clone())
        .with_applied_by("sdkwork-deploy-retention-test");
    orchestrator
        .init()
        .await
        .expect("init on an empty schema must bootstrap the baseline");
    orchestrator
        .migrate()
        .await
        .expect("migrate must apply the full forward migration chain");
    DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(4).expect("Snowflake generator"),
        common::test_secret_key(),
    )
}

/// Seeds a minimal app with an old unreferenced package and release directly
/// (the full build/package chain is exercised by other integration tests).
async fn seed_old_package_and_release(pool: &PgPool) -> (i64, i64, i64) {
    let app_id: i64 = 5001;
    sqlx::query(
        "INSERT INTO deploy_app (id, uuid, tenant_id, organization_id, name, slug, app_kind,
             app_status, created_at, updated_at)
         VALUES ($1, '00000000-0000-4000-8000-000000000002', 7, 9, 'retention-app', 'retention-app',
                 'API_SERVICE', 'ACTIVE', NOW() - INTERVAL '400 days', NOW() - INTERVAL '400 days')",
    )
    .bind(app_id)
    .execute(pool)
    .await
    .expect("insert app");
    let package_id: i64 = 5002;
    sqlx::query(
        "INSERT INTO deploy_package (id, uuid, tenant_id, organization_id, app_id,
             platform_target_id, build_id, package_format, semantic_version, package_size_bytes,
             checksum_sha256, manifest_sha256, package_status, created_at, updated_at)
         VALUES ($1, '00000000-0000-4000-8000-000000000003', 7, 9, $2, 0, 0, 'TAR_GZ', '1.0.0', 100,
                 'a'.repeat(64), 'b'.repeat(64), 'READY',
                 NOW() - INTERVAL '400 days', NOW() - INTERVAL '400 days')",
    )
    .bind(package_id)
    .bind(app_id)
    .execute(pool)
    .await
    .expect("insert old package");
    let release_id: i64 = 5003;
    sqlx::query(
        "INSERT INTO deploy_release (id, uuid, tenant_id, organization_id, app_id, package_id,
             semantic_version, release_status, created_at, updated_at)
         VALUES ($1, '00000000-0000-4000-8000-000000000004', 7, 9, NULL, $2, '1.0.0', 'ACTIVE',
                 NOW() - INTERVAL '400 days', NOW() - INTERVAL '400 days')",
    )
    .bind(release_id)
    .bind(package_id)
    .execute(pool)
    .await
    .expect("insert old release");
    (app_id, package_id, release_id)
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn retention_dry_run_reports_and_real_run_retires() {
    let repository = migrated_repository().await;
    let (_, _, _) = seed_old_package_and_release(repository.pool()).await;

    // Dry run reports both candidates without mutating.
    let dry = repository
        .run_retention(true, 365, 365, 365)
        .await
        .expect("dry run retention");
    assert_eq!(dry.packages_retired, 1);
    assert_eq!(dry.releases_retired, 1);
    assert_eq!(dry.build_logs_purged, 0);
    let status: String =
        sqlx::query_scalar("SELECT package_status FROM deploy_package WHERE id = 5002")
            .fetch_one(repository.pool())
            .await
            .expect("package status");
    assert_eq!(status, "READY", "dry run must not mutate");

    // Real run retires both; a second real run finds nothing left.
    let real = repository
        .run_retention(false, 365, 365, 365)
        .await
        .expect("real retention run");
    assert_eq!(real.packages_retired, 1);
    assert_eq!(real.releases_retired, 1);
    let package_status: String =
        sqlx::query_scalar("SELECT package_status FROM deploy_package WHERE id = 5002")
            .fetch_one(repository.pool())
            .await
            .expect("package status");
    assert_eq!(package_status, "RETIRED");
    let release_status: String =
        sqlx::query_scalar("SELECT release_status FROM deploy_release WHERE id = 5003")
            .fetch_one(repository.pool())
            .await
            .expect("release status");
    assert_eq!(release_status, "RETIRED");

    let again = repository
        .run_retention(false, 365, 365, 365)
        .await
        .expect("second retention run");
    assert_eq!(again.packages_retired, 0);
    assert_eq!(again.releases_retired, 0);
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn usage_daily_rebuild_is_idempotent_and_reconcilable() {
    let repository = migrated_repository().await;
    // Two usage facts for the same tenant/dimension/date — the aggregate must
    // sum them, and re-running must upsert rather than duplicate.
    for (idx, quantity) in [(1, 7_i64), (2, 3_i64)] {
        sqlx::query(
            "INSERT INTO deploy_usage_event
                (id, uuid, tenant_id, organization_id, period_start, dimension, quantity, unit,
                 deduplication_key, observed_at, ingested_at, created_at)
             VALUES ($1, $2, 7, 9, NOW() - INTERVAL '1 day', 'build_minutes', $3, 'MINUTES',
                     $4, NOW(), NOW(), NOW())",
        )
        .bind(6000 + idx)
        .bind(format!("00000000-0000-4000-8000-00000000000{}", idx + 4))
        .bind(quantity)
        .bind(format!("build:retention-{idx}"))
        .execute(repository.pool())
        .await
        .expect("insert usage fact");
    }

    let window_start = "2026-01-01T00:00:00.000Z";
    let window_end = "2030-01-01T00:00:00.000Z";
    let first = repository
        .rebuild_usage_daily(Some(window_start), Some(window_end))
        .await
        .expect("rebuild daily");
    assert_eq!(first.rebuilt_rows, 1);

    // The aggregate summed both facts into one row.
    let quantity: i64 = sqlx::query_scalar(
        "SELECT quantity FROM deploy_app_usage_daily
         WHERE tenant_id = 7 AND dimension = 'build_minutes' AND usage_date = (NOW() - INTERVAL '1 day')::date",
    )
    .fetch_one(repository.pool())
    .await
    .expect("daily quantity");
    assert_eq!(quantity, 10, "facts must sum into the daily aggregate");

    // Idempotent rebuild: same single row, no duplicates.
    let second = repository
        .rebuild_usage_daily(Some(window_start), Some(window_end))
        .await
        .expect("rebuild daily again");
    assert_eq!(second.rebuilt_rows, 1);
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM deploy_app_usage_daily WHERE tenant_id = 7")
            .fetch_one(repository.pool())
            .await
            .expect("daily row count");
    assert_eq!(count, 1, "rebuild must upsert, never duplicate");

    // Invalid windows fail closed.
    assert!(repository
        .rebuild_usage_daily(Some("not-a-date"), None)
        .await
        .is_err());
    assert!(repository
        .rebuild_usage_daily(
            Some("2030-01-01T00:00:00.000Z"),
            Some("2026-01-01T00:00:00.000Z")
        )
        .await
        .is_err());
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn signing_identity_health_reports_expiry_urgency() {
    let repository = migrated_repository().await;
    sqlx::query(
        "INSERT INTO deploy_signing_identity
            (id, uuid, tenant_id, organization_id, identity_name, signing_kind, expires_at,
             identity_status, created_at, updated_at)
         VALUES (7001, '00000000-0000-4000-8000-000000000009', 7, 9, 'prod-pfx',
                 'WINDOWS_AUTHENTICODE', NOW() + INTERVAL '30 days', 'VALID',
                 NOW(), NOW())",
    )
    .execute(repository.pool())
    .await
    .expect("insert signing identity");

    let page = repository
        .list_signing_identity_health(Some(7), 1, 20)
        .await
        .expect("list signing identity health");
    assert_eq!(page.total, 1);
    let item = &page.items[0];
    assert_eq!(item.signing_kind, "WINDOWS_AUTHENTICODE");
    assert_eq!(item.identity_status, "VALID");
    let days = item.days_until_expiry.expect("days until expiry");
    assert!(
        (20..=40).contains(&days),
        "expiry urgency in the expected window: {days}"
    );

    // Cross-tenant scope returns nothing.
    let other = repository
        .list_signing_identity_health(Some(8), 1, 20)
        .await
        .expect("other tenant health");
    assert_eq!(other.total, 0);

    // Platform-wide (None) sees the tenant's identity.
    let platform = repository
        .list_signing_identity_health(None, 1, 20)
        .await
        .expect("platform-wide health");
    assert_eq!(platform.total, 1);
}
