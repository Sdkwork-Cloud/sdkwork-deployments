//! App-scoped deployment repository operations (REQ-2026-0002 requirement
//! 11). Deployments execute a channel release against a typed target and
//! record platform review observations, rollback linkage, and audit.

use sdkwork_deploy_contract::{
    AppDeploymentPage, AppDeploymentResponse, CreateAppDeploymentRequest, DeployServiceError,
    DeployServiceResult, DeploymentStatus, RolloutStrategy,
};
use sqlx::{postgres::PgRow, AssertSqlSafe, Row};

use crate::support::{
    new_uuid, next_id, optional_datetime, pagination, required_datetime, resolve_app_internal_id,
    resolve_platform_target_internal_id, store_error,
};
use crate::DeployRepository;

const APP_DEPLOYMENT_SELECT: &str = "d.uuid, d.deployment_kind, d.deployment_target,
    d.environment, d.strategy, d.percentage, d.platform_review_ref, d.deployment_status,
    d.rollback_from_deployment_id, d.started_at, d.completed_at, d.created_at, d.updated_at,
    d.version";

fn map_app_deployment_row(row: &PgRow) -> Result<AppDeploymentResponse, DeployServiceError> {
    let created_at = required_datetime(row, "created_at")?;
    let updated_at = required_datetime(row, "updated_at")?;
    Ok(AppDeploymentResponse {
        id: row.try_get("uuid").unwrap_or_default(),
        app_id: row.try_get("app_uuid").unwrap_or_default(),
        platform_target_id: row.try_get("target_uuid").ok(),
        release_id: row.try_get("release_uuid").ok(),
        deployment_kind: row.try_get("deployment_kind").ok(),
        deployment_target: row.try_get("deployment_target").ok(),
        environment: row.try_get("environment").unwrap_or_default(),
        strategy: row.try_get("strategy").ok(),
        percentage: row
            .try_get::<Option<i32>, _>("percentage")
            .ok()
            .flatten()
            .map(|value| value as u32),
        platform_review_ref: row.try_get("platform_review_ref").ok(),
        deployment_status: row.try_get("deployment_status").unwrap_or_default(),
        rollback_from_deployment_id: row.try_get("rollback_from_uuid").ok(),
        started_at: optional_datetime(row, "started_at")?,
        completed_at: optional_datetime(row, "completed_at")?,
        created_at,
        updated_at,
        version: row.try_get::<i64, _>("version").unwrap_or(1).to_string(),
    })
}

