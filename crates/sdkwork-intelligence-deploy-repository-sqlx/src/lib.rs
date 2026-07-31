use sdkwork_database_id::{NodeLease, SnowflakeIdGenerator};
use sqlx::PgPool;

mod artifacts;
mod audit;
mod certificates;
mod deployments;
mod domain_zones;
mod env_variables;
mod health_checks;
mod nginx_configs;
mod nginx_orchestrator;
mod nginx_security;
mod port;
mod releases;
mod runtime_assignments;
mod servers;
mod site_composition;
mod sites;
mod support;
mod upload_sessions;

#[derive(Clone)]
pub struct DeployRepository {
    pool: PgPool,
    id_generator: SnowflakeIdGenerator,
    _node_lease: Option<NodeLease>,
}

impl DeployRepository {
    pub fn new(pool: PgPool, id_generator: SnowflakeIdGenerator) -> Self {
        Self {
            pool,
            id_generator,
            _node_lease: None,
        }
    }

    pub fn new_with_node_lease(
        pool: PgPool,
        id_generator: SnowflakeIdGenerator,
        node_lease: NodeLease,
    ) -> Self {
        Self {
            pool,
            id_generator,
            _node_lease: Some(node_lease),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn id_generator(&self) -> &SnowflakeIdGenerator {
        &self.id_generator
    }
}
