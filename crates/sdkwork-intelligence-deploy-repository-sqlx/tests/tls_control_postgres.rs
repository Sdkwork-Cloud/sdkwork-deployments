//! TLS control plane integration tests: ACME account creation, certificate
//! order request with idempotency, the forward-only order state machine,
//! challenge result recording, and transactional version storage that
//! activates the certificate.
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
use sdkwork_deploy_contract::{
    CreateAcmeAccountRequest, CreateCertificateRequest, RequestCertificateOrderRequest,
};
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
        .with_applied_by("sdkwork-deploy-tls-test");
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

async fn create_certificate(repository: &DeployRepository, tenant_id: i64) -> (String, String) {
    let certificate = repository
        .create_certificate(
            tenant_id,
            Some(9),
            Some(11),
            "cert-api-sdkwork-dev",
            &CreateCertificateRequest {
                cert_name: "api.sdkwork.dev".to_owned(),
                domain_ids: vec!["domain-1".to_owned()],
                ca_profile: "LETS_ENCRYPT_PRODUCTION".to_owned(),
                preferred_key_algorithm: "ECDSA".to_owned(),
            },
        )
        .await
        .expect("create certificate");
    (certificate.id, certificate.identifiers[0].clone())
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn tls_order_state_machine_completes_end_to_end() {
    let repository = migrated_repository().await;

    let account = repository
        .create_acme_account(
            7,
            &CreateAcmeAccountRequest {
                ca_profile: "LETS_ENCRYPT_STAGING".to_owned(),
                directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_owned(),
                contact_email: "ops@sdkwork.dev".to_owned(),
                external_account_digest: None,
            },
        )
        .await
        .expect("create acme account");
    assert_eq!(account.ca_profile, "LETS_ENCRYPT_STAGING");

    let (certificate_id, hostname) = create_certificate(&repository, 7).await;

    let order = repository
        .request_certificate_order(
            7,
            &RequestCertificateOrderRequest {
                certificate_id: certificate_id.clone(),
                idempotency_key: "order-1".to_owned(),
                challenge_type: Some("HTTP_01".to_owned()),
            },
        )
        .await
        .expect("request order");
    assert_eq!(order.status, "REQUESTED");
    assert_eq!(order.requested_version_no, 1);

    // Idempotent replay returns the same order.
    let replay = repository
        .request_certificate_order(
            7,
            &RequestCertificateOrderRequest {
                certificate_id,
                idempotency_key: "order-1".to_owned(),
                challenge_type: Some("HTTP_01".to_owned()),
            },
        )
        .await
        .expect("idempotent replay");
    assert_eq!(replay.id, order.id);

    // One HTTP_01 challenge per identifier, referencing the hostname.
    let challenges = repository
        .list_certificate_challenges(7, &order.id, 1, 20)
        .await
        .expect("list challenges");
    assert_eq!(challenges.total, 1);
    assert_eq!(challenges.items[0].hostname, hostname);
    assert_eq!(challenges.items[0].challenge_type, "HTTP_01");
    assert_eq!(challenges.items[0].status, "PENDING");
    let challenge_id = challenges.items[0].id.clone();

    // Walk the canonical state machine to CHALLENGE_VALIDATING.
    let mut status = order.status.clone();
    for expected in [
        "ACCOUNT_READY",
        "ORDER_PENDING",
        "CHALLENGE_PRESENTING",
        "CHALLENGE_VALIDATING",
    ] {
        let advanced = repository
            .advance_certificate_order(7, &order.id, &status, expected)
            .await
            .expect("advance order");
        assert_eq!(advanced, expected, "advance to {expected}");
        status = expected.to_owned();
    }

    // A valid challenge result advances the order to FINALIZING.
    repository
        .record_challenge_result(7, &order.id, Some(&challenge_id), true, None)
        .await
        .expect("valid challenge result");
    let finalizing = repository
        .retrieve_certificate_order(7, &order.id)
        .await
        .expect("retrieve order");
    assert_eq!(finalizing.status, "FINALIZING");

    // Storing the issued version completes the order and activates the cert.
    let completed = repository
        .store_certificate_version(
            7,
            &order.id,
            1,
            "1111111111111111111111111111111111111111111111111111111111111111",
            "2222222222222222222222222222222222222222222222222222222222222222",
            "3333333333333333333333333333333333333333333333333333333333333333",
            "4444444444444444444444444444444444444444444444444444444444444444",
            "R3",
            "CN=api.sdkwork.dev",
            "ECDSA",
            "2026-08-01T00:00:00.000Z",
            "2026-10-30T00:00:00.000Z",
            "secret://tls/api.sdkwork.dev/v1",
        )
        .await
        .expect("store version");
    assert_eq!(completed.status, "VERSION_STORED");

    // The certificate now references the active version (visible via the
    // certificate list which joins the current version).
    let certificates = repository
        .list_certificates(7, 1, 20)
        .await
        .expect("list certificates");
    let certificate = certificates
        .items
        .iter()
        .find(|certificate| certificate.id == replay.certificate_id)
        .expect("certificate present");
    assert_eq!(certificate.current_version_id, Some(completed.id));
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn tls_orders_fail_closed_on_invalid_transitions_and_tenants() {
    let repository = migrated_repository().await;
    repository
        .create_acme_account(
            7,
            &CreateAcmeAccountRequest {
                ca_profile: "LETS_ENCRYPT_STAGING".to_owned(),
                directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_owned(),
                contact_email: "ops@sdkwork.dev".to_owned(),
                external_account_digest: None,
            },
        )
        .await
        .expect("create acme account");
    let (certificate_id, _) = create_certificate(&repository, 7).await;

    let order = repository
        .request_certificate_order(
            7,
            &RequestCertificateOrderRequest {
                certificate_id,
                idempotency_key: "order-fail".to_owned(),
                challenge_type: None,
            },
        )
        .await
        .expect("request order");

    // Skipping a state is rejected: REQUESTED -> CHALLENGE_PRESENTING is not
    // a canonical step (the optimistic UPDATE matches nothing).
    let skipped = repository
        .advance_certificate_order(7, &order.id, "REQUESTED", "CHALLENGE_PRESENTING")
        .await
        .expect("advance returns applied status");
    assert_eq!(skipped, "REQUESTED", "non-canonical step is a no-op");

    // Cross-tenant access fails closed.
    let cross_tenant = repository.retrieve_certificate_order(8, &order.id).await;
    assert!(
        cross_tenant.is_err(),
        "cross-tenant order read must fail closed"
    );

    // Failing the order with an error code lands on FAILED.
    repository
        .fail_certificate_order(7, &order.id, "ACME_NETWORK_ERROR")
        .await
        .expect("fail order");
    let failed = repository
        .retrieve_certificate_order(7, &order.id)
        .await
        .expect("retrieve failed order");
    assert_eq!(failed.status, "FAILED");
    assert_eq!(
        failed.last_error_code.as_deref(),
        Some("ACME_NETWORK_ERROR")
    );

    // Storing a version on a FAILED order is rejected.
    let store = repository
        .store_certificate_version(
            7,
            &order.id,
            1,
            "1111111111111111111111111111111111111111111111111111111111111111",
            "2222222222222222222222222222222222222222222222222222222222222222",
            "3333333333333333333333333333333333333333333333333333333333333333",
            "4444444444444444444444444444444444444444444444444444444444444444",
            "R3",
            "CN=api.sdkwork.dev",
            "ECDSA",
            "2026-08-01T00:00:00.000Z",
            "2026-10-30T00:00:00.000Z",
            "secret://tls/api.sdkwork.dev/v1",
        )
        .await;
    assert!(
        store.is_err(),
        "version storage on a failed order must be rejected"
    );
}
