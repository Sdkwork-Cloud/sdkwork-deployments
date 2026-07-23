use std::{fs, path::PathBuf};

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::CreateDomainRequest;
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;
use sqlx::{any::AnyPoolOptions, AnyPool};

const SQLITE_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/sqlite/0001_deploy_baseline.sql");

struct SqliteTestFile(PathBuf);

impl Drop for SqliteTestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(self.0.with_extension("db-shm"));
        let _ = fs::remove_file(self.0.with_extension("db-wal"));
    }
}

async fn test_repository() -> (DeployRepository, SqliteTestFile) {
    sqlx::any::install_default_drivers();
    let relative_path = PathBuf::from(format!(
        "target/domain-verification-{}.db",
        sdkwork_database_id::uuid_v4()
    ));
    fs::create_dir_all("target").expect("create Cargo target directory");
    let database_url = format!("sqlite://{}?mode=rwc", relative_path.display());
    let pool = AnyPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("connect file-backed SQLite");
    sqlx::raw_sql(SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply SQLite baseline");
    seed_site(&pool).await;

    (
        DeployRepository::new(
            pool,
            SnowflakeIdGenerator::new(3).expect("Snowflake generator"),
        ),
        SqliteTestFile(relative_path),
    )
}

async fn seed_site(pool: &AnyPool) {
    sqlx::query(
        "INSERT INTO deploy_site (
            id,uuid,tenant_id,organization_id,name,slug,site_type,status,runtime_config,
            metadata,created_at,updated_at,version
         ) VALUES (10,'site-1',7,9,'Docs','docs',1,1,'{}','{}',
                   '2026-07-23T00:00:00Z','2026-07-23T00:00:00Z',0)",
    )
    .execute(pool)
    .await
    .expect("insert site");
}

#[tokio::test]
async fn domain_activation_requires_the_current_exact_verification_token() {
    let (repository, _database) = test_repository().await;
    let domain = repository
        .create_domain(
            7,
            "site-1",
            &CreateDomainRequest {
                hostname: "docs.example.com".to_owned(),
                is_primary: true,
                ssl_enabled: true,
                ssl_provider: Some("letsencrypt".to_owned()),
            },
        )
        .await
        .expect("create pending domain");

    let pending = repository
        .domain_verification_challenge(7, "site-1", &domain.id)
        .await
        .expect("load pending challenge");
    let token = pending.token.expect("pending challenge token");
    assert!(!pending.verified);

    assert!(!repository
        .confirm_domain_verification(7, "site-1", &domain.id, "wrong-token")
        .await
        .expect("reject wrong token"));
    assert!(
        !repository
            .domain_verification_challenge(7, "site-1", &domain.id)
            .await
            .expect("reload pending challenge")
            .verified
    );

    assert!(repository
        .confirm_domain_verification(7, "site-1", &domain.id, &token)
        .await
        .expect("confirm exact token"));
    let verified = repository
        .domain_verification_challenge(7, "site-1", &domain.id)
        .await
        .expect("load verified domain");
    assert!(verified.verified);
    assert!(verified.token.is_none());
    assert!(!repository
        .confirm_domain_verification(7, "site-1", &domain.id, &token)
        .await
        .expect("repeat confirmation is idempotent"));
}
