//! `DeployRepositoryPort` trait implementation delegating to SQLx repository modules.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    ArtifactPage, ArtifactResponse, AuditLogPage, CertificatePage, CertificateResponse,
    CreateCertificateRequest, CreateDeployUploadSessionRequest, CreateDeploymentRequest,
    CreateDomainRequest, CreateEnvVariableRequest, CreateHealthCheckRequest,
    CreateNginxConfigRequest, CreateReleaseRequest, CreateServerRequest, CreateSiteRequest,
    DeployAppRequestContext, DeployUploadSessionResponse, DeploymentPage, DeploymentResponse,
    DomainPage, DomainResponse, DomainVerifyResponse, EnvVariablePage, EnvVariableResponse,
    HealthCheckPage, HealthCheckResponse, ListNginxConfigsQuery, ListSitesQuery, NginxConfigPage,
    NginxConfigResponse, NginxReloadResponse, NginxStatusResponse, NginxValidateResponse,
    ReleasePage, ReleaseResponse, ServerPage, ServerResponse, SitePage, SiteResponse,
    UpdateNginxConfigRequest, UpdateSiteRequest, UploadCustomCertificateRequest,
};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_deploy_web_port::RuntimeAssignmentReceipt;
use sdkwork_intelligence_deploy_service::repository::InsertAuditLogCommand;
use sdkwork_intelligence_deploy_service::runtime_publication::{
    DeployRuntimeAssignmentMutationPort, DeployRuntimeAssignmentRepositoryPort,
    RuntimeAssignmentState, RuntimeObservationEvidence, RuntimeObservationPersistenceResult,
};
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;

use crate::DeployRepository;

