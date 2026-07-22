use std::sync::Arc;
use std::{fs, path::PathBuf};

use async_trait::async_trait;
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::DeployServiceResult;
use sdkwork_deploy_runtime_compiler::{
    canonical_sha256_excluding_field, CompiledRuntimeSet, RuntimeEnvironment,
};
use sdkwork_deploy_web_port::{DeployWebRuntimePort, RuntimeAssignmentReceipt};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::runtime_publication::{
    DeployRuntimeAssignmentRepositoryPort, RuntimeAssignmentPublishStatus, RuntimeTarget,
};
use sdkwork_intelligence_deploy_service::RuntimePublicationService;
use sqlx::{any::AnyPoolOptions, AnyPool, Row};

const SQLITE_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/sqlite/0001_deploy_baseline.sql");
const POSTGRES_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/postgres/0001_deploy_baseline.sql");

struct SqliteTestFile(PathBuf);

impl Drop for SqliteTestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(self.0.with_extension("db-shm"));
        let _ = fs::remove_file(self.0.with_extension("db-wal"));
    }
}

async fn file_backed_sqlite_pool() -> (AnyPool, SqliteTestFile) {
    sqlx::any::install_default_drivers();
    let relative_path = PathBuf::from(format!(
        "target/runtime-assignment-{}.db",
        sdkwork_database_id::uuid_v4()
    ));
    fs::create_dir_all("target").expect("create Cargo target directory");
    let database_url = format!("sqlite://{}?mode=rwc", relative_path.display());
    let pool = AnyPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect file-backed SQLite");
    sqlx::raw_sql(SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply SQLite baseline");
    (pool, SqliteTestFile(relative_path))
}

#[derive(Default)]
struct AcceptingWebRuntime;

fn runtime_descriptor(site_uuid: &str, tenant_scope_hash: &str) -> serde_json::Value {
    let mut descriptor = serde_json::json!({
        "siteUuid": site_uuid,
        "environment": "production",
        "tenantScopeHash": tenant_scope_hash,
        "descriptorSha256": ""
    });
    let descriptor_sha256 = canonical_sha256_excluding_field(&descriptor, "descriptorSha256")
        .expect("hash runtime descriptor");
    descriptor["descriptorSha256"] = serde_json::Value::String(descriptor_sha256);
    descriptor
}

#[async_trait]
impl DeployWebRuntimePort for AcceptingWebRuntime {
    async fn publish_runtime_assignment(
        &self,
        runtime_set: &CompiledRuntimeSet,
    ) -> DeployServiceResult<RuntimeAssignmentReceipt> {
        Ok(RuntimeAssignmentReceipt {
            assignment_uuid: "web-assignment-1".to_owned(),
            node_uuid: runtime_set.snapshot["nodeUuid"]
                .as_str()
                .unwrap()
                .to_owned(),
            environment: runtime_set.snapshot["environment"]
                .as_str()
                .unwrap()
                .to_owned(),
            generation: runtime_set.snapshot["generation"].to_string(),
            snapshot_uuid: runtime_set.snapshot["snapshotUuid"]
                .as_str()
                .unwrap()
                .to_owned(),
            snapshot_sha256: runtime_set.snapshot_sha256.clone(),
            assigned_at: "2026-07-22T00:00:01Z".to_owned(),
        })
    }
}

#[tokio::test]
async fn sqlite_outbox_is_durable_idempotent_and_publishable() {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect SQLite");
    sqlx::raw_sql(SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply SQLite baseline");
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id, uuid, tenant_id, node_uuid, environment, tenant_scope_hash,
            status, created_at, updated_at, version
         ) VALUES (1, 'target-1', 7, 'node-1', 'production', $1,
                   'ACTIVE', '2026-07-22T00:00:00Z', '2026-07-22T00:00:00Z', 1)",
    )
    .bind("1".repeat(64))
    .execute(&pool)
    .await
    .expect("insert Web Node target");

    let repository = Arc::new(DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(1).expect("Snowflake generator"),
    ));
    let publisher = RuntimePublicationService::new(
        repository.clone() as Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        Arc::new(AcceptingWebRuntime),
    );
    let target = RuntimeTarget {
        target_uuid: "target-1".to_owned(),
        tenant_id: 7,
        node_uuid: "node-1".to_owned(),
        environment: RuntimeEnvironment::Production,
        tenant_scope_hash: "1".repeat(64),
    };

    let pending = publisher
        .enqueue_target_state(
            &target,
            "snapshot-1".to_owned(),
            "2026-07-22T00:00:00Z".to_owned(),
            vec![],
        )
        .await
        .expect("persist pending assignment");
    assert_eq!(pending.generation, 1);
    assert_eq!(
        pending.publish_status,
        RuntimeAssignmentPublishStatus::Pending
    );

    let replay = publisher
        .enqueue_target_state(
            &target,
            "snapshot-unused".to_owned(),
            "2026-07-22T00:00:00Z".to_owned(),
            vec![],
        )
        .await
        .expect("same desired state is idempotent");
    assert_eq!(replay.assignment_uuid, pending.assignment_uuid);

    let batch = publisher
        .publish_due("worker-1", 10, 30)
        .await
        .expect("publish durable assignment");
    assert_eq!(batch.claimed, 1);
    assert_eq!(batch.published, 1);
    assert_eq!(batch.failed, 0);
    let published = repository
        .latest_runtime_assignment("target-1")
        .await
        .expect("reload assignment")
        .expect("assignment exists");
    assert_eq!(
        published.publish_status,
        RuntimeAssignmentPublishStatus::Published
    );
    assert_eq!(published.snapshot_sha256, pending.snapshot_sha256);
}

