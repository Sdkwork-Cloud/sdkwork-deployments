mod common;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;
use sqlx::{PgPool, Row};

async fn test_repository() -> (DeployRepository, PgPool) {
    let pool = common::postgres_pool().await;
    seed_control_plane(&pool).await;
    (
        DeployRepository::new(
            pool.clone(),
            SnowflakeIdGenerator::new(3).expect("Snowflake generator"),
            common::test_secret_key(),
        ),
        pool,
    )
}

async fn seed_control_plane(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO deploy_site (
            id,uuid,tenant_id,organization_id,name,slug,site_type,status,runtime_config,
            metadata,created_at,updated_at,version
         ) VALUES (10,'site-10',7,9,'Shop','shop',1,1,'{}','{}',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',0)",
    )
    .execute(pool)
    .await
    .expect("seed deploy_site");
    sqlx::query(
        "INSERT INTO deploy_app (
            id,uuid,tenant_id,organization_id,name,slug,app_kind,app_status,site_id,
            default_environment,created_at,updated_at,version
         ) VALUES (20,'app-20',7,9,'Shop','shop','WEB','ACTIVE',10,'production',
                   '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',0)",
    )
    .execute(pool)
    .await
    .expect("seed deploy_app");
}

/// Inserts a compiled site revision and points the site's current revision
/// at it so hostname resolution can serve the descriptor.
async fn seed_revision(pool: &PgPool, revision_no: i64, descriptor: &serde_json::Value) -> String {
    let sha256 = sdkwork_utils_rust::crypto::sha256_hash(&serde_json::to_vec(descriptor).unwrap());
    sqlx::query(
        "INSERT INTO deploy_site_revision (
            id,uuid,tenant_id,organization_id,site_id,revision_no,environment,
            descriptor_schema_version,descriptor_json,descriptor_sha256,compiler_version,
            source_config_version,idempotency_key,request_sha256,validation_status,created_by,
            created_at
         ) VALUES ($1,$2,7,9,10,$3,'production',
            'sdkwork.website-runtime.v1',$4,$5,'test-compiler/1',1,'key','req','VALID',NULL,
            '2026-07-22T00:00:00Z')",
    )
    .bind(revision_no)
    .bind(format!("revision-{revision_no}"))
    .bind(revision_no)
    .bind(descriptor)
    .bind(&sha256)
    .execute(pool)
    .await
    .expect("seed deploy_site_revision");
    sqlx::query("UPDATE deploy_site SET current_revision_id = $1 WHERE id = 10")
        .bind(revision_no)
        .execute(pool)
        .await
        .expect("point current revision");
    sha256
}

fn descriptor() -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": "sdkwork.website-runtime.v1",
        "kind": "sdkwork.website-runtime.descriptor",
        "revisionUuid": "revision-0001",
        "siteUuid": "site-10",
        "tenantScopeHash": "1111111111111111111111111111111111111111111111111111111111111111",
        "environment": "production",
        "compilerVersion": "test-compiler/1",
        "descriptorSha256": "0".repeat(64),
        "siteDefaultVariantUuid": "variant-desktop",
        "bindings": [],
        "variants": [],
        "variantRules": [],
        "resources": [],
        "mounts": [],
        "deliveryPolicy": {},
        "securityPolicy": {},
        "limits": {},
        "observabilityPolicy": {}
    })
}

#[tokio::test]
async fn provisions_default_domains_and_bindings_idempotently() {
    let (repository, pool) = test_repository().await;
    let zones = repository
        .ensure_platform_app_zones(7, 9, Some(1))
        .await
        .expect("ensure zones");
    assert_eq!(zones, 14, "one platform zone per suffix");
    // Idempotent zone ensure.
    let zones_again = repository
        .ensure_platform_app_zones(7, 9, Some(1))
        .await
        .expect("ensure zones again");
    assert_eq!(zones_again, 0);

    let first = repository
        .provision_app_default_domains(7, 9, Some(1), "site-10", "shop", "production")
        .await
        .expect("provision");
    assert_eq!(first.created_zones, 0);
    assert_eq!(first.created_domains, 14);
    assert_eq!(first.created_bindings, 14);
    assert_eq!(first.existing_domains, 0);
    assert_eq!(first.existing_bindings, 0);
    assert_eq!(first.hostnames.len(), 14);
    assert!(first.hostnames.contains(&"shop.app.sdkwork.com".to_owned()));
    assert!(first.hostnames.contains(&"shop.app.86offer.cn".to_owned()));

    // Idempotent second pass: nothing new, everything reported as existing.
    let second = repository
        .provision_app_default_domains(7, 9, Some(1), "site-10", "shop", "production")
        .await
        .expect("re-provision");
    assert_eq!(second.created_domains, 0);
    assert_eq!(second.created_bindings, 0);
    assert_eq!(second.existing_domains, 14);
    assert_eq!(second.existing_bindings, 14);

    // Zones were created by the explicit ensure call.
    let zones: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deploy_dns_zone WHERE tenant_id = 7 AND deleted_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count zones");
    assert_eq!(zones, 14);
    let domains: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deploy_domain
         WHERE tenant_id = 7 AND verification_status = 'VERIFIED' AND deleted_at IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count domains");
    assert_eq!(domains, 14 + 14, "14 zone apexes + 14 app domains");
}

