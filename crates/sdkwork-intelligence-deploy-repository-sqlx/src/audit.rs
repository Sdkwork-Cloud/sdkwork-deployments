use sdkwork_deploy_contract::{
    AuditLogPage, AuditLogQuery, AuditLogResponse, DeployServiceError, DeployServiceResult,
};
use sdkwork_intelligence_deploy_service::repository::InsertAuditLogCommand;
use sqlx::{postgres::PgRow, AssertSqlSafe, Row};

use crate::support::{datetime_from_row, new_uuid, next_id, now_rfc3339, pagination, store_error};
use crate::DeployRepository;

impl DeployRepository {
    pub(super) async fn list_audit_logs_repo(
        &self,
        tenant_id: Option<i64>,
        query: &AuditLogQuery,
    ) -> DeployServiceResult<AuditLogPage> {
        let (page, page_size, offset) = pagination(query.page, query.page_size);

        let Some(tenant_id) = tenant_id else {
            // 审计日志不允许无租户上下文的全库枚举：调用方必须先解析租户。
            return Err(DeployServiceError::forbidden(
                "tenant context is required for audit log listing",
            ));
        };

        // OpenAPI 声明的过滤器逐项落地（PAGINATION_SPEC §4：声明即实现）。
        // 所有动态值经 bind 参数注入，WHERE 片段仅由固定子句拼接。
        let mut conditions = vec!["tenant_id = $1".to_string()];
        let mut bind_index = 2;
        if query.target_type.as_deref().is_some_and(|v| !v.is_empty()) {
            conditions.push(format!("target_type = ${bind_index}"));
            bind_index += 1;
        }
        if query.action.as_deref().is_some_and(|v| !v.is_empty()) {
            conditions.push(format!("action = ${bind_index}"));
            bind_index += 1;
        }
        if query.operator_id.is_some() {
            conditions.push(format!("operator_id = ${bind_index}"));
            bind_index += 1;
        }
        if query.start_date.as_deref().is_some_and(|v| !v.is_empty()) {
            conditions.push(format!("created_at >= CAST(${bind_index} AS TIMESTAMPTZ)"));
            bind_index += 1;
        }
        if query.end_date.as_deref().is_some_and(|v| !v.is_empty()) {
            conditions.push(format!("created_at <= CAST(${bind_index} AS TIMESTAMPTZ)"));
            bind_index += 1;
        }
        let where_clause = conditions.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) AS total FROM deploy_audit_log WHERE {where_clause}");
        let mut count_query = sqlx::query(AssertSqlSafe(count_sql.as_str())).bind(tenant_id);
        if let Some(target_type) = query.target_type.as_deref().filter(|v| !v.is_empty()) {
            count_query = count_query.bind(target_type);
        }
        if let Some(action) = query.action.as_deref().filter(|v| !v.is_empty()) {
            count_query = count_query.bind(action);
        }
        if let Some(operator_id) = query.operator_id {
            count_query = count_query.bind(operator_id);
        }
        if let Some(start_date) = query.start_date.as_deref().filter(|v| !v.is_empty()) {
            count_query = count_query.bind(start_date);
        }
        if let Some(end_date) = query.end_date.as_deref().filter(|v| !v.is_empty()) {
            count_query = count_query.bind(end_date);
        }
        let count_row = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count deploy_audit_log", error))?;
        let total: i64 = count_row
            .try_get("total")
            .map_err(|error| store_error("map deploy_audit_log count", error))?;

        let limit_index = bind_index;
        let offset_index = bind_index + 1;
        let list_sql = format!(
            "SELECT uuid, action, target_type, created_at
             FROM deploy_audit_log
             WHERE {where_clause}
             ORDER BY created_at DESC, id DESC LIMIT ${limit_index} OFFSET ${offset_index}"
        );
        let mut list_query = sqlx::query(AssertSqlSafe(list_sql.as_str())).bind(tenant_id);
        if let Some(target_type) = query.target_type.as_deref().filter(|v| !v.is_empty()) {
            list_query = list_query.bind(target_type);
        }
        if let Some(action) = query.action.as_deref().filter(|v| !v.is_empty()) {
            list_query = list_query.bind(action);
        }
        if let Some(operator_id) = query.operator_id {
            list_query = list_query.bind(operator_id);
        }
        if let Some(start_date) = query.start_date.as_deref().filter(|v| !v.is_empty()) {
            list_query = list_query.bind(start_date);
        }
        if let Some(end_date) = query.end_date.as_deref().filter(|v| !v.is_empty()) {
            list_query = list_query.bind(end_date);
        }
        let rows = list_query
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list deploy_audit_log", error))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_audit_log_row(row).map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_audit_log row: {error}"))
            })?);
        }

        Ok(AuditLogPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn insert_audit_log_repo(
        &self,
        command: &InsertAuditLogCommand,
    ) -> DeployServiceResult<()> {
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        let now = now_rfc3339();

        sqlx::query(
            "INSERT INTO deploy_audit_log (
                id, uuid, tenant_id, organization_id, operator_id, action, target_type,
                target_id, target_uuid, metadata, created_at
             ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, '{}', CAST($10 AS TIMESTAMPTZ)
             )",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(command.organization_id)
        .bind(command.operator_id)
        .bind(&command.action)
        .bind(&command.target_type)
        .bind(command.target_id)
        .bind(command.target_uuid.as_deref())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_audit_log", error))?;

        Ok(())
    }
}

fn map_audit_log_row(row: &PgRow) -> Result<AuditLogResponse, sqlx::Error> {
    Ok(AuditLogResponse {
        id: row.try_get("uuid")?,
        action: row.try_get("action")?,
        resource: row.try_get("target_type")?,
        created_at: datetime_from_row(row, "created_at")?,
    })
}
