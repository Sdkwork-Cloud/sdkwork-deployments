use sdkwork_deploy_contract::{
    CreateServerRequest, DeployServiceError, DeployServiceResult, ServerPage, ServerResponse,
    UpdateServerRequest,
};
use sqlx::{
    postgres::{PgArguments, PgRow},
    AssertSqlSafe, Postgres, Row,
};

use crate::support::{
    datetime_from_row, new_uuid, next_id, now_rfc3339, pagination,
    resolve_node_cluster_internal_id, store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_servers_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
        cluster_id: Option<String>,
    ) -> DeployServiceResult<ServerPage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let mut count_sql =
            String::from("SELECT COUNT(*) AS total FROM deploy_server s WHERE s.tenant_id = $1");
        let mut list_sql = String::from(
            "SELECT s.uuid, s.name, s.host, s.ssh_port, s.cluster_id,
                    c.uuid AS cluster_uuid, c.name AS cluster_name,
                    s.node_role, s.status, s.ssh_user, s.description, s.created_at
             FROM deploy_server s
             LEFT JOIN deploy_node_cluster c ON c.id = s.cluster_id AND c.tenant_id = s.tenant_id
             WHERE s.tenant_id = $1",
        );
        let mut binds: Vec<BindValue> = Vec::new();
        if let Some(cluster_uuid) = cluster_id.as_deref() {
            let cluster_internal_id =
                resolve_node_cluster_internal_id(&self.pool, tenant_id, cluster_uuid).await?;
            let index = binds.len() + 2;
            let clause = format!(" AND s.cluster_id = ${index}");
            count_sql.push_str(&clause);
            list_sql.push_str(&clause);
            binds.push(BindValue::I64(cluster_internal_id));
        }

        let limit_index = binds.len() + 2;
        let offset_index = binds.len() + 3;
        list_sql.push_str(&format!(
            " ORDER BY s.updated_at DESC, s.id DESC LIMIT ${limit_index} OFFSET ${offset_index}"
        ));

        let count_row = apply_binds(sqlx::query(AssertSqlSafe(&*count_sql)), &binds)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count deploy_server", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = apply_binds(sqlx::query(AssertSqlSafe(&*list_sql)), &binds)
            .bind(tenant_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list deploy_server", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_server_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_server row: {error}"))
            })?);
        }

        Ok(ServerPage { items, total })
    }

    pub(super) async fn create_server_repo(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        let cluster_internal_id = match request.cluster_id.as_deref() {
            Some(cluster_uuid) => {
                Some(resolve_node_cluster_internal_id(&self.pool, tenant_id, cluster_uuid).await?)
            }
            None => None,
        };

        sqlx::query(
            "INSERT INTO deploy_server (
                id, uuid, tenant_id, name, host, ssh_port, cluster_id, node_role, status,
                ssh_user, ssh_key_path, description, metadata, created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 0, 0, $8, $9, $10, '{}', CAST($11 AS TIMESTAMPTZ), CAST($11 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(&request.name)
        .bind(&request.host)
        .bind(request.ssh_port)
        .bind(cluster_internal_id)
        .bind(request.ssh_user.as_deref())
        .bind(request.ssh_key_path.as_deref())
        .bind(request.description.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_server", error))?;

        self.fetch_server_by_uuid(tenant_id, &uuid).await
    }

    pub(super) async fn update_server_repo(
        &self,
        tenant_id: i64,
        server_id: &str,
        request: &UpdateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        let row = sqlx::query("SELECT id FROM deploy_server WHERE tenant_id = $1 AND uuid = $2")
            .bind(tenant_id)
            .bind(server_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("resolve deploy_server id", error))?;
        let internal_id: i64 = row
            .and_then(|row| row.try_get::<i64, _>("id").ok())
            .ok_or_else(|| DeployServiceError::not_found("server not found"))?;

        let now = now_rfc3339();
        let mut sets: Vec<String> = Vec::new();
        let mut binds: Vec<BindValue> = Vec::new();
        if let Some(name) = request.name.as_deref() {
            binds.push(BindValue::Str(name));
            sets.push(format!("name = ${}", binds.len()));
        }
        if let Some(ssh_port) = request.ssh_port {
            binds.push(BindValue::I32(ssh_port));
            sets.push(format!("ssh_port = ${}", binds.len()));
        }
        if let Some(cluster_uuid) = request.cluster_id.as_deref() {
            let cluster_internal_id =
                resolve_node_cluster_internal_id(&self.pool, tenant_id, cluster_uuid).await?;
            binds.push(BindValue::I64(cluster_internal_id));
            sets.push(format!("cluster_id = ${}", binds.len()));
        }
        if let Some(ssh_user) = request.ssh_user.as_deref() {
            binds.push(BindValue::Str(ssh_user));
            sets.push(format!("ssh_user = ${}", binds.len()));
        }
        if let Some(description) = request.description.as_deref() {
            binds.push(BindValue::Str(description));
            sets.push(format!("description = ${}", binds.len()));
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
                "UPDATE deploy_server SET {}, updated_at = CAST(${updated_index} AS TIMESTAMPTZ), version = version + 1
                 WHERE tenant_id = ${tenant_index} AND id = ${id_index}",
                sets.join(", ")
            );
            apply_binds(sqlx::query(AssertSqlSafe(&*sql)), &binds)
                .bind(&now)
                .bind(tenant_id)
                .bind(internal_id)
                .execute(&self.pool)
                .await
                .map_err(|error| store_error("update deploy_server", error))?;
        }

        self.fetch_server_by_internal_id(tenant_id, internal_id)
            .await
    }

    async fn fetch_server_by_uuid(
        &self,
        tenant_id: i64,
        uuid: &str,
    ) -> DeployServiceResult<ServerResponse> {
        let row = sqlx::query(
            "SELECT s.uuid, s.name, s.host, s.ssh_port, s.cluster_id,
                    c.uuid AS cluster_uuid, c.name AS cluster_name,
                    s.node_role, s.status, s.ssh_user, s.description, s.created_at
             FROM deploy_server s
             LEFT JOIN deploy_node_cluster c ON c.id = s.cluster_id AND c.tenant_id = s.tenant_id
             WHERE s.tenant_id = $1 AND s.uuid = $2",
        )
        .bind(tenant_id)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("fetch deploy_server", error))?;

        row.map(|row| map_server_row(&row))
            .transpose()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_server row: {error}"))
            })?
            .ok_or_else(|| DeployServiceError::not_found("server not found"))
    }

    async fn fetch_server_by_internal_id(
        &self,
        tenant_id: i64,
        internal_id: i64,
    ) -> DeployServiceResult<ServerResponse> {
        let row = sqlx::query(
            "SELECT s.uuid, s.name, s.host, s.ssh_port, s.cluster_id,
                    c.uuid AS cluster_uuid, c.name AS cluster_name,
                    s.node_role, s.status, s.ssh_user, s.description, s.created_at
             FROM deploy_server s
             LEFT JOIN deploy_node_cluster c ON c.id = s.cluster_id AND c.tenant_id = s.tenant_id
             WHERE s.tenant_id = $1 AND s.id = $2",
        )
        .bind(tenant_id)
        .bind(internal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("fetch deploy_server", error))?;

        row.map(|row| map_server_row(&row))
            .transpose()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_server row: {error}"))
            })?
            .ok_or_else(|| DeployServiceError::not_found("server not found"))
    }
}

