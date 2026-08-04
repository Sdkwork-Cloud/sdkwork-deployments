//! In-process Deploy service and runtime-publication composition.

use std::sync::Arc;

use sdkwork_database_id::{NodeLease, SnowflakeIdGenerator, SnowflakeNodeAllocator};
use sdkwork_deploy_content_provider_port::{
    content_provider_port_from_env, website_provider_event_delivery_port_from_env,
};
use sdkwork_deploy_database_host::bootstrap_deploy_database_from_env;
use sdkwork_deploy_drive_port::deploy_drive_port_from_env;
use sdkwork_deploy_web_port::{
    DeployWebRuntimePort, SdkWebRuntimeFacade, UnconfiguredWebRuntimePort,
};
use sdkwork_intelligence_deploy_repository_sqlx::DeployRepository;
use sdkwork_intelligence_deploy_service::{
    DeployRepositoryPort, DeployRuntimeAssignmentRepositoryPort, DeployService,
    RuntimePublicationService,
};

mod domain_verification;

use domain_verification::DnsTxtDomainOwnershipVerifier;

pub struct DeployServiceHost {
    pub service: Arc<DeployService>,
}

pub struct RuntimePublicationHost {
    pub publication: Arc<RuntimePublicationService>,
}

async fn snowflake_from_env() -> Result<(SnowflakeIdGenerator, Option<NodeLease>), String> {
    if sdkwork_deploy_core::deploy_is_production_like_environment() {
        if std::env::var("SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID").is_ok() {
            return Err(
                "static SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID is forbidden in production-like environments"
                    .to_owned(),
            );
        }
        let (generator, lease) =
            SnowflakeNodeAllocator::allocate_generator_from_env("sdkwork-deploy", "DEPLOY")
                .await
                .map_err(|error| {
                    format!("allocate Deploy Snowflake database node lease failed: {error}")
                })?;
        return Ok((generator, Some(lease)));
    }

    let node_id = match std::env::var("SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID") {
        Ok(value) => value
            .parse::<u16>()
            .map_err(|error| format!("invalid SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID: {error}"))?,
        Err(_) => 1,
    };
    SnowflakeIdGenerator::new(node_id)
        .map(|generator| (generator, None))
        .map_err(|error| error.to_string())
}

async fn repository_from_env() -> Result<Arc<DeployRepository>, String> {
    let database = bootstrap_deploy_database_from_env().await?;
    let pool = database
        .pool()
        .as_postgres()
        .cloned()
        .ok_or_else(|| "Deploy authoritative database must use PostgreSQL".to_owned())?;
    let (id_generator, node_lease) = snowflake_from_env().await?;
    let secret_key = secret_key_from_env()?;
    Ok(Arc::new(match node_lease {
        Some(node_lease) => {
            DeployRepository::new_with_node_lease(pool, id_generator, node_lease, secret_key)
        }
        None => DeployRepository::new(pool, id_generator, secret_key),
    }))
}

/// AES-256 key derivation for secrets at rest, aligned with the Web Server
/// repository contract: production-like environments require
/// `SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY`; development falls back to a
/// derived constant with a warning so local runs stay functional.
fn secret_key_from_env() -> Result<[u8; 32], String> {
    let production_like = sdkwork_deploy_core::deploy_is_production_like_environment();
    let raw = match std::env::var("SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY") {
        Ok(value) => value,
        Err(_) if !production_like => {
            tracing::warn!(
                "SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY missing; using development-only derived key"
            );
            "sdkwork-deploy-development-secret-key".to_string()
        }
        Err(_) => {
            return Err(
                "SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY is required in production-like environments"
                    .to_string(),
            );
        }
    };
    Ok(sdkwork_utils_rust::crypto::derive_aes_256_key(
        raw.as_bytes(),
        b"sdkwork-deploy-env",
        b"env-variable-encryption",
    ))
}

fn web_runtime_from_env() -> Result<Arc<dyn DeployWebRuntimePort>, String> {
    match SdkWebRuntimeFacade::from_env() {
        Ok(facade) => Ok(Arc::new(facade)),
        Err(error) if sdkwork_deploy_core::deploy_is_production_like_environment() => {
            Err(format!("configure Web runtime publication failed: {error}"))
        }
        Err(_) => Ok(Arc::new(UnconfiguredWebRuntimePort)),
    }
}

pub async fn bootstrap_runtime_publication_host_from_env() -> Result<RuntimePublicationHost, String>
{
    let repository = repository_from_env().await?;
    let runtime_repository = repository as Arc<dyn DeployRuntimeAssignmentRepositoryPort>;
    Ok(RuntimePublicationHost {
        publication: Arc::new(RuntimePublicationService::new_with_provider_event_delivery(
            runtime_repository,
            web_runtime_from_env()?,
            website_provider_event_delivery_port_from_env()?,
        )),
    })
}

pub async fn bootstrap_deploy_service_host_from_env() -> Result<DeployServiceHost, String> {
    let repository = repository_from_env().await?;
    let service_repository = repository.clone() as Arc<dyn DeployRepositoryPort>;
    let runtime_repository = repository as Arc<dyn DeployRuntimeAssignmentRepositoryPort>;
    let runtime_publication = Arc::new(RuntimePublicationService::new(
        runtime_repository,
        web_runtime_from_env()?,
    ));
    Ok(DeployServiceHost {
        service: Arc::new(DeployService::new_with_runtime_publication(
            service_repository,
            deploy_drive_port_from_env()?,
            content_provider_port_from_env()?,
            Arc::new(DnsTxtDomainOwnershipVerifier::from_system_config()?),
            runtime_publication,
        )),
    })
}
