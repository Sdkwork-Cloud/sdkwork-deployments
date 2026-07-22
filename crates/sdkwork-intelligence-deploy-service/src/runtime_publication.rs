//! Durable runtime assignment publication orchestration.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, SecondsFormat, Utc};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceErrorKind, DeployServiceResult};
use sdkwork_deploy_runtime_compiler::{
    canonical_sha256_excluding_field, compile_runtime_set, normalize_runtime_descriptors,
    CompiledRuntimeSet, RuntimeEnvironment, RuntimeSetCompilationInput,
};
use sdkwork_deploy_web_port::{DeployWebRuntimePort, RuntimeAssignmentReceipt};
use serde_json::Value;

const DEFAULT_MAXIMUM_SITES: usize = 10_000;
const MAXIMUM_PUBLICATION_BATCH: i64 = 100;
const MAXIMUM_ATTEMPTS: i32 = 20;
const MINIMUM_LEASE_SECONDS: i64 = 5;
const MAXIMUM_LEASE_SECONDS: i64 = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTarget {
    pub target_uuid: String,
    pub tenant_id: i64,
    pub node_uuid: String,
    pub environment: RuntimeEnvironment,
    pub tenant_scope_hash: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeAssignmentState {
    pub assignment_uuid: String,
    pub target_uuid: String,
    pub tenant_id: i64,
    pub generation: u64,
    pub snapshot_uuid: String,
    pub snapshot_sha256: String,
    pub desired_state_sha256: String,
    pub runtime_set: Value,
    pub publish_status: RuntimeAssignmentPublishStatus,
    pub attempt_count: i32,
    pub lease_owner: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAssignmentPublishStatus {
    Pending,
    Publishing,
    Published,
    Failed,
    Superseded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePublicationBatchResult {
    pub claimed: usize,
    pub published: usize,
    pub failed: usize,
}

#[derive(Clone, Debug)]
pub struct PersistRuntimeAssignmentCommand {
    pub assignment_uuid: String,
    pub snapshot_uuid: String,
    pub snapshot_sha256: String,
    pub desired_state_sha256: String,
    pub runtime_set: Value,
    pub runtime_set_bytes: usize,
    pub created_at: String,
}

#[async_trait]
pub trait DeployRuntimeAssignmentMutationPort: Send {
    fn latest_runtime_assignment(&self) -> Option<&RuntimeAssignmentState>;

    fn next_generation(&self) -> u64;

    async fn commit_without_change(self: Box<Self>) -> DeployServiceResult<()>;

    async fn persist_runtime_assignment(
        self: Box<Self>,
        command: PersistRuntimeAssignmentCommand,
    ) -> DeployServiceResult<RuntimeAssignmentState>;
}

#[async_trait]
pub trait DeployRuntimeAssignmentRepositoryPort: Send + Sync {
    async fn latest_runtime_assignment(
        &self,
        target_uuid: &str,
    ) -> DeployServiceResult<Option<RuntimeAssignmentState>>;

    async fn begin_runtime_assignment_mutation(
        &self,
        target_uuid: &str,
        tenant_id: i64,
    ) -> DeployServiceResult<Box<dyn DeployRuntimeAssignmentMutationPort>>;

    async fn claim_due_runtime_assignments(
        &self,
        maximum_items: i64,
        now: &str,
        lease_owner: &str,
        lease_expires_at: &str,
        maximum_attempts: i32,
    ) -> DeployServiceResult<Vec<RuntimeAssignmentState>>;

    async fn mark_runtime_assignment_published(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        receipt: &RuntimeAssignmentReceipt,
        published_at: &str,
    ) -> DeployServiceResult<()>;

    async fn mark_runtime_assignment_failed(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        error_code: &str,
        next_attempt_at: Option<&str>,
        updated_at: &str,
    ) -> DeployServiceResult<()>;
}

pub struct RuntimePublicationService {
    repository: Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
    web_runtime: Arc<dyn DeployWebRuntimePort>,
    maximum_sites: usize,
}

impl RuntimePublicationService {
    pub fn new(
        repository: Arc<dyn DeployRuntimeAssignmentRepositoryPort>,
        web_runtime: Arc<dyn DeployWebRuntimePort>,
    ) -> Self {
        Self {
            repository,
            web_runtime,
            maximum_sites: DEFAULT_MAXIMUM_SITES,
        }
    }

    pub fn with_maximum_sites(mut self, maximum_sites: usize) -> DeployServiceResult<Self> {
        if maximum_sites == 0 || maximum_sites > DEFAULT_MAXIMUM_SITES {
            return Err(DeployServiceError::validation(
                "maximumSites is outside the supported range",
            ));
        }
        self.maximum_sites = maximum_sites;
        Ok(self)
    }

    pub async fn enqueue_target_state(
        &self,
        target: &RuntimeTarget,
        snapshot_uuid: String,
        generated_at: String,
        mut descriptors: Vec<Value>,
    ) -> DeployServiceResult<RuntimeAssignmentState> {
        validate_target_scope(target, &descriptors)?;
        normalize_runtime_descriptors(&mut descriptors);
        let desired_state_sha256 = desired_state_sha256(&descriptors)?;
        let mutation = self
            .repository
            .begin_runtime_assignment_mutation(&target.target_uuid, target.tenant_id)
            .await?;
        if let Some(latest) = mutation
            .latest_runtime_assignment()
            .filter(|assignment| {
                assignment.desired_state_sha256 == desired_state_sha256
                    && assignment.publish_status != RuntimeAssignmentPublishStatus::Superseded
            })
            .cloned()
        {
            mutation.commit_without_change().await?;
            return Ok(latest);
        }
        let generation = mutation.next_generation();
        if generation > 9_007_199_254_740_991 {
            return Err(DeployServiceError::conflict(
                "runtime assignment generation is exhausted",
            ));
        }
        let compiled = compile_runtime_set(RuntimeSetCompilationInput {
            snapshot_uuid: snapshot_uuid.clone(),
            node_uuid: target.node_uuid.clone(),
            environment: target.environment,
            generation,
            generated_at: generated_at.clone(),
            maximum_sites: self.maximum_sites,
            descriptors,
        })
        .map_err(|error| DeployServiceError::validation(error.to_string()))?;
        let runtime_set_bytes = serde_json::to_vec(&compiled.snapshot)
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?
            .len();
        mutation
            .persist_runtime_assignment(PersistRuntimeAssignmentCommand {
                assignment_uuid: sdkwork_database_id::uuid_v4(),
                snapshot_uuid,
                snapshot_sha256: compiled.snapshot_sha256,
                desired_state_sha256,
                runtime_set: compiled.snapshot,
                runtime_set_bytes,
                created_at: generated_at,
            })
            .await
    }

    pub async fn publish_assignment(
        &self,
        assignment: &RuntimeAssignmentState,
    ) -> DeployServiceResult<()> {
        if assignment.publish_status != RuntimeAssignmentPublishStatus::Publishing {
            return Err(DeployServiceError::conflict(
                "runtime assignment must hold a publication lease",
            ));
        }
        let lease_owner = assignment.lease_owner.as_deref().ok_or_else(|| {
            DeployServiceError::conflict("runtime assignment publication lease owner is missing")
        })?;
        let compiled = CompiledRuntimeSet {
            snapshot: assignment.runtime_set.clone(),
            snapshot_sha256: assignment.snapshot_sha256.clone(),
        };
        match self.web_runtime.publish_runtime_assignment(&compiled).await {
            Ok(receipt) => {
                if let Err(error) = validate_receipt(assignment, &receipt) {
                    let next_attempt = next_attempt_at(assignment.attempt_count);
                    let now = now_seconds();
                    self.repository
                        .mark_runtime_assignment_failed(
                            &assignment.assignment_uuid,
                            lease_owner,
                            "WEB_ASSIGNMENT_RECEIPT_MISMATCH",
                            next_attempt.as_deref(),
                            &now,
                        )
                        .await?;
                    return Err(error);
                }
                let now = now_seconds();
                self.repository
                    .mark_runtime_assignment_published(
                        &assignment.assignment_uuid,
                        lease_owner,
                        &receipt,
                        &now,
                    )
                    .await
            }
            Err(error) => {
                let next_attempt = next_attempt_at(assignment.attempt_count);
                let error_code = publication_error_code(&error);
                let now = now_seconds();
                self.repository
                    .mark_runtime_assignment_failed(
                        &assignment.assignment_uuid,
                        lease_owner,
                        error_code,
                        next_attempt.as_deref(),
                        &now,
                    )
                    .await?;
                Err(error)
            }
        }
    }

    pub async fn publish_due(
        &self,
        worker_id: &str,
        maximum_items: i64,
        lease_seconds: i64,
    ) -> DeployServiceResult<RuntimePublicationBatchResult> {
        if !(1..=MAXIMUM_PUBLICATION_BATCH).contains(&maximum_items) {
            return Err(DeployServiceError::validation(
                "runtime publication batch must be between 1 and 100",
            ));
        }
        validate_worker_id(worker_id)?;
        if !(MINIMUM_LEASE_SECONDS..=MAXIMUM_LEASE_SECONDS).contains(&lease_seconds) {
            return Err(DeployServiceError::validation(
                "runtime publication lease must be between 5 and 300 seconds",
            ));
        }
        let now = Utc::now();
        let now_text = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        let lease_expires_at =
            (now + Duration::seconds(lease_seconds)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let assignments = self
            .repository
            .claim_due_runtime_assignments(
                maximum_items,
                &now_text,
                worker_id,
                &lease_expires_at,
                MAXIMUM_ATTEMPTS,
            )
            .await?;
        let mut result = RuntimePublicationBatchResult {
            claimed: assignments.len(),
            ..RuntimePublicationBatchResult::default()
        };
        for assignment in assignments {
            if self.publish_assignment(&assignment).await.is_ok() {
                result.published += 1;
            } else {
                result.failed += 1;
            }
        }
        Ok(result)
    }
}

fn validate_worker_id(worker_id: &str) -> DeployServiceResult<()> {
    if worker_id.is_empty()
        || worker_id.len() > 128
        || !worker_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(DeployServiceError::validation(
            "runtime publication workerId is invalid",
        ));
    }
    Ok(())
}

fn validate_target_scope(target: &RuntimeTarget, descriptors: &[Value]) -> DeployServiceResult<()> {
    if target.tenant_id <= 0 {
        return Err(DeployServiceError::Forbidden);
    }
    if target.tenant_scope_hash.len() != 64
        || !target
            .tenant_scope_hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(DeployServiceError::validation(
            "runtime target tenant scope hash is invalid",
        ));
    }
    for descriptor in descriptors {
        let scope = descriptor
            .get("tenantScopeHash")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                DeployServiceError::validation("descriptor tenantScopeHash is required")
            })?;
        if scope != target.tenant_scope_hash {
            return Err(DeployServiceError::Forbidden);
        }
    }
    Ok(())
}