fn map_server_row(row: &PgRow) -> Result<ServerResponse, sqlx::Error> {
    Ok(ServerResponse {
        id: row.try_get("uuid")?,
        name: row.try_get("name")?,
        host: row.try_get("host")?,
        ssh_port: row.try_get("ssh_port")?,
        cluster_id: row.try_get::<Option<String>, _>("cluster_uuid")?,
        cluster_name: row.try_get::<Option<String>, _>("cluster_name")?,
        node_role: row.try_get("node_role")?,
        status: row.try_get("status")?,
        ssh_user: row.try_get::<Option<String>, _>("ssh_user")?,
        description: row.try_get::<Option<String>, _>("description")?,
        created_at: datetime_from_row(row, "created_at")?,
    })
}

enum BindValue<'a> {
    I64(i64),
    I32(i32),
    Str(&'a str),
}

fn apply_binds<'q>(
    mut query: sqlx::query::Query<'q, Postgres, PgArguments>,
    binds: &[BindValue<'_>],
) -> sqlx::query::Query<'q, Postgres, PgArguments> {
    for value in binds {
        query = match value {
            BindValue::I64(value) => query.bind(*value),
            BindValue::I32(value) => query.bind(*value),
            BindValue::Str(value) => query.bind(*value),
        };
    }
    query
}
