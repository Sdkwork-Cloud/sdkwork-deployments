use sdkwork_deploy_contract::{
    CreateHealthCheckRequest, DeployServiceError, DeployServiceResult, HealthCheckPage,
    HealthCheckResponse,
};
use sqlx::{postgres::PgRow, Row};

use crate::support::{new_uuid, next_id, now_rfc3339, resolve_site_internal_id, store_error};
use crate::DeployRepository;

/// 单个站点的健康检查集合上限；列表内存与响应体积保持 O(上限)。
const MAX_SITE_HEALTH_CHECKS: i64 = 100;

impl DeployRepository {
    pub(super) async fn list_health_checks_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;

        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_health_check
             WHERE tenant_id = $1 AND site_id = $2",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_health_check", error))?;
        let total: i64 = count_row
            .try_get("total")
            .map_err(|error| store_error("map deploy_health_check count", error))?;
        if total > MAX_SITE_HEALTH_CHECKS {
            tracing::error!(
                tenant_id,
                site_id,
                total,
                maximum = MAX_SITE_HEALTH_CHECKS,
                "deploy health-check cardinality invariant violated"
            );
            return Err(DeployServiceError::Internal(
                "health-check collection exceeds its configured capacity".to_string(),
            ));
        }

        let rows = sqlx::query(
            "SELECT uuid, check_type, check_url, status
             FROM deploy_health_check
             WHERE tenant_id = $1 AND site_id = $2
             ORDER BY created_at DESC, id DESC
             LIMIT 100",
        )
        .bind(tenant_id)
        .bind(site_internal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_health_check", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_health_check_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_health_check row: {error}"))
            })?);
        }

        Ok(HealthCheckPage { items, total })
    }

    pub(super) async fn create_health_check_repo(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse> {
        let site_internal_id = resolve_site_internal_id(&self.pool, tenant_id, site_id).await?;
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_health_check (
                id, uuid, tenant_id, site_id, check_type, check_url, status,
                created_at, updated_at, version
             ) VALUES (
                $1, $2, $3, $4, $5, $6, 1, CAST($7 AS TIMESTAMPTZ), CAST($7 AS TIMESTAMPTZ), 0
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(site_internal_id)
        .bind(request.check_type)
        .bind(&request.url)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_health_check", error))?;

        Ok(HealthCheckResponse {
            id: uuid,
            check_type: request.check_type,
            url: request.url.clone(),
            status: 1,
        })
    }
}

fn map_health_check_row(row: &PgRow) -> Result<HealthCheckResponse, sqlx::Error> {
    Ok(HealthCheckResponse {
        id: row.try_get("uuid")?,
        check_type: row.try_get("check_type")?,
        url: row.try_get("check_url")?,
        status: row.try_get("status")?,
    })
}
