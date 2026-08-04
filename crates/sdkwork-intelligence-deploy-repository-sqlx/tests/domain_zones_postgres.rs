mod common;

use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_deploy_contract::{
    CreateDomainHostnameRequest, CreateDomainZoneRequest, ListDomainZonesQuery,
    UpdateDomainHostnameRequest,
};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_TEST_POSTGRES_URL"]
async fn postgres_domain_zone_lifecycle_enforces_resource_boundaries() {
    let pool = common::postgres_pool().await;
    let repository = DeployRepository::new(
        pool,
        SnowflakeIdGenerator::new(4).expect("Snowflake generator"),
        common::test_secret_key(),
    );
    let apex = format!(
        "zone{}.dev",
        sdkwork_database_id::uuid_v4().replace('-', "")
    );

    let zone = repository
        .create_domain_zone(
            7,
            Some(9),
            Some(11),
            &CreateDomainZoneRequest {
                apex_hostname: apex.clone(),
                display_name: Some("Production zone".to_owned()),
                dns_provider: Some("manual".to_owned()),
                provider_zone_ref: None,
            },
        )
        .await
        .expect("create root domain zone");
    assert_eq!(zone.apex_hostname, apex);
    assert_eq!(zone.hostname_count, 1);

    let listed = repository
        .list_domain_zones(
            7,
            &ListDomainZonesQuery {
                page: 1,
                page_size: 20,
                status: Some("ACTIVE".to_owned()),
                keyword: Some("Production".to_owned()),
            },
        )
        .await
        .expect("list root domain zones");
    assert!(listed.items.iter().any(|item| item.id == zone.id));

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
        .expect("create child hostname");
    assert_eq!(hostname.hostname, format!("docs.{apex}"));

    let first_challenge = repository
        .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
        .await
        .expect("create verification challenge");
    assert!(first_challenge.token.is_some());
    let repeated_challenge = repository
        .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
        .await
        .expect("reload verification challenge");
    assert!(repeated_challenge.token.is_none());
    assert_eq!(
        repeated_challenge.verification_id,
        first_challenge.verification_id
    );

    // Renaming keeps the hostname in the zone but resets ownership
    // verification and expires the previous challenge (the TXT record
    // changes with the DNS name).
    let renamed = repository
        .update_domain_hostname(
            7,
            Some(11),
            &zone.id,
            &hostname.id,
            &UpdateDomainHostnameRequest {
                relative_name: "docs2".to_owned(),
            },
        )
        .await
        .expect("rename child hostname");
    assert_eq!(renamed.hostname, format!("docs2.{apex}"));
    assert_eq!(renamed.hostname_type, "EXACT");
    assert_eq!(renamed.verification_status, "PENDING");
    assert!(renamed.verified_at.is_none());
    let renamed_challenge = repository
        .domain_hostname_verification_challenge(7, &zone.id, &hostname.id)
        .await
        .expect("challenge after rename");
    assert_ne!(
        renamed_challenge.verification_id, first_challenge.verification_id,
        "rename must expire the old verification challenge"
    );
    assert!(
        renamed_challenge
            .record_name
            .as_deref()
            .is_some_and(|record| record.contains("docs2")),
        "challenge record must target the renamed hostname"
    );

    // The apex hostname is owned by the zone and cannot be renamed, and a
    // hostname cannot be renamed back onto the apex.
    let apex_hostname = repository
        .list_domain_hostnames(7, &zone.id, 1, 20)
        .await
        .expect("list apex hostname")
        .items
        .into_iter()
        .find(|item| item.relative_name == "@")
        .expect("apex hostname");
    assert!(
        repository
            .update_domain_hostname(
                7,
                Some(11),
                &zone.id,
                &apex_hostname.id,
                &UpdateDomainHostnameRequest {
                    relative_name: "renamed-apex".to_owned(),
                },
            )
            .await
            .is_err(),
        "apex hostname must not be renamed"
    );
    assert!(
        repository
            .update_domain_hostname(
                7,
                Some(11),
                &zone.id,
                &hostname.id,
                &UpdateDomainHostnameRequest {
                    relative_name: "@".to_owned(),
                },
            )
            .await
            .is_err(),
        "hostname must not be renamed onto the zone apex"
    );

    assert!(repository.delete_domain_zone(7, &zone.id).await.is_err());
    repository
        .delete_domain_hostname(7, &zone.id, &hostname.id)
        .await
        .expect("delete unbound child hostname");
    let apex_hostname = repository
        .list_domain_hostnames(7, &zone.id, 1, 20)
        .await
        .expect("list apex hostname")
        .items
        .into_iter()
        .find(|item| item.relative_name == "@")
        .expect("apex hostname");
    assert!(
        repository
            .delete_domain_hostname(7, &zone.id, &apex_hostname.id)
            .await
            .is_err(),
        "the apex hostname belongs to the zone and cannot be deleted independently"
    );
    repository
        .delete_domain_zone(7, &zone.id)
        .await
        .expect("delete root domain zone with its apex hostname");

    let recreated = repository
        .create_domain_zone(
            7,
            Some(9),
            Some(11),
            &CreateDomainZoneRequest {
                apex_hostname: apex,
                display_name: None,
                dns_provider: None,
                provider_zone_ref: None,
            },
        )
        .await
        .expect("reuse soft-deleted apex");
    let recreated_apex = repository
        .list_domain_hostnames(7, &recreated.id, 1, 20)
        .await
        .expect("list recreated apex")
        .items
        .into_iter()
        .next()
        .expect("recreated apex hostname");
    assert!(
        repository
            .delete_domain_hostname(7, &recreated.id, &recreated_apex.id)
            .await
            .is_err(),
        "the recreated apex hostname belongs to the zone and cannot be deleted independently"
    );
    repository
        .delete_domain_zone(7, &recreated.id)
        .await
        .expect("delete recreated zone");
}
