//! Pagination keyset and secret-encryption integration tests
//! (PAGINATION_SPEC §6, SECURITY_SPEC secret-at-rest). Requires
//! `SDKWORK_DATABASE_TEST_POSTGRES_URL`; ignored by default like the other
//! PostgreSQL integration tests in this crate.

mod common;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::{
    AppKind, AuditLogQuery, CreateAppRequest, CreateEnvVariableRequest, DeployServiceErrorKind,
};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::repository::InsertAuditLogCommand;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;
use sqlx::PgPool;

fn repository(pool: PgPool) -> DeployRepository {
    DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(6).expect("Snowflake generator"),
        common::test_secret_key(),
    )
}

async fn create_app(repo: &DeployRepository, tenant_id: i64, name: &str) -> String {
    let response = repo
        .create_app(
            tenant_id,
            Some(0),
            Some(1),
            &CreateAppRequest {
                name: name.to_string(),
                slug: None,
                app_kind: AppKind::StaticWeb,
                app_type: Some(2),
                runtime_config: None,
                description: None,
                default_environment: None,
                idempotency_key: None,
            },
        )
        .await
        .expect("create app");
    response.id
}

/// 机密环境变量必须加密落库、响应掩码、列表永不明文（SECURITY_SPEC）。
#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn secret_env_variable_is_encrypted_at_rest_and_masked_in_responses() {
    let pool = common::postgres_pool().await;
    let repo = repository(pool.clone());
    let app_id = create_app(&repo, 7, "secret-app").await;

    let created = repo
        .create_env_variable(
            7,
            &app_id,
            &CreateEnvVariableRequest {
                environment: "production".to_string(),
                key: "API_TOKEN".to_string(),
                value: "super-secret-value".to_string(),
                is_secret: true,
            },
        )
        .await
        .expect("create secret env variable");
    // 响应必须掩码，绝不明文回传。
    assert_eq!(created.value, "***", "secret value must be masked");
    assert!(created.value != "super-secret-value");

    // 落库值必须是密文（base64(nonce||ciphertext)），不是明文也不是掩码。
    let stored: String =
        sqlx::query_scalar("SELECT value_encrypted FROM deploy_env_variable WHERE uuid = $1")
            .bind(&created.id)
            .fetch_one(&pool)
            .await
            .expect("read stored secret");
    assert_ne!(stored, "super-secret-value");
    assert_ne!(stored, "***");
    assert!(!stored.is_empty());

    // 列表返回掩码。
    let page = repo
        .list_env_variables(7, &app_id, Some("production"))
        .await
        .expect("list env variables");
    let listed = page
        .items
        .iter()
        .find(|item| item.id == created.id)
        .expect("created variable listed");
    assert_eq!(listed.value, "***", "listed secret must be masked");
}

/// 审计日志 keyset 翻页与无租户拒绝（fail-closed）。
#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn audit_logs_keyset_pages_and_requires_tenant() {
    let pool = common::postgres_pool().await;
    let repo = repository(pool.clone());

    for index in 0..5 {
        repo.insert_audit_log(InsertAuditLogCommand {
            tenant_id: 7,
            organization_id: 0,
            operator_id: 1,
            action: format!("action-{index}"),
            target_type: "app".to_string(),
            target_id: None,
            target_uuid: None,
        })
        .await
        .expect("insert audit log");
    }

    let query = AuditLogQuery {
        page_size: 2,
        ..AuditLogQuery::default()
    };
    let mut collected = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let page = repo
            .list_audit_logs(Some(7), &query, cursor.as_deref())
            .await
            .expect("list audit page");
        collected.extend(page.items.iter().map(|item| item.id.clone()));
        match (page.has_more, page.next_cursor) {
            (Some(true), Some(next)) => cursor = Some(next),
            _ => break,
        }
    }
    assert_eq!(collected.len(), 5, "all audit rows collected");
    let unique: std::collections::HashSet<_> = collected.iter().collect();
    assert_eq!(unique.len(), 5, "no duplicate audit rows across pages");

    // 无租户上下文的调用必须被拒绝（跨租户越权防线）。
    let denied = repo
        .list_audit_logs(None, &query, None)
        .await
        .expect_err("tenant-less audit listing must be rejected");
    assert_eq!(denied.kind(), DeployServiceErrorKind::Forbidden);
}
