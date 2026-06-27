//! Deploy runtime bootstrap: database lifecycle + repository + service assembly.

use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::create_any_pool_from_config;
use sdkwork_deploy_database_host::bootstrap_deploy_database_from_env;
use sdkwork_database_id::SnowflakeIdGenerator;
use sdkwork_intelligence_deploy_service::DeployService;
use sqlx::AnyPool;
use std::sync::Arc;

use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

use crate::DeployRepository;

/// Bootstrapped deploy application runtime.
pub struct DeployRuntime {
    pub service: DeployService,
}

fn snowflake_from_env() -> Result<SnowflakeIdGenerator, String> {
    let node_id = std::env::var("SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(1);
    SnowflakeIdGenerator::new(node_id).map_err(|error| error.to_string())
}

async fn any_pool_from_env() -> Result<AnyPool, String> {
    let _ = dotenvy::dotenv();
    let config = DatabaseConfig::from_env("DEPLOY")
        .map_err(|error| format!("read deploy database config failed: {error}"))?;
    create_any_pool_from_config(config)
        .await
        .map_err(|error| format!("create deploy any pool failed: {error}"))
}

/// Bootstrap database lifecycle, repository, and service from environment variables.
pub async fn bootstrap_deploy_runtime_from_env() -> Result<DeployRuntime, String> {
    bootstrap_deploy_database_from_env().await?;
    let pool = any_pool_from_env().await?;
    let id_generator = snowflake_from_env()?;
    let repository =
        Arc::new(DeployRepository::new(pool, id_generator)) as Arc<dyn DeployRepositoryPort>;
    Ok(DeployRuntime {
        service: DeployService::new(repository),
    })
}
