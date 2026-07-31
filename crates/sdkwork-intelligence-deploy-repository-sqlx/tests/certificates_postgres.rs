mod common;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::CreateCertificateRequest;
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

async fn seed_domains(pool: &sqlx::PgPool) {
    sqlx::raw_sql(
        "INSERT INTO deploy_dns_zone (
            id,uuid,tenant_id,organization_id,apex_hostname,status
         ) VALUES
            (10,'zone-primary',7,9,'example.com','ACTIVE'),
            (11,'zone-foreign',8,9,'foreign.example','ACTIVE');
         INSERT INTO deploy_domain (
            id,uuid,tenant_id,organization_id,zone_id,hostname_ascii,hostname_type,
            verification_status,verified_at,status
         ) VALUES
            (20,'domain-apex',7,9,10,'example.com','EXACT','VERIFIED',NOW(),'ACTIVE'),
            (21,'domain-docs',7,9,10,'docs.example.com','EXACT','VERIFIED',NOW(),'ACTIVE'),
            (22,'domain-pending',7,9,10,'pending.example.com','EXACT','PENDING',NULL,'ACTIVE'),
            (23,'domain-foreign',8,9,11,'foreign.example','EXACT','VERIFIED',NOW(),'ACTIVE');",
    )
    .execute(pool)
    .await
    .expect("seed certificate hostname resources");
}

fn request(cert_name: &str, domain_ids: &[&str]) -> CreateCertificateRequest {
    CreateCertificateRequest {
        cert_name: cert_name.to_owned(),
        domain_ids: domain_ids.iter().map(|value| (*value).to_owned()).collect(),
        ca_profile: "LETS_ENCRYPT_STAGING".to_owned(),
        preferred_key_algorithm: "ECDSA".to_owned(),
    }
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn certificates_support_many_to_many_hostnames_and_strict_creation_boundaries() {
    let pool = common::postgres_pool().await;
    seed_domains(&pool).await;
    let repository = DeployRepository::new(
        pool.clone(),
        SnowflakeIdGenerator::new(5).expect("Snowflake generator"),
    );

    let multi_hostname_request = request("Primary ECDSA", &["domain-apex", "domain-docs"]);
    let first = repository
        .create_certificate(
            7,
            Some(9),
            Some(11),
            "certificate-primary-ecdsa",
            &multi_hostname_request,
        )
        .await
        .expect("create one certificate for multiple hostnames");
    assert_eq!(
        first.identifiers,
        vec!["docs.example.com".to_owned(), "example.com".to_owned()]
    );
    assert_eq!(first.status, "PENDING");

    let replay = repository
        .create_certificate(
            7,
            Some(9),
            Some(11),
            "certificate-primary-ecdsa",
            &multi_hostname_request,
        )
        .await
        .expect("replay identical certificate request");
    assert_eq!(replay.id, first.id);

    let second = repository
        .create_certificate(
            7,
            Some(9),
            Some(11),
            "certificate-primary-rsa",
            &CreateCertificateRequest {
                preferred_key_algorithm: "RSA".to_owned(),
                ..request("Primary RSA", &["domain-apex"])
            },
        )
        .await
        .expect("associate another certificate with the same hostname");
    assert_ne!(second.id, first.id);
    let apex_certificate_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT ci.certificate_id)
         FROM deploy_certificate_identifier ci
         JOIN deploy_domain d ON d.id = ci.domain_id
         WHERE d.uuid = 'domain-apex'",
    )
    .fetch_one(&pool)
    .await
    .expect("count certificates for apex hostname");
    assert_eq!(apex_certificate_count, 2);

    let idempotency_conflict = repository
        .create_certificate(
            7,
            Some(9),
            Some(11),
            "certificate-primary-ecdsa",
            &request("Changed request", &["domain-apex"]),
        )
        .await
        .expect_err("reject idempotency key reuse with another request");
    assert!(idempotency_conflict.to_string().contains("Idempotency-Key"));

    for (key, invalid_request) in [
        (
            "certificate-duplicate-domain",
            request("Duplicate", &["domain-apex", "domain-apex"]),
        ),
        (
            "certificate-pending-domain",
            request("Pending", &["domain-pending"]),
        ),
        (
            "certificate-cross-tenant-domain",
            request("Foreign", &["domain-foreign"]),
        ),
    ] {
        assert!(repository
            .create_certificate(7, Some(9), Some(11), key, &invalid_request)
            .await
            .is_err());
    }
}