impl DeployRepository {
    pub(super) async fn create_app_deployment_repo(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateAppDeploymentRequest,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        if let Some(existing) = self
            .find_app_deployment_by_idempotency_key_repo(tenant_id, &request.idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        // The release pins the (app, platform target) scope.
        let release_scope = sqlx::query(
            "SELECT r.app_id, r.platform_target_id, r.release_status FROM deploy_release r
             WHERE r.tenant_id = $1 AND r.uuid = $2 AND r.release_status IS NOT NULL",
        )
        .bind(tenant_id)
        .bind(&request.release_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve deploy_release app scope", error))?;

        let Some(release_scope) = release_scope else {
            return Err(DeployServiceError::not_found("release not found"));
        };
        let app_internal_id: i64 = release_scope.try_get("app_id").unwrap_or(0);
        let target_internal_id: i64 = release_scope.try_get("platform_target_id").unwrap_or(0);
        let release_status: String = release_scope.try_get("release_status").unwrap_or_default();
        if matches!(release_status.as_str(), "RETIRED" | "ARCHIVED") {
            return Err(DeployServiceError::validation(format!(
                "release status {release_status} cannot be deployed"
            )));
        }

        let requested_target = resolve_platform_target_internal_id(
            &self.pool,
            tenant_id,
            app_internal_id,
            &request.platform_target_id,
        )
        .await?;
        if requested_target != target_internal_id {
            return Err(DeployServiceError::validation(
                "platform target does not match the release platform target",
            ));
        }

        let strategy = request.strategy.unwrap_or(RolloutStrategy::Immediate);
        if let Some(percentage) = request.percentage {
            if !(1..=100).contains(&percentage) {
                return Err(DeployServiceError::validation("percentage must be 1..=100"));
            }
            if strategy != RolloutStrategy::Percentage {
                return Err(DeployServiceError::validation(
                    "percentage requires the PERCENTAGE strategy",
                ));
            }
        }

        let deployment_id = next_id(self.id_generator())?;
        let deployment_uuid = new_uuid();
        let environment = request
            .environment
            .clone()
            .unwrap_or_else(|| "production".to_owned());
        let initial_status = match request.deployment_kind.as_str() {
            "MINIPROGRAM_REVIEW" | "STORE_SUBMISSION" => DeploymentStatus::Submitting.as_str(),
            _ => DeploymentStatus::Pending.as_str(),
        };

        sqlx::query(
            "INSERT INTO deploy_deployment
                (id, uuid, tenant_id, organization_id, app_id, platform_target_id,
                 release_id, deployment_kind, deployment_target, environment, strategy,
                 percentage, platform_review_ref, deployment_status, idempotency_key,
                 started_at, created_by, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NULL, $13, $14,
                     NOW(), $15, NOW(), NOW(), 1)
             ON CONFLICT (tenant_id, idempotency_key) DO NOTHING
             RETURNING uuid",
        )
        .bind(deployment_id)
        .bind(&deployment_uuid)
        .bind(tenant_id)
        .bind(0)
        .bind(app_internal_id)
        .bind(target_internal_id)
        .bind(
            sqlx::query("SELECT id FROM deploy_release WHERE tenant_id = $1 AND uuid = $2")
                .bind(tenant_id)
                .bind(&request.release_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| store_error("resolve deploy_release id", error))?
                .and_then(|row| row.try_get::<i64, _>("id").ok())
                .ok_or_else(|| DeployServiceError::not_found("release not found"))?,
        )
        .bind(request.deployment_kind.as_str())
        .bind(request.deployment_target.as_str())
        .bind(&environment)
        .bind(strategy.as_str())
        .bind(request.percentage.map(|value| value as i32))
        .bind(initial_status)
        .bind(&request.idempotency_key)
        .bind(actor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_deployment", error))?;

        // The response resolves the app scope from the release.
        let app_uuid: String = sqlx::query(
            "SELECT a.uuid FROM deploy_release r
             JOIN deploy_app a ON a.id = r.app_id
             WHERE r.tenant_id = $1 AND r.uuid = $2",
        )
        .bind(tenant_id)
        .bind(&request.release_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve release app uuid", error))?
        .and_then(|row| row.try_get::<String, _>("uuid").ok())
        .ok_or_else(|| DeployServiceError::not_found("release not found"))?;
        self.retrieve_app_deployment_repo(tenant_id, &app_uuid, &deployment_uuid)
            .await
    }

    pub(super) async fn find_app_deployment_by_idempotency_key_repo(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<AppDeploymentResponse>> {
        let query = format!(
            "SELECT {APP_DEPLOYMENT_SELECT}, a.uuid AS app_uuid, t.uuid AS target_uuid,
                    r.uuid AS release_uuid, rd.uuid AS rollback_from_uuid
             FROM deploy_deployment d
             JOIN deploy_app a ON a.id = d.app_id
             LEFT JOIN deploy_app_platform_target t ON t.id = d.platform_target_id
             LEFT JOIN deploy_release r ON r.id = d.release_id
             LEFT JOIN deploy_deployment rd ON rd.id = d.rollback_from_deployment_id
             WHERE d.tenant_id = $1 AND d.idempotency_key = $2
             ORDER BY d.created_at DESC LIMIT 1"
        );
        let row = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(idempotency_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("find deploy_deployment by idempotency key", error))?;

        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(map_app_deployment_row(&row)?))
    }

    pub(super) async fn list_app_deployments_repo(
        &self,
        tenant_id: i64,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppDeploymentPage> {
        let app_internal_id = resolve_app_internal_id(&self.pool, tenant_id, app_id).await?;
        let (page, page_size, offset) = pagination(page, page_size);
        let count_row = sqlx::query(
            "SELECT COUNT(*) AS total FROM deploy_deployment
             WHERE tenant_id = $1 AND app_id = $2",
        )
        .bind(tenant_id)
        .bind(app_internal_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_deployment app", error))?;
        let total: i64 = count_row.try_get("total").unwrap_or(0);

        let query = format!(
            "SELECT {APP_DEPLOYMENT_SELECT}, a.uuid AS app_uuid, t.uuid AS target_uuid,
                    r.uuid AS release_uuid, rd.uuid AS rollback_from_uuid
             FROM deploy_deployment d
             JOIN deploy_app a ON a.id = d.app_id
             LEFT JOIN deploy_app_platform_target t ON t.id = d.platform_target_id
             LEFT JOIN deploy_release r ON r.id = d.release_id
             LEFT JOIN deploy_deployment rd ON rd.id = d.rollback_from_deployment_id
             WHERE d.tenant_id = $1 AND d.app_id = $2
             ORDER BY d.created_at DESC, d.id DESC LIMIT $3 OFFSET $4"
        );
        let rows = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(app_internal_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list deploy_deployment app", error))?;

        let items = rows
            .iter()
            .map(map_app_deployment_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AppDeploymentPage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn retrieve_app_deployment_repo(
        &self,
        tenant_id: i64,
        app_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        let app_internal_id = resolve_app_internal_id(&self.pool, tenant_id, app_id).await?;
        let query = format!(
            "SELECT {APP_DEPLOYMENT_SELECT}, a.uuid AS app_uuid, t.uuid AS target_uuid,
                    r.uuid AS release_uuid, rd.uuid AS rollback_from_uuid
             FROM deploy_deployment d
             JOIN deploy_app a ON a.id = d.app_id
             LEFT JOIN deploy_app_platform_target t ON t.id = d.platform_target_id
             LEFT JOIN deploy_release r ON r.id = d.release_id
             LEFT JOIN deploy_deployment rd ON rd.id = d.rollback_from_deployment_id
             WHERE d.tenant_id = $1 AND d.app_id = $2 AND d.uuid = $3"
        );
        let row = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(app_internal_id)
            .bind(deployment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| store_error("retrieve deploy_deployment app", error))?;

        let Some(row) = row else {
            return Err(DeployServiceError::not_found("deployment not found"));
        };
        map_app_deployment_row(&row)
    }

    /// Lists deployments currently in platform review states, newest first,
    /// for review-observation polling (bounded scan; review state is observed,
    /// never inferred).
    pub(super) async fn list_review_pending_deployments_repo(
        &self,
        tenant_id: i64,
        limit: i64,
    ) -> DeployServiceResult<Vec<AppDeploymentResponse>> {
        let bounded_limit = limit.clamp(1, 100);
        let query = format!(
            "SELECT {APP_DEPLOYMENT_SELECT}, a.uuid AS app_uuid, t.uuid AS target_uuid,
                    r.uuid AS release_uuid, rd.uuid AS rollback_from_uuid
             FROM deploy_deployment d
             JOIN deploy_app a ON a.id = d.app_id
             LEFT JOIN deploy_app_platform_target t ON t.id = d.platform_target_id
             LEFT JOIN deploy_release r ON r.id = d.release_id
             LEFT JOIN deploy_deployment rd ON rd.id = d.rollback_from_deployment_id
             WHERE d.tenant_id = $1
               AND d.deployment_status IN ('SUBMITTING', 'PENDING_REVIEW', 'IN_REVIEW')
             ORDER BY d.created_at ASC, d.id ASC LIMIT $2"
        );
        let rows = sqlx::query(AssertSqlSafe(&*query))
            .bind(tenant_id)
            .bind(bounded_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| store_error("list review pending deploy_deployment", error))?;
        let items = rows
            .iter()
            .map(map_app_deployment_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// Records a platform review observation (submission reference, review
    /// state) and completes the deployment on terminal states.
    pub(super) async fn update_app_deployment_state_repo(
        &self,
        tenant_id: i64,
        app_id: &str,
        deployment_id: &str,
        deployment_status: DeploymentStatus,
        platform_review_ref: Option<&str>,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        let app_internal_id = resolve_app_internal_id(&self.pool, tenant_id, app_id).await?;
        let terminal = matches!(
            deployment_status,
            DeploymentStatus::Live
                | DeploymentStatus::Active
                | DeploymentStatus::Failed
                | DeploymentStatus::RolledBack
                | DeploymentStatus::Rejected
                | DeploymentStatus::Cancelled
        );
        sqlx::query(
            "UPDATE deploy_deployment SET
                deployment_status = $3,
                platform_review_ref = COALESCE($4, platform_review_ref),
                completed_at = CASE WHEN $5 THEN NOW() ELSE completed_at END,
                updated_at = NOW(), version = version + 1
             WHERE tenant_id = $1 AND app_id = $2 AND uuid = $6 AND deployment_status NOT IN
                 ('LIVE', 'ACTIVE', 'FAILED', 'ROLLED_BACK', 'REJECTED', 'CANCELLED')
             RETURNING uuid",
        )
        .bind(tenant_id)
        .bind(app_internal_id)
        .bind(deployment_status.as_str())
        .bind(platform_review_ref)
        .bind(terminal)
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("update deploy_deployment state", error))?;

        self.retrieve_app_deployment_repo(tenant_id, app_id, deployment_id)
            .await
    }
}
