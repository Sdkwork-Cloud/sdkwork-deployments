mod common;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::{CreateDomainHostnameRequest, CreateDomainZoneRequest};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn domain_activation_requires_external_evidence_for_the_current_attempt() {
    let pool = common::postgres_pool().await;
    let repository = DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(3).expect("Snowflake generator"),
    );
    let apex = format!(
        "verify{}.dev",
        sdkwork_database_id::uuid_v4().replace('-', "")
    );
    let zone = repository
        .create_domain_zone(
            7,
            Some(9),
            Some(11),
            &CreateDomainZoneRequest {
                apex_hostname: apex,
                display_name: Some("Verification test".to_owned()),
                dns_provider: Some("manual".to_owned()),
                provider_zone_ref: None,
            },
        )
        .await
        .expect("create root domain zone");
    let hostname = repository
        .create_domain_hostname(
            7,
            Some(11),
            &zone.id,
            &CreateDomainHostnameRequest {
                relative_name: "docs".to_owned(),
            },
        )
        .await
        .expect("create pending hostname");

    let pending = repository
        .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
        .await
        .expect("load pending challenge");
    let token = pending.token.expect("new challenge returns the proof once");
    let verification_id = pending
        .verification_id
        .expect("pending challenge verification id");
    let proof_sha256 = pending.proof_sha256.expect("pending proof digest");
    assert!(!pending.verified);
    assert_eq!(
        sdkwork_utils_rust::crypto::sha256_hash(token.as_bytes()),
        proof_sha256
    );

    assert!(!repository
        .confirm_domain_hostname_verification(
            7,
            &zone.id,
            &hostname.id,
            &verification_id,
            &"0".repeat(64),
            "test-resolver",
        )
        .await
        .expect("reject mismatched observation"));
    assert!(
        !repository
            .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
            .await
            .expect("reload pending challenge")
            .verified
    );

    assert!(repository
        .confirm_domain_hostname_verification(
            7,
            &zone.id,
            &hostname.id,
            &verification_id,
            &proof_sha256,
            "test-resolver",
        )
        .await
        .expect("confirm exact observed digest"));
    let verified = repository
        .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
        .await
        .expect("load verified hostname");
    assert!(verified.verified);
    assert!(verified.token.is_none());
    assert!(!repository
        .confirm_domain_hostname_verification(
            7,
            &zone.id,
            &hostname.id,
            &verification_id,
            &proof_sha256,
            "test-resolver",
        )
        .await
        .expect("repeat confirmation is idempotent"));
}
