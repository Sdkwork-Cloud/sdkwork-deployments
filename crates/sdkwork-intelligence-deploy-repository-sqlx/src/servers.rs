use sdkwork_deploy_contract::{
    CreateServerRequest, DeployServiceError, DeployServiceResult, ServerPage, ServerResponse,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{new_uuid, next_id, now_rfc3339, pagination, store_error};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_servers_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ServerPage> {
        let (_page, page_size, offset) = pagination(page, page_size);

        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM deploy_server WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count deploy_server", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT uuid, name, host, ssh_port, status, created_at
             FROM deploy_server
             WHERE tenant_id = $1
             ORDER BY updated_at DESC LIMIT $2 OFFSET $3",
        )
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

        sqlx::query(
            "INSERT INTO deploy_server (
                id, uuid, tenant_id, name, host, ssh_port, status, metadata,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 0, '{}', $7, $7, 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(&request.name)
        .bind(&request.host)
        .bind(request.ssh_port)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_server", error))?;

        Ok(ServerResponse {
            id: uuid,
            name: request.name.clone(),
            host: request.host.clone(),
            ssh_port: request.ssh_port,
            status: 0,
            created_at: now,
        })
    }
}

fn map_server_row(row: &AnyRow) -> Result<ServerResponse, sqlx::Error> {
    Ok(ServerResponse {
        id: row.try_get("uuid")?,
        name: row.try_get("name")?,
        host: row.try_get("host")?,
        ssh_port: row.try_get("ssh_port")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
