//! Retention enforcement, usage daily reconciliation, and signing identity
//! health (PRD §5.8, TECH §8): bounded housekeeping over packages, releases,
//! and build logs, the rebuildable daily usage aggregate, and the signing
//! expiry management surface.

use sdkwork_deploy_contract::{
    DeployServiceError, DeployServiceResult, RetentionRunResponse, SigningIdentityHealthPage,
    SigningIdentityHealthResponse, UsageReconciliationResponse,
};
use sqlx::{AssertSqlSafe, Row};

use crate::support::{optional_datetime, pagination, store_error};
use crate::DeployRepository;

impl DeployRepository {
    /// Applies retention policies. `dry_run` reports the candidate counts
    /// without mutating; real runs retire unreferenced packages/releases past
    /// their retention windows and purge expired build log references.
    /// Retention windows come from the caller (platform configuration); zero
    /// or negative windows disable that policy dimension.
    pub(super) async fn run_retention_repo(
        &self,
        dry_run: bool,
        package_retention_days: i64,
        release_retention_days: i64,
        build_log_retention_days: i64,
    ) -> DeployServiceResult<RetentionRunResponse> {
        let mut packages_retired = 0_i64;
        let mut releases_retired = 0_i64;
        let mut build_logs_purged = 0_i64;

        if package_retention_days > 0 {
            // Packages past retention that no release references.
            let candidates = sqlx::query(
                "SELECT p.id FROM deploy_package p
                 WHERE p.deleted_at IS NULL
                   AND p.status IN ('READY', 'VALIDATED', 'SUPERSEDED')
                   AND p.updated_at < NOW() - ($1 * INTERVAL '1 day')
                   AND NOT EXISTS (
                       SELECT 1 FROM deploy_release r
                       WHERE r.package_id = p.id AND r.release_status IS NOT NULL
                   )",
            )
            .bind(package_retention_days)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("scan package retention candidates", error))?;
            packages_retired = candidates.len() as i64;
            if !dry_run && !candidates.is_empty() {
                for candidate in &candidates {
                    let id: i64 = candidate.try_get("id").unwrap_or(0);
                    sqlx::query(
                        "UPDATE deploy_package SET status = 'RETIRED', updated_at = NOW(),
                             version = version + 1
                         WHERE id = $1 AND status <> 'RETIRED'",
                    )
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|error| store_error("retire package", error))?;
                }
            }
        }

        if release_retention_days > 0 {
            // Releases past retention that no channel points at.
            let candidates = sqlx::query(
                "SELECT r.id FROM deploy_release r
                 WHERE r.deleted_at IS NULL
                   AND r.release_status IN ('ACTIVE', 'SUPERSEDED', 'DEPRECATED')
                   AND r.updated_at < NOW() - ($1 * INTERVAL '1 day')
                   AND NOT EXISTS (
                       SELECT 1 FROM deploy_release_channel c
                       WHERE c.current_release_id = r.id
                   )",
            )
            .bind(release_retention_days)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("scan release retention candidates", error))?;
            releases_retired = candidates.len() as i64;
            if !dry_run && !candidates.is_empty() {
                for candidate in &candidates {
                    let id: i64 = candidate.try_get("id").unwrap_or(0);
                    sqlx::query(
                        "UPDATE deploy_release SET release_status = 'RETIRED', updated_at = NOW()
                         WHERE id = $1 AND release_status <> 'RETIRED'",
                    )
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|error| store_error("retire release", error))?;
                }
            }
        }

        if build_log_retention_days > 0 {
            // Terminal builds past retention: drop the log reference; the
            // build row and audit trail are immutable and must survive.
            let candidates = sqlx::query(
                "SELECT id FROM deploy_build
                 WHERE deleted_at IS NULL
                   AND log_ref IS NOT NULL
                   AND build_status IN ('SUCCEEDED', 'FAILED', 'CANCELLED', 'TIMED_OUT')
                   AND updated_at < NOW() - ($1 * INTERVAL '1 day')",
            )
            .bind(build_log_retention_days)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("scan build log retention candidates", error))?;
            build_logs_purged = candidates.len() as i64;
            if !dry_run && !candidates.is_empty() {
                for candidate in &candidates {
                    let id: i64 = candidate.try_get("id").unwrap_or(0);
                    sqlx::query(
                        "UPDATE deploy_build SET log_ref = NULL, updated_at = NOW(),
                             version = version + 1
                         WHERE id = $1 AND log_ref IS NOT NULL",
                    )
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|error| store_error("purge build log reference", error))?;
                }
            }
        }

        Ok(RetentionRunResponse {
            dry_run,
            packages_retired,
            releases_retired,
            build_logs_purged,
            package_retention_days,
            release_retention_days,
            build_log_retention_days,
        })
    }

    /// Rebuilds the reconcilable daily usage aggregate from retained usage
    /// facts (design contract: `deploy_app_usage_daily` is rebuildable).
    /// Idempotent: the unique (tenant, app, date, dimension, unit) scope is
    /// upserted, never duplicated. `window_start`/`window_end` bound the
    /// rebuild; `None` rebuilds the trailing 90 days.
    pub(super) async fn rebuild_usage_daily_repo(
        &self,
        window_start: Option<&str>,
        window_end: Option<&str>,
    ) -> DeployServiceResult<UsageReconciliationResponse> {
        let default_start = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(90))
            .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
            .unwrap_or_else(|| "2026-01-01T00:00:00.000Z".to_owned());
        let window_start = window_start.unwrap_or(&default_start).to_owned();
        let window_end = window_end.map(|value| value.to_owned()).unwrap_or_else(|| {
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        });
        // Validate timestamps before binding to the interval.
        if chrono::DateTime::parse_from_rfc3339(&window_start).is_err()
            || chrono::DateTime::parse_from_rfc3339(&window_end).is_err()
        {
            return Err(DeployServiceError::validation(
                "windowStart/windowEnd must be RFC3339 timestamps",
            ));
        }
        if window_start >= window_end {
            return Err(DeployServiceError::validation(
                "windowStart must be before windowEnd",
            ));
        }
        let result = sqlx::query(
            "INSERT INTO deploy_app_usage_daily
                (id, uuid, tenant_id, organization_id, app_id, binding_id, usage_date,
                 dimension, quantity, unit, source_revision, finalization_status,
                 created_at, updated_at)
             SELECT sdkwork_next_bigint(), gen_random_uuid(), u.tenant_id,
                    MAX(u.organization_id), u.app_id, u.binding_id,
                    (u.period_start AT TIME ZONE 'UTC')::date, u.dimension,
                    SUM(u.quantity), u.unit, 'rebuild:' || to_char(MAX(u.ingested_at), 'YYYYMMDDHH24MISS'),
                    'PENDING', NOW(), NOW()
             FROM deploy_usage_event u
             WHERE u.period_start >= $1 AND u.period_start < $2
             GROUP BY u.tenant_id, u.app_id, u.binding_id,
                      (u.period_start AT TIME ZONE 'UTC')::date, u.dimension, u.unit
             ON CONFLICT (tenant_id, app_id, COALESCE(binding_id, 0), usage_date, dimension, unit)
             DO UPDATE SET quantity = EXCLUDED.quantity, source_revision = EXCLUDED.source_revision,
                           updated_at = NOW()",
        )
        .bind(&window_start)
        .bind(&window_end)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("rebuild deploy_app_usage_daily", error))?;
        let site_rows = result.rows_affected() as i64;
        // Tenant-level daily rollup (SaaS billing): one row per tenant,
        // dimension and day, including unmanaged traffic (tenant 0).
        let tenant_result = sqlx::query(
            "INSERT INTO deploy_tenant_usage_daily
                (id, uuid, tenant_id, organization_id, usage_date, dimension,
                 quantity, unit, source_revision, finalization_status, created_at, updated_at)
             SELECT sdkwork_next_bigint(), gen_random_uuid(), u.tenant_id,
                    MAX(u.organization_id),
                    (u.period_start AT TIME ZONE 'UTC')::date, u.dimension,
                    SUM(u.quantity), u.unit,
                    'rebuild:' || to_char(MAX(u.ingested_at), 'YYYYMMDDHH24MISS'),
                    'PENDING', NOW(), NOW()
             FROM deploy_usage_event u
             WHERE u.period_start >= $1 AND u.period_start < $2
             GROUP BY u.tenant_id, (u.period_start AT TIME ZONE 'UTC')::date,
                      u.dimension, u.unit
             ON CONFLICT (tenant_id, dimension, usage_date, unit)
             DO UPDATE SET quantity = EXCLUDED.quantity, source_revision = EXCLUDED.source_revision,
                           updated_at = NOW()",
        )
        .bind(&window_start)
        .bind(&window_end)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("rebuild deploy_tenant_usage_daily", error))?;
        Ok(UsageReconciliationResponse {
            rebuilt_rows: site_rows + tenant_result.rows_affected() as i64,
            window_start,
            window_end,
        })
    }

    /// Backend signing identity health surface: expiry observations sorted by
    /// urgency, tenant-scoped when a tenant is provided.
    pub(super) async fn list_signing_identity_health_repo(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SigningIdentityHealthPage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let (filter, bind) = match tenant_id {
            Some(_tenant_id) => ("WHERE tenant_id = $1 AND deleted_at IS NULL", true),
            None => ("WHERE deleted_at IS NULL", false),
        };
        let count_query = format!("SELECT COUNT(*) AS total FROM deploy_signing_identity {filter}");
        let mut count = sqlx::query(AssertSqlSafe(&*count_query));
        if bind {
            count = count.bind(tenant_id.unwrap_or(0));
        }
        let count_row = count
            .fetch_one(&self.pool)
            .await
            .map_err(|error| store_error("count signing identities", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let list_query = format!(
            "SELECT uuid, tenant_id, identity_name, signing_kind, expires_at, identity_status
             FROM deploy_signing_identity {filter}
             ORDER BY expires_at ASC NULLS LAST, id DESC LIMIT $1 OFFSET $2"
        );
        let mut list = sqlx::query(AssertSqlSafe(&*list_query));
        if bind {
            list = list
                .bind(tenant_id.unwrap_or(0))
                .bind(page_size)
                .bind(offset);
        } else {
            list = list.bind(page_size).bind(offset);
        }
        let rows = list
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list signing identity health", error))?;

        let now = chrono::Utc::now();
        let items = rows
            .iter()
            .map(|row| {
                let expires_at = optional_datetime(row, "expires_at")?;
                let days_until_expiry = expires_at.as_deref().and_then(|value| {
                    chrono::DateTime::parse_from_rfc3339(value)
                        .ok()
                        .map(|expiry| (expiry.with_timezone(&chrono::Utc) - now).num_days())
                });
                Ok(SigningIdentityHealthResponse {
                    id: row.try_get("uuid").unwrap_or_default(),
                    tenant_id: row.try_get("tenant_id").unwrap_or(0),
                    identity_name: row.try_get("identity_name").unwrap_or_default(),
                    signing_kind: row.try_get("signing_kind").unwrap_or_default(),
                    expires_at,
                    days_until_expiry,
                    identity_status: row.try_get("identity_status").unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, DeployServiceError>>()?;
        Ok(SigningIdentityHealthPage {
            items,
            total,
            page,
            page_size,
        })
    }
}
