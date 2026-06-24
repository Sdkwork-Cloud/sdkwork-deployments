use sdkwork_id_core::SnowflakeIdGenerator;
use sqlx::AnyPool;

mod audit;
mod certificates;
mod deployments;
mod domains;
mod env_variables;
mod health_checks;
mod nginx_configs;
mod nginx_orchestrator;
mod nginx_security;
mod port;
mod runtime;
mod servers;
mod sites;
mod support;

pub use runtime::{bootstrap_deploy_runtime_from_env, DeployRuntime};

#[derive(Clone)]
pub struct DeployRepository {
    pool: AnyPool,
    id_generator: SnowflakeIdGenerator,
}

impl DeployRepository {
    pub fn new(pool: AnyPool, id_generator: SnowflakeIdGenerator) -> Self {
        Self { pool, id_generator }
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }
}
