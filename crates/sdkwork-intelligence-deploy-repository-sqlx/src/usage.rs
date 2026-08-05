//! Usage metering repository operations (TECH §4.6): append-only usage facts
//! with deduplication identity, and the entitlement projection read model.

use sdkwork_deploy_contract::{
    DeployServiceError, DeployServiceResult, UsageEventPage, UsageEventResponse,
};
use sdkwork_intelligence_deploy_service::repository::InsertUsageEventCommand;
use sqlx::{postgres::PgRow, Row};

use crate::support::{
    datetime_from_row, is_unique_violation, new_uuid, next_id, now_rfc3339, pagination, store_error,
};
use crate::DeployRepository;

impl DeployRepository {
    /// Records one usage fact. Idempotent on the tenant deduplication key;
    /// duplicate delivery returns the existing fact.
    pub(super) async fn insert_usage_event_repo(
        &self,
        command: &InsertUsageEventCommand,
    ) -> DeployServiceResult<UsageEventResponse> {
        if let Some(existing) = self
            .find_usage_event_by_dedup_key_repo(command.tenant_id, &command.deduplication_key)
            .await?
        {
            return Ok(existing);
        }
        let event_id = next_id(self.id_generator())?;
        let event_uuid = new_uuid();
        let now = now_rfc3339();
        let result = sqlx::query(
            "INSERT INTO deploy_usage_event
                (id, uuid, tenant_id, organization_id, site_id, period_start, dimension,
                 quantity, unit, source_target_uuid, source_window_id, deduplication_key,
                 attribution_json, observed_at, ingested_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, '{}', $13, $13, $13)
             ON CONFLICT (tenant_id, deduplication_key) DO NOTHING
             RETURNING uuid",
        )
        .bind(event_id)
        .bind(&event_uuid)
        .bind(command.tenant_id)
        .bind(command.organization_id)
        .bind(command.site_id)
        .bind(&command.period_start)
        .bind(&command.dimension)
        .bind(command.quantity)
        .bind(&command.unit)
        .bind(command.source_target_uuid.as_deref())
        .bind(command.source_window_id.as_deref())
        .bind(&command.deduplication_key)
        .bind(&now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                DeployServiceError::conflict("usage event deduplication conflict")
            } else {
                store_error("insert deploy_usage_event", error)
            }
        })?;

        let Some(row) = result else {
            return self
                .find_usage_event_by_dedup_key_repo(command.tenant_id, &command.deduplication_key)
                .await?
                .ok_or_else(|| {
                    DeployServiceError::Internal(
                        "usage event disappeared after concurrent insertion".into(),
                    )
                });
        };
        let inserted_uuid: String = row.try_get("uuid").map_err(|error| {
            DeployServiceError::Internal(format!("read usage event uuid: {error}"))
        })?;
        self.retrieve_usage_event_repo(command.tenant_id, &inserted_uuid)
            .await
    }

    pub(super) async fn find_usage_event_by_dedup_key_repo(
        &self,
        tenant_id: i64,
        deduplication_key: &str,
    ) -> DeployServiceResult<Option<UsageEventResponse>> {
        let row = sqlx::query(
            "SELECT u.uuid, u.tenant_id, s.uuid AS site_uuid, u.period_start, u.dimension,
                    u.quantity, u.unit, u.source_target_uuid, u.source_window_id,
                    u.deduplication_key, u.observed_at, u.created_at
             FROM deploy_usage_event u
             LEFT JOIN deploy_site s ON s.id = u.site_id
             WHERE u.tenant_id = $1 AND u.deduplication_key = $2
             ORDER BY u.created_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .bind(deduplication_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("find deploy_usage_event by dedup key", error))?;

        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(map_usage_event_row(&row)?))
    }

    pub(super) async fn retrieve_usage_event_repo(
        &self,
        tenant_id: i64,
        event_id: &str,
    ) -> DeployServiceResult<UsageEventResponse> {
        let row = sqlx::query(
            "SELECT u.uuid, u.tenant_id, s.uuid AS site_uuid, u.period_start, u.dimension,
                    u.quantity, u.unit, u.source_target_uuid, u.source_window_id,
                    u.deduplication_key, u.observed_at, u.created_at
             FROM deploy_usage_event u
             LEFT JOIN deploy_site s ON s.id = u.site_id
             WHERE u.tenant_id = $1 AND u.uuid = $2",
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_usage_event", error))?;

        let Some(row) = row else {
            return Err(DeployServiceError::not_found("usage event not found"));
        };
        map_usage_event_row(&row)
    }

    pub(super) async fn list_usage_events_repo(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<UsageEventPage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let count_row =
            sqlx::query("SELECT COUNT(*) AS total FROM deploy_usage_event WHERE tenant_id = $1")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|error| store_error("count deploy_usage_event", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let rows = sqlx::query(
            "SELECT u.uuid, u.tenant_id, s.uuid AS site_uuid, u.period_start, u.dimension,
                    u.quantity, u.unit, u.source_target_uuid, u.source_window_id,
                    u.deduplication_key, u.observed_at, u.created_at
             FROM deploy_usage_event u
             LEFT JOIN deploy_site s ON s.id = u.site_id
             WHERE u.tenant_id = $1
             ORDER BY u.period_start DESC, u.id DESC LIMIT $2 OFFSET $3",
        )
        .bind(tenant_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_usage_event", error))?;

        let items = rows
            .iter()
            .map(map_usage_event_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UsageEventPage {
            items,
            total,
            page,
            page_size,
        })
    }
}

fn map_usage_event_row(row: &PgRow) -> Result<UsageEventResponse, DeployServiceError> {
    let period_start = datetime_from_row(row, "period_start")
        .map_err(|error| DeployServiceError::Internal(format!("read period_start: {error}")))?;
    let observed_at = datetime_from_row(row, "observed_at")
        .map_err(|error| DeployServiceError::Internal(format!("read observed_at: {error}")))?;
    let created_at = datetime_from_row(row, "created_at")
        .map_err(|error| DeployServiceError::Internal(format!("read created_at: {error}")))?;
    Ok(UsageEventResponse {
        id: row.try_get("uuid").unwrap_or_default(),
        tenant_id: row.try_get("tenant_id").unwrap_or(0),
        site_id: row
            .try_get::<Option<String>, _>("site_uuid")
            .ok()
            .flatten()
            .filter(|value| !value.is_empty()),
        period_start,
        dimension: row.try_get("dimension").unwrap_or_default(),
        quantity: row.try_get("quantity").unwrap_or(0),
        unit: row.try_get("unit").unwrap_or_default(),
        source_target_uuid: row.try_get("source_target_uuid").ok(),
        source_window_id: row.try_get("source_window_id").ok(),
        deduplication_key: row.try_get("deduplication_key").unwrap_or_default(),
        observed_at,
        created_at,
    })
}