#[tokio::test]
async fn sqlite_outbox_reuses_assignment_when_descriptor_order_changes() {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect SQLite");
    sqlx::raw_sql(SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply SQLite baseline");
    let tenant_scope_hash = "1".repeat(64);
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id, uuid, tenant_id, node_uuid, environment, tenant_scope_hash,
            status, created_at, updated_at, version
         ) VALUES (1, 'target-1', 7, 'node-1', 'production', $1,
                   'ACTIVE', '2026-07-22T00:00:00Z', '2026-07-22T00:00:00Z', 1)",
    )
    .bind(&tenant_scope_hash)
    .execute(&pool)
    .await
    .expect("insert Web Node target");

    let repository = Arc::new(DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(1).expect("Snowflake generator"),
    ));
    let publisher = RuntimePublicationService::new(
        repository as Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        Arc::new(AcceptingWebRuntime),
    );
    let target = RuntimeTarget {
        target_uuid: "target-1".to_owned(),
        tenant_id: 7,
        node_uuid: "node-1".to_owned(),
        environment: RuntimeEnvironment::Production,
        tenant_scope_hash: tenant_scope_hash.clone(),
    };
    let site_a = runtime_descriptor("site-a", &tenant_scope_hash);
    let site_b = runtime_descriptor("site-b", &tenant_scope_hash);

    let first = publisher
        .enqueue_target_state(
            &target,
            "snapshot-1".to_owned(),
            "2026-07-22T00:00:00Z".to_owned(),
            vec![site_b.clone(), site_a.clone()],
        )
        .await
        .expect("persist ordered assignment");
    let replay = publisher
        .enqueue_target_state(
            &target,
            "snapshot-unused".to_owned(),
            "2026-07-22T00:00:01Z".to_owned(),
            vec![site_a, site_b],
        )
        .await
        .expect("reuse assignment for equivalent descriptor set");

    assert_eq!(replay.assignment_uuid, first.assignment_uuid);
    assert_eq!(replay.generation, first.generation);
}

