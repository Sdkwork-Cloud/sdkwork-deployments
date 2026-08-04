//! Platform review observation boundary for mini-program and store
//! deployments (REQ-2026-0002, deployment kinds `MINIPROGRAM_REVIEW` and
//! `STORE_SUBMISSION`).
//!
//! Review decisions belong to the external platform (WeChat/Douyin/App
//! Store); Deploy tracks them as observations. This module defines the typed
//! observer boundary and a no-op default. Real platform adapters (WeChat CI
//! upload status, TestFlight build processing, AppGallery review) are
//! enabled when the corresponding credentials and environments are
//! integrated; until then the state machine is driven through the control
//! plane's state update operation.

use sdkwork_deploy_contract::{AppDeploymentResponse, DeployServiceResult, DeploymentStatus};

/// One review observation reported by a platform adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewObservation {
    pub deployment_id: String,
    pub app_id: String,
    pub status: DeploymentStatus,
    pub platform_review_ref: Option<String>,
    pub detail: String,
}

/// Observer boundary: polls or receives platform review state and reports
/// typed observations to the control plane.
#[async_trait::async_trait]
pub trait ReviewObserver: Send + Sync {
    /// Observes the current review state for one deployment.
    async fn observe(
        &self,
        deployment: &AppDeploymentResponse,
    ) -> DeployServiceResult<ReviewObservation>;
}

/// Default observer: reports the deployment's current state unchanged.
/// Real platform adapters replace this once credentials are integrated.
pub struct NoOpReviewObserver;

#[async_trait::async_trait]
impl ReviewObserver for NoOpReviewObserver {
    async fn observe(
        &self,
        deployment: &AppDeploymentResponse,
    ) -> DeployServiceResult<ReviewObservation> {
        let status = serde_json::from_str::<DeploymentStatus>(&format!(
            "\"{}\"",
            deployment.deployment_status
        ))
        .unwrap_or(DeploymentStatus::PendingReview);
        Ok(ReviewObservation {
            deployment_id: deployment.id.clone(),
            app_id: deployment.app_id.clone(),
            status,
            platform_review_ref: deployment.platform_review_ref.clone(),
            detail: "no platform adapter configured; state unchanged".to_owned(),
        })
    }
}

/// Validates a review state transition for `MINIPROGRAM_REVIEW` and
/// `STORE_SUBMISSION` deployments. Review states are observed, never
/// inferred; terminal states are final.
pub fn validate_review_transition(current: &str, next: &str) -> Result<(), String> {
    let terminal = matches!(next, "LIVE" | "REJECTED" | "FAILED" | "CANCELLED");
    let allowed = if terminal {
        matches!(
            current,
            "SUBMITTING" | "PENDING_REVIEW" | "IN_REVIEW" | "APPROVED"
        )
    } else {
        matches!(
            (current, next),
            ("SUBMITTING", "PENDING_REVIEW")
                | ("PENDING_REVIEW", "IN_REVIEW")
                | ("IN_REVIEW", "APPROVED")
        )
    };
    if allowed {
        Ok(())
    } else {
        Err(format!(
            "invalid review state transition {current} -> {next}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_state_machine_is_forward_only() {
        assert!(validate_review_transition("SUBMITTING", "PENDING_REVIEW").is_ok());
        assert!(validate_review_transition("PENDING_REVIEW", "IN_REVIEW").is_ok());
        assert!(validate_review_transition("IN_REVIEW", "APPROVED").is_ok());
        assert!(validate_review_transition("APPROVED", "LIVE").is_ok());
        assert!(validate_review_transition("IN_REVIEW", "REJECTED").is_ok());
        assert!(validate_review_transition("LIVE", "IN_REVIEW").is_err());
        assert!(validate_review_transition("SUBMITTING", "APPROVED").is_err());
        assert!(validate_review_transition("PENDING_REVIEW", "SUBMITTING").is_err());
    }

    #[test]
    fn no_op_observer_reports_unchanged_state() {
        let deployment = AppDeploymentResponse {
            id: "deployment-1".to_owned(),
            app_id: "app-1".to_owned(),
            deployment_status: "PENDING_REVIEW".to_owned(),
            ..AppDeploymentResponse::default()
        };
        let observation = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(NoOpReviewObserver.observe(&deployment))
            .expect("observe");
        assert_eq!(observation.status, DeploymentStatus::PendingReview);
        assert_eq!(
            observation.detail,
            "no platform adapter configured; state unchanged"
        );
    }
}
