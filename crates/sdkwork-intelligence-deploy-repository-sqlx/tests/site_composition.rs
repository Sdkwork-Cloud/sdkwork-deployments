use std::fs;
use std::path::PathBuf;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_content_provider_port::ValidatedContentProviderResource;
use sdkwork_deploy_contract::{
    ContentProviderResourceSource, DriveWebsiteContentMode, DriveWebsiteRootSelector,
    SiteBindingAction, SiteBindingDefinition, SiteClientClass, SiteDeliveryPolicy, SiteEnvironment,
    SiteMountDefinition, SiteMountHandler, SiteMountMode, SiteObservabilityPolicy,
    SiteResourceDefinition, SiteRuntimeLimits, SiteSecurityPolicy, SiteVariantDefinition,
    SiteVariantRuleDefinition, SiteVariantRuleMatcher, UpdateSiteCompositionRequest,
};
use sdkwork_deploy_runtime_compiler::{RuntimeProviderType, RuntimeResourceCapabilities};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::{
    ReplaceSiteCompositionCommand, SiteCompositionRepositoryPort,
};
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

async fn test_repository() -> (DeployRepository, AnyPool, SqliteTestFile) {
    sqlx::any::install_default_drivers();
    let relative_path = PathBuf::from(format!(
        "target/site-composition-{}.db",
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
    seed_control_plane(&pool).await;
    (
        DeployRepository::new(
            pool.clone(),
            SnowflakeIdGenerator::new(2).expect("Snowflake generator"),
        ),
        pool,
        SqliteTestFile(relative_path),
    )
}

async fn seed_control_plane(pool: &AnyPool) {
    sqlx::query(
        "INSERT INTO deploy_site (
            id,uuid,tenant_id,organization_id,name,slug,site_type,status,runtime_config,
            metadata,created_at,updated_at,version
         ) VALUES (10,'site-1',7,9,'Docs','docs',1,1,'{}','{}',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',0)",
    )
    .execute(pool)
    .await
    .expect("insert site");
    sqlx::query(
        "INSERT INTO deploy_domain (
            id,uuid,tenant_id,organization_id,site_id,hostname,is_verified,status,metadata,
            created_at,updated_at,version
         ) VALUES (20,'domain-1',7,9,10,'docs.example.com',TRUE,1,'{}',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',0)",
    )
    .execute(pool)
    .await
    .expect("insert domain");
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id,uuid,tenant_id,node_uuid,environment,tenant_scope_hash,status,
            created_at,updated_at,version
         ) VALUES (30,'target-1',7,'node-1','production',$1,'ACTIVE',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',1)",
    )
    .bind("a".repeat(64))
    .execute(pool)
    .await
    .expect("insert target");
}

fn request(handler: SiteMountHandler) -> UpdateSiteCompositionRequest {
    UpdateSiteCompositionRequest {
        environment: SiteEnvironment::Production,
        default_variant_key: "default".to_owned(),
        resources: vec![SiteResourceDefinition {
            key: "content".to_owned(),
            source: ContentProviderResourceSource::drive_directory(
                "space-1".to_owned(),
                DriveWebsiteRootSelector::SpaceRoot,
                DriveWebsiteContentMode::LiveTree,
            ),
        }],
        variants: vec![SiteVariantDefinition {
            key: "default".to_owned(),
            label: "Default".to_owned(),
            client_class: SiteClientClass::Other,
            priority: 0,
        }],
        variant_rules: vec![],
        mounts: vec![SiteMountDefinition {
            key: "root".to_owned(),
            variant_key: "default".to_owned(),
            resource_key: "content".to_owned(),
            path_prefix: "/".to_owned(),
            resource_subpath: "/".to_owned(),
            mode: SiteMountMode::Root,
            handler,
            index_files: vec!["index.html".to_owned()],
            spa_fallback: None,
            priority: 0,
        }],
        bindings: vec![SiteBindingDefinition {
            key: "primary".to_owned(),
            domain_id: "domain-1".to_owned(),
            path_prefix: "/".to_owned(),
            action: SiteBindingAction::Serve {
                default_variant_key: None,
                forced_variant_key: None,
            },
        }],
        delivery_policy: SiteDeliveryPolicy::default(),
        security_policy: SiteSecurityPolicy::default(),
        limits: SiteRuntimeLimits::default(),
        observability_policy: SiteObservabilityPolicy::default(),
    }
}

fn command(
    expected_version: i64,
    idempotency_key: &str,
    request_sha256: &str,
    handler: SiteMountHandler,
) -> ReplaceSiteCompositionCommand {
    let request = request(handler);
    ReplaceSiteCompositionCommand {
        tenant_id: 7,
        organization_id: 9,
        actor_id: 11,
        site_uuid: "site-1".to_owned(),
        expected_site_version: expected_version,
        idempotency_key: idempotency_key.to_owned(),
        request_sha256: request_sha256.to_owned(),
        generated_at: "2026-07-22T00:00:01Z".to_owned(),
        resources: vec![ValidatedContentProviderResource {
            key: "content".to_owned(),
            source: request.resources[0].source.clone(),
            provider_type: RuntimeProviderType::Drive,
            provider_resource_uuid: "website-root-1".to_owned(),
            provider_contract_version: "sdkwork.drive.website-root.v1".to_owned(),
            capabilities: RuntimeResourceCapabilities {
                static_content: true,
                wiki_routes: false,
                wiki_search: false,
                range_requests: true,
            },
        }],
        request,
    }
}

fn tv_command(
    expected_version: i64,
    idempotency_key: &str,
    request_sha256: &str,
) -> ReplaceSiteCompositionCommand {
    let mut command = command(
        expected_version,
        idempotency_key,
        request_sha256,
        SiteMountHandler::Static,
    );
    command.request.variants[0].client_class = SiteClientClass::Tv;
    command
        .request
        .variant_rules
        .push(SiteVariantRuleDefinition {
            key: "tv-client".to_owned(),
            target_variant_key: "default".to_owned(),
            priority: 100,
            matcher: SiteVariantRuleMatcher::ClientClass {
                client_class: SiteClientClass::Tv,
            },
        });
    command
}

#[tokio::test]
async fn composition_is_atomic_idempotent_and_does_not_create_releases() {
    let (repository, pool, _file) = test_repository().await;
    let first = repository
        .replace_site_composition(command(
            0,
            "composition-1",
            &"1".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect("publish first composition");
    assert_eq!(first.site_version, "1");
    assert_eq!(first.revision.number, "1");
    assert_eq!(first.runtime_assignments.len(), 1);
    assert_eq!(first.runtime_assignments[0].generation, "1");

    let replay = repository
        .replace_site_composition(command(
            0,
            "composition-1",
            &"1".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect("replay composition");
    assert_eq!(replay.revision.id, first.revision.id);
    assert_eq!(
        replay.runtime_assignments[0].assignment_id,
        first.runtime_assignments[0].assignment_id
    );

    let conflicting_key = repository
        .replace_site_composition(command(
            0,
            "composition-1",
            &"2".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect_err("same key with another request must conflict");
    assert!(conflicting_key.to_string().contains("Idempotency-Key"));

    repository
        .replace_site_composition(command(
            0,
            "composition-stale",
            &"3".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect_err("stale site version must conflict");

    repository
        .replace_site_composition(command(
            1,
            "composition-invalid",
            &"4".repeat(64),
            SiteMountHandler::Wiki,
        ))
        .await
        .expect_err("incompatible provider and handler must roll back");

    let site = sqlx::query(
        "SELECT version, desired_revision_id, default_variant_id FROM deploy_site WHERE id = 10",
    )
    .fetch_one(&pool)
    .await
    .expect("load site");
    assert_eq!(site.try_get::<i64, _>("version").unwrap(), 1);
    assert!(site.try_get::<i64, _>("desired_revision_id").is_ok());
    assert!(site.try_get::<i64, _>("default_variant_id").is_ok());

    let counts = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM deploy_site_revision) AS revisions,
            (SELECT COUNT(*) FROM deploy_runtime_assignment) AS assignments,
            (SELECT COUNT(*) FROM deploy_release) AS releases,
            (SELECT COUNT(*) FROM deploy_deployment) AS deployments",
    )
    .fetch_one(&pool)
    .await
    .expect("load counts");
    assert_eq!(counts.try_get::<i64, _>("revisions").unwrap(), 1);
    assert_eq!(counts.try_get::<i64, _>("assignments").unwrap(), 1);
    assert_eq!(counts.try_get::<i64, _>("releases").unwrap(), 0);
    assert_eq!(counts.try_get::<i64, _>("deployments").unwrap(), 0);
}

#[tokio::test]
async fn composition_persists_tv_client_class_and_compiles_the_runtime_rule() {
    let (repository, pool, _file) = test_repository().await;
    repository
        .replace_site_composition(tv_command(0, "composition-tv", &"9".repeat(64)))
        .await
        .expect("publish TV composition");

    let stored = sqlx::query(
        "SELECT v.client_class, r.match_value,
                CAST(revision.descriptor_json AS TEXT) AS descriptor_json
         FROM deploy_site_variant v
         INNER JOIN deploy_site_variant_rule r ON r.site_id = v.site_id
         INNER JOIN deploy_site site ON site.id = v.site_id
         INNER JOIN deploy_site_revision revision ON revision.id = site.desired_revision_id
         WHERE v.site_id = 10",
    )
    .fetch_one(&pool)
    .await
    .expect("load TV composition");
    assert_eq!(stored.try_get::<String, _>("client_class").unwrap(), "TV");
    assert_eq!(stored.try_get::<String, _>("match_value").unwrap(), "TV");
    let descriptor: serde_json::Value =
        serde_json::from_str(&stored.try_get::<String, _>("descriptor_json").unwrap())
            .expect("parse stored runtime descriptor");
    assert_eq!(descriptor["variantRules"][0]["match"]["clientClass"], "TV");
}

#[tokio::test]
async fn composition_rejects_a_domain_owned_by_another_tenant() {
    let (repository, pool, _file) = test_repository().await;
    sqlx::query(
        "INSERT INTO deploy_domain (
            id,uuid,tenant_id,organization_id,site_id,hostname,is_verified,status,metadata,
            created_at,updated_at,version
         ) VALUES (21,'domain-foreign',8,9,10,'foreign.example.com',TRUE,1,'{}',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',0)",
    )
    .execute(&pool)
    .await
    .expect("insert foreign tenant domain");

    let mut command = command(
        0,
        "composition-foreign-domain",
        &"6".repeat(64),
        SiteMountHandler::Static,
    );
    command.request.bindings[0].domain_id = "domain-foreign".to_owned();
    let error = repository
        .replace_site_composition(command)
        .await
        .expect_err("cross-tenant domain must not bind");
    assert!(error.to_string().contains("domain not found for site"));
    assert_composition_was_not_committed(&pool).await;
}

#[tokio::test]
async fn composition_requires_an_active_web_target() {
    let (repository, pool, _file) = test_repository().await;
    sqlx::query("DELETE FROM deploy_web_node_target WHERE tenant_id = 7")
        .execute(&pool)
        .await
        .expect("remove Web target");

    let error = repository
        .replace_site_composition(command(
            0,
            "composition-no-target",
            &"7".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect_err("composition without target must fail");
    assert!(error.to_string().contains("no active Web Node target"));
    assert_composition_was_not_committed(&pool).await;
}

#[tokio::test]
async fn composition_rejects_inconsistent_target_tenant_scope() {
    let (repository, pool, _file) = test_repository().await;
    sqlx::query(
        "INSERT INTO deploy_web_node_target (
            id,uuid,tenant_id,node_uuid,environment,tenant_scope_hash,status,
            created_at,updated_at,version
         ) VALUES (31,'target-2',7,'node-2','production',$1,'ACTIVE',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',1)",
    )
    .bind("b".repeat(64))
    .execute(&pool)
    .await
    .expect("insert inconsistent Web target");

    let error = repository
        .replace_site_composition(command(
            0,
            "composition-scope-conflict",
            &"8".repeat(64),
            SiteMountHandler::Static,
        ))
        .await
        .expect_err("inconsistent target scope must fail");
    assert!(error
        .to_string()
        .contains("Web Node targets have inconsistent tenant scope"));
    assert_composition_was_not_committed(&pool).await;
}

async fn assert_composition_was_not_committed(pool: &AnyPool) {
    let row = sqlx::query(
        "SELECT version,
                (SELECT COUNT(*) FROM deploy_site_revision) AS revisions,
                (SELECT COUNT(*) FROM deploy_runtime_assignment) AS assignments
         FROM deploy_site WHERE id = 10",
    )
    .fetch_one(pool)
    .await
    .expect("load rolled-back composition state");
    assert_eq!(row.try_get::<i64, _>("version").unwrap(), 0);
    assert_eq!(row.try_get::<i64, _>("revisions").unwrap(), 0);
    assert_eq!(row.try_get::<i64, _>("assignments").unwrap(), 0);
}

#[tokio::test]
#[ignore = "requires an empty PostgreSQL database in SDKWORK_DEPLOY_TEST_POSTGRES_URL"]
async fn postgres_composition_matches_sqlite_transaction_semantics() {
    sqlx::any::install_default_drivers();
    let database_url = std::env::var("SDKWORK_DEPLOY_TEST_POSTGRES_URL")
        .expect("SDKWORK_DEPLOY_TEST_POSTGRES_URL must target an empty PostgreSQL database");
    let pool = AnyPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect PostgreSQL integration database");
    sqlx::raw_sql(POSTGRES_BASELINE)
        .execute(&pool)
        .await
        .expect("apply PostgreSQL baseline");
    seed_control_plane(&pool).await;
    let repository = DeployRepository::new(
        pool.clone(),
        SnowflakeIdGenerator::new(3).expect("Snowflake generator"),
    );

    let first = repository
        .replace_site_composition(tv_command(0, "composition-postgres-1", &"5".repeat(64)))
        .await
        .expect("publish PostgreSQL composition");
    assert_eq!(first.site_version, "1");
    assert_eq!(first.runtime_assignments[0].generation, "1");
    let replay = repository
        .replace_site_composition(tv_command(0, "composition-postgres-1", &"5".repeat(64)))
        .await
        .expect("replay PostgreSQL composition");
    assert_eq!(replay.revision.id, first.revision.id);

    let row = sqlx::query(
        "SELECT s.version, r.request_sha256, CAST(r.result_json AS TEXT) AS result_json,
                a.generation, a.publish_status, v.client_class, vr.match_value
         FROM deploy_site s
         INNER JOIN deploy_site_revision r ON r.id = s.desired_revision_id
         INNER JOIN deploy_runtime_assignment a ON a.trigger_site_revision_id = r.id
         INNER JOIN deploy_site_variant v ON v.site_id = s.id
         INNER JOIN deploy_site_variant_rule vr ON vr.site_id = s.id
         WHERE s.id = 10",
    )
    .fetch_one(&pool)
    .await
    .expect("load PostgreSQL composition state");
    assert_eq!(row.try_get::<i64, _>("version").unwrap(), 1);
    assert_eq!(row.try_get::<i64, _>("generation").unwrap(), 1);
    assert_eq!(
        row.try_get::<String, _>("publish_status").unwrap(),
        "PENDING"
    );
    assert_eq!(row.try_get::<String, _>("client_class").unwrap(), "TV");
    assert_eq!(row.try_get::<String, _>("match_value").unwrap(), "TV");
    assert!(row
        .try_get::<String, _>("result_json")
        .unwrap()
        .contains(&first.revision.id));
}
