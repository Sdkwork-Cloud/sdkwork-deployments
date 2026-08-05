//! Entitlement consumption enforcement (TECH §4.6): tenant capacity is
//! gated by the Commerce-backed `deploy_tenant_entitlement_projection` read
//! model. Failures are fail-closed for new capacity; already-active delivery
//! is never suspended by enforcement.

use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_deploy_core::deploy_entitlement_enforcement_enabled;

use crate::DeployService;

impl DeployService {
    /// Enforces one entitlement dimension for the tenant before a capacity
    /// creation operation commits. No-op while enforcement is disabled.
    ///
    /// When enabled:
    /// - no active projection → fail closed (new capacity requires a plan);
    /// - expired projection → fail closed;
    /// - a dimension limit is present and usage is at/over it → quota
    ///   exceeded (429).
    pub async fn enforce_entitlement(
        &self,
        tenant_id: i64,
        dimension: &str,
    ) -> DeployServiceResult<()> {
        if !deploy_entitlement_enforcement_enabled() {
            return Ok(());
        }
        let page = self
            .repository
            .list_entitlement_projections(Some(tenant_id), 1, 1)
            .await?;
        let Some(projection) = page.items.into_iter().next() else {
            return Err(DeployServiceError::forbidden(format!(
                "tenant {tenant_id} has no active entitlement plan; new capacity is locked"
            )));
        };
        if projection.projection_status != "ACTIVE" {
            return Err(DeployServiceError::forbidden(format!(
                "entitlement projection is {status}; new capacity is locked",
                status = projection.projection_status
            )));
        }
        if let Some(expires_at) = projection.expires_at.as_deref() {
            if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expires_at.with_timezone(&chrono::Utc) < chrono::Utc::now() {
                    return Err(DeployServiceError::forbidden(format!(
                        "entitlement projection expired at {expires_at}; new capacity is locked"
                    )));
                }
            }
        }
        let Some(limit) = projection
            .entitlements
            .get(dimension)
            .and_then(serde_json::Value::as_i64)
        else {
            // The plan does not constrain this dimension.
            return Ok(());
        };
        let usage = self
            .repository
            .entitlement_usage(tenant_id, dimension)
            .await?;
        if usage >= limit {
            return Err(DeployServiceError::quota_exceeded(format!(
                "{dimension} plan limit {limit} reached (usage {usage})"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_limit_is_read_from_projection() {
        // The enforcement contract reads snake_case dimension keys matching
        // the ENTITLEMENT_DIMENSION_* constants; absent dimensions mean the
        // plan does not constrain that capacity.
        let entitlements = serde_json::json!({
            "active_apps": 5,
            "package_storage_bytes": 1073741824,
        });
        assert_eq!(
            entitlements
                .get("active_apps")
                .and_then(serde_json::Value::as_i64),
            Some(5)
        );
        assert_eq!(
            entitlements
                .get("package_storage_bytes")
                .and_then(serde_json::Value::as_i64),
            Some(1073741824)
        );
        assert_eq!(
            entitlements
                .get("build_concurrency")
                .and_then(serde_json::Value::as_i64),
            None
        );
        // Non-numeric limits are not treated as constraints.
        assert_eq!(
            serde_json::json!({"active_apps": "unlimited"})
                .get("active_apps")
                .and_then(serde_json::Value::as_i64),
            None
        );
    }
}