#[tokio::test]
async fn sqlite_concurrent_mutations_are_serialized_and_idempotent() {
    let (pool, _database_file) = file_backed_sqlite_pool().await;
    let tenant_scope_hash = "1".repeat(64);
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id, uuid, tenant_id, node_uuid, environment, tenant_scope_hash,
            status, created_at, updated_at, version
         ) VALUES (1, 'target-1', 7, 'node-1', 'production', $1,
                   'ACTIVE', '2026-07-22T00:00:00Z', '2026-07-22T00:00:00Z', 1)",
    )
    .bind(&tenant_scope_hash)
    .execute(&pool)
    .await
    .expect("insert Web Node target");

    let repository = Arc::new(DeployRepository::new(
        pool.clone(),
        SnowflakeIdGenerator::new(1).expect("Snowflake generator"),
    ));
    let publisher = Arc::new(RuntimePublicationService::new(
        repository.clone() as Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        Arc::new(AcceptingWebRuntime),
    ));
    let target = RuntimeTarget {
        target_uuid: "target-1".to_owned(),
        tenant_id: 7,
        node_uuid: "node-1".to_owned(),
        environment: RuntimeEnvironment::Production,
        tenant_scope_hash: tenant_scope_hash.clone(),
    };
    let site_a = runtime_descriptor("site-a", &tenant_scope_hash);

    let first = publisher.enqueue_target_state(
        &target,
        "snapshot-1".to_owned(),
        "2026-07-22T00:00:00Z".to_owned(),
        vec![site_a.clone()],
    );
    let duplicate = publisher.enqueue_target_state(
        &target,
        "snapshot-duplicate".to_owned(),
        "2026-07-22T00:00:01Z".to_owned(),
        vec![site_a],
    );
    let (first, duplicate) = tokio::join!(first, duplicate);
    let first = first.expect("persist first concurrent desired state");
    let duplicate = duplicate.expect("reuse concurrent desired state");
    assert_eq!(first.assignment_uuid, duplicate.assignment_uuid);
    assert_eq!(first.generation, 1);

    let second = publisher.enqueue_target_state(
        &target,
        "snapshot-2".to_owned(),
        "2026-07-22T00:00:02Z".to_owned(),
        vec![runtime_descriptor("site-b", &tenant_scope_hash)],
    );
    let third = publisher.enqueue_target_state(
        &target,
        "snapshot-3".to_owned(),
        "2026-07-22T00:00:03Z".to_owned(),
        vec![runtime_descriptor("site-c", &tenant_scope_hash)],
    );
    let (second, third) = tokio::join!(second, third);
    let mut concurrent_generations = [
        second.expect("persist second desired state").generation,
        third.expect("persist third desired state").generation,
    ];
    concurrent_generations.sort_unstable();
    assert_eq!(concurrent_generations, [2, 3]);

    let rows = sqlx::query(
        "SELECT generation, publish_status FROM deploy_runtime_assignment ORDER BY generation",
    )
    .fetch_all(&pool)
    .await
    .expect("load serialized assignments");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get::<i64, _>("generation"), 1);
    assert_eq!(rows[0].get::<String, _>("publish_status"), "SUPERSEDED");
    assert_eq!(rows[1].get::<i64, _>("generation"), 2);
    assert_eq!(rows[1].get::<String, _>("publish_status"), "SUPERSEDED");
    assert_eq!(rows[2].get::<i64, _>("generation"), 3);
    assert_eq!(rows[2].get::<String, _>("publish_status"), "PENDING");

    drop(publisher);
    drop(repository);
    pool.close().await;
}