fn desired_state_sha256(descriptors: &[Value]) -> DeployServiceResult<String> {
    canonical_sha256_excluding_field(
        &serde_json::json!({"descriptors": descriptors}),
        "__no_excluded_field",
    )
    .map_err(|error| DeployServiceError::Internal(error.to_string()))
}

fn validate_receipt(
    assignment: &RuntimeAssignmentState,
    receipt: &RuntimeAssignmentReceipt,
) -> DeployServiceResult<()> {
    if receipt.snapshot_uuid != assignment.snapshot_uuid
        || receipt.snapshot_sha256 != assignment.snapshot_sha256
        || receipt.generation != assignment.generation.to_string()
    {
        return Err(DeployServiceError::conflict(
            "Web runtime assignment receipt does not match the durable assignment",
        ));
    }
    Ok(())
}

fn publication_error_code(error: &DeployServiceError) -> &'static str {
    match error.kind() {
        DeployServiceErrorKind::NotFound => "WEB_TARGET_NOT_FOUND",
        DeployServiceErrorKind::Conflict => "WEB_ASSIGNMENT_CONFLICT",
        DeployServiceErrorKind::Validation => "WEB_ASSIGNMENT_REJECTED",
        DeployServiceErrorKind::Forbidden => "WEB_ASSIGNMENT_FORBIDDEN",
        DeployServiceErrorKind::DatabaseUnavailable => "WEB_DATABASE_UNAVAILABLE",
        DeployServiceErrorKind::Internal => "WEB_PUBLICATION_UNAVAILABLE",
    }
}

fn next_attempt_at(attempt_count: i32) -> Option<String> {
    if attempt_count >= MAXIMUM_ATTEMPTS {
        return None;
    }
    let exponent = attempt_count.clamp(1, 10) as u32;
    let seconds = 1_i64 << exponent;
    Some((Utc::now() + Duration::seconds(seconds)).to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn now_seconds() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_scope_rejects_cross_tenant_descriptor() {
        let target = RuntimeTarget {
            target_uuid: "target-1".to_owned(),
            tenant_id: 1,
            node_uuid: "node-1".to_owned(),
            environment: RuntimeEnvironment::Production,
            tenant_scope_hash: "1".repeat(64),
        };
        let descriptor = serde_json::json!({"tenantScopeHash": "2".repeat(64)});
        assert!(matches!(
            validate_target_scope(&target, &[descriptor]),
            Err(DeployServiceError::Forbidden)
        ));
    }

    #[test]
    fn retry_schedule_is_bounded() {
        assert!(next_attempt_at(1).is_some());
        assert!(next_attempt_at(MAXIMUM_ATTEMPTS).is_none());
    }
}
