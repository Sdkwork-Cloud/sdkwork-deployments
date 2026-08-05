use sdkwork_database_id::{NodeLease, SnowflakeIdGenerator};
use sqlx::PgPool;

mod app_deployments;
mod app_releases;
mod apps;
mod artifacts;
mod audit;
mod builds;
mod certificates;
mod database_profiles;
mod deployments;
mod domain_zones;
mod entitlement;
mod env_variables;
mod environments;
mod health_checks;
mod nginx_configs;
mod nginx_orchestrator;
mod nginx_security;
mod node_clusters;
mod port;
mod releases;
mod retention;
mod runtime_assignments;
mod servers;
mod site_composition;
mod sites;
mod source_events;
mod support;
mod tls_control;
mod upload_sessions;
mod usage;

#[derive(Clone)]
pub struct DeployRepository {
    pool: PgPool,
    id_generator: SnowflakeIdGenerator,
    secret_key: [u8; 32],
    _node_lease: Option<NodeLease>,
}

impl DeployRepository {
    pub fn new(pool: PgPool, id_generator: SnowflakeIdGenerator, secret_key: [u8; 32]) -> Self {
        Self {
            pool,
            id_generator,
            secret_key,
            _node_lease: None,
        }
    }

    pub fn new_with_node_lease(
        pool: PgPool,
        id_generator: SnowflakeIdGenerator,
        node_lease: NodeLease,
        secret_key: [u8; 32],
    ) -> Self {
        Self {
            pool,
            id_generator,
            secret_key,
            _node_lease: Some(node_lease),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }

    /// AES-256 key used to protect environment-variable secrets at rest
    /// (derived from `SDKWORK_DEPLOY_SECRET_ENCRYPTION_KEY` at bootstrap).
    pub fn secret_key(&self) -> &[u8; 32] {
        &self.secret_key
    }
}
