use sdkwork_deploy_contract::{
    CreateNodeClusterRequest, DeployServiceError, DeployServiceResult, NodeClusterPage,
    NodeClusterResponse, UpdateNodeClusterRequest,
};
use sqlx::{
    postgres::{PgArguments, PgRow},
    AssertSqlSafe, Postgres, Row,
};

use crate::support::{datetime_from_row, new_uuid, next_id, now_rfc3339, pagination, store_error};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_node_clusters_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<NodeClusterPage> {
        let (page, page_size, offset) = pagination(page, page_size);

        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM deploy_node_cluster WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count deploy_node_cluster", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT c.uuid, c.name, c.description, c.region, c.status,
                    COUNT(s.id) AS node_count, c.created_at
             FROM deploy_node_cluster c
             LEFT JOIN deploy_server s ON s.cluster_id = c.id AND s.tenant_id = c.tenant_id
             WHERE c.tenant_id = $1
             GROUP BY c.id
             ORDER BY c.updated_at DESC, c.id DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_node_cluster", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_node_cluster_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_node_cluster row: {error}"))
            })?);
        }

        Ok(NodeClusterPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_node_cluster_repo(
        &self,
        tenant_id: i64,
        request: &CreateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_node_cluster (
                id, uuid, tenant_id, name, description, region, status, metadata,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 0, '{}', CAST($7 AS TIMESTAMPTZ), CAST($7 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(&request.name)
        .bind(request.description.as_deref())
        .bind(request.region.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_node_cluster", error))?;

        Ok(NodeClusterResponse {
            id: uuid,
            name: request.name.clone(),
            description: request.description.clone(),
            region: request.region.clone(),
            status: 0,
            node_count: 0,
            created_at: now,
        })
    }

    pub(super) async fn update_node_cluster_repo(
        &self,
        tenant_id: i64,
        cluster_id: &str,
        request: &UpdateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse> {
        let row =
            sqlx::query("SELECT id FROM deploy_node_cluster WHERE tenant_id = $1 AND uuid = $2")
                .bind(tenant_id)
                .bind(cluster_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("resolve deploy_node_cluster id", error))?;
        let internal_id: i64 = row
            .and_then(|row| row.try_get::<i64, _>("id").ok())
            .ok_or_else(|| DeployServiceError::not_found("cluster not found"))?;

        let now = now_rfc3339();
        let mut sets: Vec<String> = Vec::new();
        let mut binds: Vec<BindValue> = Vec::new();
        if let Some(description) = request.description.as_deref() {
            binds.push(BindValue::Str(description));
            sets.push(format!("description = ${}", binds.len()));
        }
        if let Some(region) = request.region.as_deref() {
            binds.push(BindValue::Str(region));
            sets.push(format!("region = ${}", binds.len()));
        }
        if let Some(status) = request.status {
            binds.push(BindValue::I32(status));
            sets.push(format!("status = ${}", binds.len()));
        }

        if !sets.is_empty() {
            let updated_index = binds.len() + 1;
            let tenant_index = binds.len() + 2;
            let id_index = binds.len() + 3;
            let sql = format!(
                "UPDATE deploy_node_cluster SET {}, updated_at = CAST(${updated_index} AS TIMESTAMPTZ), version = version + 1
                 WHERE tenant_id = ${tenant_index} AND id = ${id_index}",
                sets.join(", ")
            );
            apply_binds(sqlx::query(AssertSqlSafe(&*sql)), &binds)
                .bind(&now)
                .bind(tenant_id)
                .bind(internal_id)
                .execute(&self.pool)
                .await
                .map_err(|error| store_error("update deploy_node_cluster", error))?;
        }

        self.fetch_node_cluster_by_internal_id(tenant_id, internal_id)
            .await
    }

    async fn fetch_node_cluster_by_internal_id(
        &self,
        tenant_id: i64,
        internal_id: i64,
    ) -> DeployServiceResult<NodeClusterResponse> {
        let row = sqlx::query(
            "SELECT c.uuid, c.name, c.description, c.region, c.status,
                    COUNT(s.id) AS node_count, c.created_at
             FROM deploy_node_cluster c
             LEFT JOIN deploy_server s ON s.cluster_id = c.id AND s.tenant_id = c.tenant_id
             WHERE c.tenant_id = $1 AND c.id = $2
             GROUP BY c.id",
        )
        .bind(tenant_id)
        .bind(internal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("fetch deploy_node_cluster", error))?;

        row.map(|row| map_node_cluster_row(&row))
            .transpose()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_node_cluster row: {error}"))
            })?
            .ok_or_else(|| DeployServiceError::not_found("cluster not found"))
    }
}

fn map_node_cluster_row(row: &PgRow) -> Result<NodeClusterResponse, sqlx::Error> {
    Ok(NodeClusterResponse {
        id: row.try_get("uuid")?,
        name: row.try_get("name")?,
        description: row.try_get::<Option<String>, _>("description")?,
        region: row.try_get::<Option<String>, _>("region")?,
        status: row.try_get("status")?,
        node_count: row.try_get("node_count")?,
        created_at: datetime_from_row(row, "created_at")?,
    })
}

enum BindValue<'a> {
    I32(i32),
    Str(&'a str),
}

fn apply_binds<'q>(
    mut query: sqlx::query::Query<'q, Postgres, PgArguments>,
    binds: &[BindValue<'_>],
) -> sqlx::query::Query<'q, Postgres, PgArguments> {
    for value in binds {
        query = match value {
            BindValue::I32(value) => query.bind(*value),
            BindValue::Str(value) => query.bind(*value),
        };
    }
    query
}