#[async_trait]
impl DeployRepositoryPort for DeployRepository {
    async fn ready_check(&self) -> DeployServiceResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|_| DeployServiceError::DatabaseUnavailable)?;
        Ok(())
    }

    async fn list_sites(
        &self,
        tenant_id: i64,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage> {
        self.list_sites_repo(tenant_id, query).await
    }

    async fn create_site(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<SiteResponse> {
        self.create_site_repo(tenant_id, organization_id, actor_id, request)
            .await
    }

    async fn retrieve_site(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse> {
        self.retrieve_site_repo(tenant_id, site_id).await
    }

    async fn update_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse> {
        self.update_site_repo(tenant_id, site_id, request).await
    }

    async fn delete_site(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<()> {
        self.delete_site_repo(tenant_id, site_id, actor_id).await
    }

    async fn set_site_status(
        &self,
        tenant_id: i64,
        site_id: &str,
        status: i32,
    ) -> DeployServiceResult<SiteResponse> {
        self.set_site_status_repo(tenant_id, site_id, status).await
    }

    async fn list_domains(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainPage> {
        self.list_domains_repo(tenant_id, site_id, page, page_size)
            .await
    }

    async fn create_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateDomainRequest,
    ) -> DeployServiceResult<DomainResponse> {
        self.create_domain_repo(tenant_id, site_id, request).await
    }

    async fn retrieve_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainResponse> {
        self.retrieve_domain_repo(tenant_id, site_id, domain_id)
            .await
    }

    async fn delete_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<()> {
        self.delete_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn verify_domain(
        &self,
        tenant_id: i64,
        site_id: &str,
        domain_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse> {
        self.verify_domain_repo(tenant_id, site_id, domain_id).await
    }

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
    ) -> DeployServiceResult<DeploymentPage> {
        self.list_deployments_repo(tenant_id, site_id, page, page_size, status)
            .await
    }

    async fn create_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        actor_id: Option<i64>,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.create_deployment_repo(tenant_id, site_id, actor_id, request)
            .await
    }

    async fn retrieve_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.retrieve_deployment_repo(tenant_id, site_id, deployment_id)
            .await
    }

    async fn rollback_deployment(
        &self,
        tenant_id: i64,
        site_id: &str,
        deployment_id: &str,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<DeploymentResponse> {
        self.rollback_deployment_repo(tenant_id, site_id, deployment_id, actor_id)
            .await
    }

    async fn list_artifacts(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ArtifactPage> {
        self.list_artifacts_repo(tenant_id, page, page_size).await
    }

    async fn retrieve_artifact(
        &self,
        tenant_id: i64,
        artifact_id: &str,
    ) -> DeployServiceResult<ArtifactResponse> {
        self.retrieve_artifact_repo(tenant_id, artifact_id).await
    }

    async fn retain_artifact(&self, tenant_id: i64, artifact_id: &str) -> DeployServiceResult<()> {
        self.retain_artifact_repo(tenant_id, artifact_id).await
    }

    async fn create_artifact_from_upload_session(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
        checksum_sha256: &str,
    ) -> DeployServiceResult<ArtifactResponse> {
        self.create_artifact_from_upload_session_repo(tenant_id, upload_session_id, checksum_sha256)
            .await
    }

    async fn list_releases(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ReleasePage> {
        self.list_releases_repo(tenant_id, site_id, page, page_size)
            .await
    }

    async fn retrieve_release(
        &self,
        tenant_id: i64,
        site_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<ReleaseResponse> {
        self.retrieve_release_repo(tenant_id, site_id, release_id)
            .await
    }

    async fn create_release(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateReleaseRequest,
    ) -> DeployServiceResult<ReleaseResponse> {
        self.create_release_repo(tenant_id, site_id, request).await
    }

    async fn find_release_by_idempotency_key(
        &self,
        tenant_id: i64,
        site_id: &str,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<ReleaseResponse>> {
        self.find_release_by_idempotency_key_repo(tenant_id, site_id, idempotency_key)
            .await
    }

    async fn list_env_variables(
        &self,
        tenant_id: i64,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<EnvVariablePage> {
        self.list_env_variables_repo(tenant_id, site_id, environment)
            .await
    }

    async fn create_env_variable(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<EnvVariableResponse> {
        self.create_env_variable_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_certificates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage> {
        self.list_certificates_repo(tenant_id, page, page_size)
            .await
    }

    async fn create_certificate(
        &self,
        tenant_id: i64,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse> {
        self.create_certificate_repo(tenant_id, request).await
    }

    async fn upload_custom_certificate(
        &self,
        tenant_id: i64,
        request: &UploadCustomCertificateRequest,
        certificate_upload: &DeployUploadSessionResponse,
        private_key_upload: &DeployUploadSessionResponse,
    ) -> DeployServiceResult<CertificateResponse> {
        self.upload_custom_certificate_repo(
            tenant_id,
            request,
            certificate_upload,
            private_key_upload,
        )
        .await
    }

    async fn find_certificate_by_idempotency_key(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<CertificateResponse>> {
        self.find_certificate_by_idempotency_key_repo(tenant_id, idempotency_key)
            .await
    }

    async fn retrieve_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse> {
        self.retrieve_certificate_repo(tenant_id, certificate_id)
            .await
    }

    async fn delete_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<()> {
        self.delete_certificate_repo(tenant_id, certificate_id)
            .await
    }

    async fn renew_certificate(
        &self,
        tenant_id: i64,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse> {
        self.renew_certificate_repo(tenant_id, certificate_id).await
    }

    async fn list_health_checks(
        &self,
        tenant_id: i64,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage> {
        self.list_health_checks_repo(tenant_id, site_id).await
    }

    async fn create_health_check(
        &self,
        tenant_id: i64,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse> {
        self.create_health_check_repo(tenant_id, site_id, request)
            .await
    }

    async fn list_nginx_configs(
        &self,
        tenant_id: Option<i64>,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<NginxConfigPage> {
        self.list_nginx_configs_repo(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        tenant_id: i64,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.create_nginx_config_repo(tenant_id, request).await
    }

    async fn retrieve_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.retrieve_nginx_config_repo(tenant_id, config_id).await
    }

    async fn update_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.update_nginx_config_repo(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxValidateResponse> {
        self.validate_nginx_config_repo(tenant_id, config_id).await
    }

    async fn deploy_nginx_config(
        &self,
        tenant_id: Option<i64>,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse> {
        self.deploy_nginx_config_repo(tenant_id, config_id).await
    }

    async fn reload_nginx(&self) -> DeployServiceResult<NginxReloadResponse> {
        self.reload_nginx_repo().await
    }

    async fn retrieve_nginx_status(
        &self,
        tenant_id: Option<i64>,
    ) -> DeployServiceResult<NginxStatusResponse> {
        self.retrieve_nginx_status_repo(tenant_id).await
    }

    async fn list_servers(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ServerPage> {
        self.list_servers_repo(tenant_id, page, page_size).await
    }

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        self.create_server_repo(tenant_id, request).await
    }

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AuditLogPage> {
        self.list_audit_logs_repo(tenant_id, page, page_size).await
    }

    async fn insert_audit_log(&self, command: InsertAuditLogCommand) -> DeployServiceResult<()> {
        self.insert_audit_log_repo(&command).await
    }

    async fn create_upload_session_ref(
        &self,
        tenant_id: i64,
        context: &DeployAppRequestContext,
        request: &CreateDeployUploadSessionRequest,
        drive: &DeployUploadSessionResponse,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        self.create_upload_session_ref_repo(tenant_id, context, request, drive)
            .await
    }

    async fn find_upload_session_by_idempotency_key(
        &self,
        tenant_id: i64,
        idempotency_key: &str,
    ) -> DeployServiceResult<Option<DeployUploadSessionResponse>> {
        self.find_upload_session_by_idempotency_key_repo(tenant_id, idempotency_key)
            .await
    }

    async fn retrieve_upload_session_ref(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        self.retrieve_upload_session_ref_repo(tenant_id, upload_session_id)
            .await
    }

    async fn update_upload_session_status(
        &self,
        tenant_id: i64,
        upload_session_id: &str,
        status: i32,
        drive_node_id: Option<&str>,
    ) -> DeployServiceResult<DeployUploadSessionResponse> {
        self.update_upload_session_status_repo(tenant_id, upload_session_id, status, drive_node_id)
            .await
    }
}

#[async_trait]
impl DeployRuntimeAssignmentRepositoryPort for DeployRepository {
    async fn latest_runtime_assignment(
        &self,
        target_uuid: &str,
    ) -> sdkwork_deploy_contract::DeployServiceResult<Option<RuntimeAssignmentState>> {
        self.latest_runtime_assignment_repo(target_uuid).await
    }

    async fn begin_runtime_assignment_mutation(
        &self,
        target_uuid: &str,
        tenant_id: i64,
    ) -> sdkwork_deploy_contract::DeployServiceResult<Box<dyn DeployRuntimeAssignmentMutationPort>>
    {
        self.begin_runtime_assignment_mutation_repo(target_uuid, tenant_id)
            .await
    }

    async fn claim_due_runtime_assignments(
        &self,
        maximum_items: i64,
        now: &str,
        lease_owner: &str,
        lease_expires_at: &str,
        maximum_attempts: i32,
    ) -> sdkwork_deploy_contract::DeployServiceResult<Vec<RuntimeAssignmentState>> {
        self.claim_due_runtime_assignments_repo(
            maximum_items,
            now,
            lease_owner,
            lease_expires_at,
            maximum_attempts,
        )
        .await
    }

    async fn mark_runtime_assignment_published(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        receipt: &RuntimeAssignmentReceipt,
        published_at: &str,
    ) -> sdkwork_deploy_contract::DeployServiceResult<()> {
        self.mark_runtime_assignment_published_repo(
            assignment_uuid,
            lease_owner,
            receipt,
            published_at,
        )
        .await
    }

    async fn mark_runtime_assignment_failed(
        &self,
        assignment_uuid: &str,
        lease_owner: &str,
        error_code: &str,
        next_attempt_at: Option<&str>,
        updated_at: &str,
    ) -> sdkwork_deploy_contract::DeployServiceResult<()> {
        self.mark_runtime_assignment_failed_repo(
            assignment_uuid,
            lease_owner,
            error_code,
            next_attempt_at,
            updated_at,
        )
        .await
    }

    async fn list_runtime_assignments_requiring_observation(
        &self,
        maximum_items: i64,
    ) -> sdkwork_deploy_contract::DeployServiceResult<Vec<RuntimeAssignmentState>> {
        self.list_runtime_assignments_requiring_observation_repo(maximum_items)
            .await
    }

    async fn list_active_runtime_assignments_after(
        &self,
        after_target_uuid: Option<&str>,
        maximum_items: i64,
    ) -> sdkwork_deploy_contract::DeployServiceResult<Vec<RuntimeAssignmentState>> {
        self.list_active_runtime_assignments_after_repo(after_target_uuid, maximum_items)
            .await
    }

    async fn persist_runtime_observation(
        &self,
        assignment_uuid: &str,
        observation: &RuntimeObservationEvidence,
        ingested_at: &str,
    ) -> sdkwork_deploy_contract::DeployServiceResult<RuntimeObservationPersistenceResult> {
        self.persist_runtime_observation_repo(assignment_uuid, observation, ingested_at)
            .await
    }
}