#[tokio::test]
async fn sqlite_claim_lease_fences_stale_workers_and_stops_at_attempt_limit() {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect SQLite");
    sqlx::raw_sql(SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply SQLite baseline");
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id, uuid, tenant_id, node_uuid, environment, tenant_scope_hash,
            status, created_at, updated_at, version
         ) VALUES (1, 'target-1', 7, 'node-1', 'production', $1,
                   'ACTIVE', '2026-07-22T00:00:00Z', '2026-07-22T00:00:00Z', 1)",
    )
    .bind("1".repeat(64))
    .execute(&pool)
    .await
    .expect("insert Web Node target");
    let repository = Arc::new(DeployRepository::new(
        pool.clone(),
        SnowflakeIdGenerator::new(1).expect("Snowflake generator"),
    ));
    let publisher = RuntimePublicationService::new(
        repository.clone() as Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        Arc::new(AcceptingWebRuntime),
    );
    let target = RuntimeTarget {
        target_uuid: "target-1".to_owned(),
        tenant_id: 7,
        node_uuid: "node-1".to_owned(),
        environment: RuntimeEnvironment::Production,
        tenant_scope_hash: "1".repeat(64),
    };
    publisher
        .enqueue_target_state(
            &target,
            "snapshot-1".to_owned(),
            "2026-07-22T00:00:00Z".to_owned(),
            vec![],
        )
        .await
        .expect("persist pending assignment");

    let first_claim = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:00:00Z",
            "worker-1",
            "2026-07-22T00:00:30Z",
            20,
        )
        .await
        .expect("claim pending assignment");
    assert_eq!(first_claim.len(), 1);
    assert_eq!(first_claim[0].attempt_count, 1);
    assert_eq!(first_claim[0].lease_owner.as_deref(), Some("worker-1"));

    let early_claim = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:00:10Z",
            "worker-2",
            "2026-07-22T00:00:40Z",
            20,
        )
        .await
        .expect("skip active lease");
    assert!(early_claim.is_empty());

    let takeover = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:00:31Z",
            "worker-2",
            "2026-07-22T00:01:01Z",
            20,
        )
        .await
        .expect("take over expired lease");
    assert_eq!(takeover.len(), 1);
    assert_eq!(takeover[0].attempt_count, 2);
    assert_eq!(takeover[0].lease_owner.as_deref(), Some("worker-2"));

    assert!(repository
        .mark_runtime_assignment_failed(
            &takeover[0].assignment_uuid,
            "worker-1",
            "STALE_WORKER",
            None,
            "2026-07-22T00:00:32Z",
        )
        .await
        .is_err());
    repository
        .mark_runtime_assignment_failed(
            &takeover[0].assignment_uuid,
            "worker-2",
            "WEB_PUBLICATION_UNAVAILABLE",
            Some("2026-07-22T00:01:30Z"),
            "2026-07-22T00:00:32Z",
        )
        .await
        .expect("lease owner records failure");

    sqlx::query(
        "UPDATE deploy_runtime_assignment
         SET attempt_count = 20, next_attempt_at = '2026-07-22T00:01:30Z'
         WHERE uuid = $1",
    )
    .bind(&takeover[0].assignment_uuid)
    .execute(&pool)
    .await
    .expect("set maximum attempts");
    let exhausted = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:02:00Z",
            "worker-3",
            "2026-07-22T00:02:30Z",
            20,
        )
        .await
        .expect("scan exhausted assignment");
    assert!(exhausted.is_empty());
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in SDKWORK_DEPLOY_TEST_POSTGRES_URL"]
async fn postgres_serializes_mutations_and_fences_assignment_leases() {
    sqlx::any::install_default_drivers();
    let database_url = std::env::var("SDKWORK_DEPLOY_TEST_POSTGRES_URL")
        .expect("SDKWORK_DEPLOY_TEST_POSTGRES_URL must target an empty PostgreSQL database");
    let pool = AnyPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("connect PostgreSQL integration database");
    sqlx::raw_sql(POSTGRES_BASELINE)
        .execute(&pool)
        .await
        .expect("apply PostgreSQL baseline");

    let tenant_scope_hash = "1".repeat(64);
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id, uuid, tenant_id, node_uuid, environment, tenant_scope_hash,
            status, created_at, updated_at, version
         ) VALUES (1, 'target-1', 7, 'node-1', 'production', $1,
                   'ACTIVE', '2026-07-22T00:00:00Z', '2026-07-22T00:00:00Z', 1)",
    )
    .bind(&tenant_scope_hash)
    .execute(&pool)
    .await
    .expect("insert PostgreSQL Web Node target");

    let repository = Arc::new(DeployRepository::new(
        pool.clone(),
        SnowflakeIdGenerator::new(1).expect("Snowflake generator"),
    ));
    let publisher = Arc::new(RuntimePublicationService::new(
        repository.clone() as Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        Arc::new(AcceptingWebRuntime),
    ));
    let target = RuntimeTarget {
        target_uuid: "target-1".to_owned(),
        tenant_id: 7,
        node_uuid: "node-1".to_owned(),
        environment: RuntimeEnvironment::Production,
        tenant_scope_hash: tenant_scope_hash.clone(),
    };

    let site_a = runtime_descriptor("site-a", &tenant_scope_hash);
    let first = publisher.enqueue_target_state(
        &target,
        "snapshot-1".to_owned(),
        "2026-07-22T00:00:00Z".to_owned(),
        vec![site_a.clone()],
    );
    let duplicate = publisher.enqueue_target_state(
        &target,
        "snapshot-duplicate".to_owned(),
        "2026-07-22T00:00:01Z".to_owned(),
        vec![site_a],
    );
    let (first, duplicate) = tokio::join!(first, duplicate);
    let first = first.expect("persist first PostgreSQL desired state");
    let duplicate = duplicate.expect("reuse concurrent PostgreSQL desired state");
    assert_eq!(first.assignment_uuid, duplicate.assignment_uuid);
    assert_eq!(first.generation, 1);

    let second = publisher.enqueue_target_state(
        &target,
        "snapshot-2".to_owned(),
        "2026-07-22T00:00:02Z".to_owned(),
        vec![runtime_descriptor("site-b", &tenant_scope_hash)],
    );
    let third = publisher.enqueue_target_state(
        &target,
        "snapshot-3".to_owned(),
        "2026-07-22T00:00:03Z".to_owned(),
        vec![runtime_descriptor("site-c", &tenant_scope_hash)],
    );
    let (second, third) = tokio::join!(second, third);
    let mut concurrent_generations = [
        second
            .expect("persist second PostgreSQL desired state")
            .generation,
        third
            .expect("persist third PostgreSQL desired state")
            .generation,
    ];
    concurrent_generations.sort_unstable();
    assert_eq!(concurrent_generations, [2, 3]);

    let rows = sqlx::query(
        "SELECT generation, publish_status
         FROM deploy_runtime_assignment
         ORDER BY generation",
    )
    .fetch_all(&pool)
    .await
    .expect("load PostgreSQL serialized assignments");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get::<i64, _>("generation"), 1);
    assert_eq!(rows[0].get::<String, _>("publish_status"), "SUPERSEDED");
    assert_eq!(rows[1].get::<i64, _>("generation"), 2);
    assert_eq!(rows[1].get::<String, _>("publish_status"), "SUPERSEDED");
    assert_eq!(rows[2].get::<i64, _>("generation"), 3);
    assert_eq!(rows[2].get::<String, _>("publish_status"), "PENDING");

    let first_claim = repository.claim_due_runtime_assignments(
        10,
        "2026-07-22T00:00:04Z",
        "worker-1",
        "2026-07-22T00:00:34Z",
        20,
    );
    let competing_claim = repository.claim_due_runtime_assignments(
        10,
        "2026-07-22T00:00:04Z",
        "worker-2",
        "2026-07-22T00:00:34Z",
        20,
    );
    let (first_claim, competing_claim) = tokio::join!(first_claim, competing_claim);
    let first_claim = first_claim.expect("claim PostgreSQL pending assignment");
    let competing_claim = competing_claim.expect("run competing PostgreSQL claim");
    assert_eq!(first_claim.len() + competing_claim.len(), 1);
    let claimed = first_claim
        .first()
        .or_else(|| competing_claim.first())
        .expect("one PostgreSQL worker owns the assignment");
    let first_owner = claimed
        .lease_owner
        .as_deref()
        .expect("claimed assignment has lease owner");
    assert_eq!(claimed.attempt_count, 1);

    let active_lease_claim = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:00:20Z",
            "worker-3",
            "2026-07-22T00:00:50Z",
            20,
        )
        .await
        .expect("skip active PostgreSQL lease");
    assert!(active_lease_claim.is_empty());

    let takeover = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:00:35Z",
            "worker-3",
            "2026-07-22T00:01:05Z",
            20,
        )
        .await
        .expect("take over expired PostgreSQL lease");
    assert_eq!(takeover.len(), 1);
    assert_eq!(takeover[0].attempt_count, 2);
    assert_eq!(takeover[0].lease_owner.as_deref(), Some("worker-3"));

    assert!(repository
        .mark_runtime_assignment_failed(
            &takeover[0].assignment_uuid,
            first_owner,
            "STALE_WORKER",
            None,
            "2026-07-22T00:00:36Z",
        )
        .await
        .is_err());
    repository
        .mark_runtime_assignment_failed(
            &takeover[0].assignment_uuid,
            "worker-3",
            "WEB_PUBLICATION_UNAVAILABLE",
            Some("2026-07-22T00:01:30Z"),
            "2026-07-22T00:00:36Z",
        )
        .await
        .expect("PostgreSQL lease owner records failure");

    sqlx::query(
        "UPDATE deploy_runtime_assignment
         SET attempt_count = 20, next_attempt_at = '2026-07-22T00:01:30Z'
         WHERE uuid = $1",
    )
    .bind(&takeover[0].assignment_uuid)
    .execute(&pool)
    .await
    .expect("set PostgreSQL maximum attempts");
    let exhausted = repository
        .claim_due_runtime_assignments(
            10,
            "2026-07-22T00:02:00Z",
            "worker-4",
            "2026-07-22T00:02:30Z",
            20,
        )
        .await
        .expect("scan exhausted PostgreSQL assignment");
    assert!(exhausted.is_empty());

    drop(publisher);
    drop(repository);
    pool.close().await;
}