#[tokio::test]
async fn resolves_active_binding_to_latest_valid_revision() {
    let (repository, _) = test_repository().await;
    let _ = repository
        .ensure_platform_app_zones(7, 9, Some(1))
        .await
        .expect("ensure zones");
    let _ = repository
        .provision_app_default_domains(7, 9, Some(1), "site-10", "shop", "production")
        .await
        .expect("provision");
    let sha256 = seed_revision(&repository.pool().clone(), 1, &descriptor()).await;

    let resolved = repository
        .resolve_server_by_hostname("shop.app.sdkwork.com", "production")
        .await
        .expect("resolve")
        .expect("default app domain must resolve");
    assert_eq!(resolved.site_uuid, "site-10");
    assert_eq!(resolved.site_slug, "shop");
    assert_eq!(resolved.hostname, "shop.app.sdkwork.com");
    assert_eq!(resolved.path_prefix, "/");
    assert_eq!(resolved.action_type, "SERVE");
    assert_eq!(resolved.environment, "production");
    assert_eq!(resolved.revision_no, 1);
    assert_eq!(resolved.descriptor_sha256, sha256);
    assert_eq!(
        resolved.descriptor_json["siteUuid"],
        serde_json::json!("site-10")
    );

    // Case-insensitive lookup and other suffixes resolve too.
    let other = repository
        .resolve_server_by_hostname("SHOP.APP.BIRDBODER.COM", "production")
        .await
        .expect("resolve case-insensitive");
    assert!(other.is_none(), "unknown suffix must not resolve");

    // Unmatched custom hostnames resolve to None, not an error.
    let none = repository
        .resolve_server_by_hostname("mysite.example.com", "production")
        .await
        .expect("resolve custom");
    assert!(none.is_none());
}

#[tokio::test]
async fn resolves_custom_domains_and_respects_environment() {
    let (repository, pool) = test_repository().await;
    let _ = repository
        .ensure_platform_app_zones(7, 9, Some(1))
        .await
        .expect("ensure zones");
    let _ = repository
        .provision_app_default_domains(7, 9, Some(1), "site-10", "shop", "production")
        .await
        .expect("provision");
    // A user custom domain binding (VERIFIED domain + SERVE binding).
    let domain_row = sqlx::query(
        "INSERT INTO deploy_domain (
            id,uuid,tenant_id,organization_id,zone_id,hostname_ascii,hostname_type,
            verification_status,verified_at,status,created_by,updated_by,created_at,updated_at,version
         ) VALUES (90,'domain-90',7,9,
            (SELECT id FROM deploy_dns_zone WHERE tenant_id = 7 AND apex_hostname = 'app.sdkwork.com' LIMIT 1),
            'mysite.example.com','EXACT','VERIFIED','2026-07-22T00:00:00Z','ACTIVE',1,1,
            '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert custom domain");
    let domain_id: i64 = domain_row.try_get("id").expect("domain id");
    sqlx::query(
        "INSERT INTO deploy_site_binding (
            id,uuid,tenant_id,organization_id,site_id,binding_key,domain_id,hostname_ascii,
            environment,path_prefix,action_type,is_canonical,status,verified_at,activated_at,
            created_by,updated_by,created_at,updated_at,version
         ) VALUES (95,'binding-95',7,9,10,'custom-1',$1,'mysite.example.com','production',
            '/','SERVE',FALSE,'ACTIVE','2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',1,1,
            '2026-07-22T00:00:00Z','2026-07-22T00:00:00Z',1)",
    )
    .bind(domain_id)
    .execute(&pool)
    .await
    .expect("insert custom binding");
    let _ = seed_revision(&pool, 2, &descriptor()).await;

    let resolved = repository
        .resolve_server_by_hostname("mysite.example.com", "production")
        .await
        .expect("resolve")
        .expect("custom domain must resolve");
    assert_eq!(resolved.hostname, "mysite.example.com");
    assert_eq!(resolved.action_type, "SERVE");

    // The same hostname in another environment must not resolve.
    let other_env = repository
        .resolve_server_by_hostname("mysite.example.com", "development")
        .await
        .expect("resolve");
    assert!(other_env.is_none());
}
