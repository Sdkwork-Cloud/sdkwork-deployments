//! `DeployRepositoryPort` trait implementation delegating to SQLx repository modules.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    AppDeploymentPage, AppDeploymentResponse, AppPage, AppReleasePage, AppReleaseResponse,
    AppResponse, ArtifactPage, ArtifactResponse, AuditLogPage, BuildPage, BuildResponse,
    BuildTemplatePage, BuildTemplateResponse, CertificatePage, CertificateResponse, ChannelPage,
    ChannelResponse, ChannelRolloutPage, ChannelRolloutResponse, CreateAppDeploymentRequest,
    CreateAppReleaseRequest, CreateAppRequest, CreateArtifactRequest, CreateBuildRequest,
    CreateBuildTemplateRequest, CreateCertificateRequest, CreateDeployUploadSessionRequest,
    CreateDeploymentRequest, CreateDomainHostnameRequest, CreateDomainZoneRequest,
    CreateEnvVariableRequest, CreateHealthCheckRequest, CreateNginxConfigRequest,
    CreateNodeClusterRequest, CreatePlatformTargetRequest, CreateReleaseRequest,
    CreateServerRequest, CreateSigningIdentityRequest, CreateSiteRequest,
    CreateSourceRepositoryRequest, DeployAppRequestContext, DeployUploadSessionResponse,
    DeploymentPage, DeploymentResponse, DeploymentStatus, DomainHostnamePage,
    DomainHostnameResponse, DomainZonePage, DomainZoneResponse, EnvVariablePage,
    EnvVariableResponse, HealthCheckPage, HealthCheckResponse, ListDomainZonesQuery,
    ListNginxConfigsQuery, ListSitesQuery, NginxConfigPage, NginxConfigResponse,
    NginxReloadResponse, NginxStatusResponse, NginxValidateResponse, NodeClusterPage,
    NodeClusterResponse, PackagePage, PackageResponse, PlatformTargetPage, PlatformTargetResponse,
    PromoteChannelRequest, RegisterPackageRequest, ReleasePage, ReleaseResponse, ReleaseStatus,
    ServerPage, ServerResponse, SigningIdentityPage, SigningIdentityResponse, SitePage,
    SiteResponse, SourceRepositoryPage, SourceRepositoryResponse, UpdateAppRequest,
    UpdateBuildStateRequest, UpdateDomainHostnameRequest, UpdateDomainZoneRequest,
    UpdateNginxConfigRequest, UpdateNodeClusterRequest, UpdateServerRequest, UpdateSiteRequest,
};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_deploy_web_port::RuntimeAssignmentReceipt;
use sdkwork_intelligence_deploy_service::repository::InsertAuditLogCommand;
use sdkwork_intelligence_deploy_service::runtime_publication::{
    DeployRuntimeAssignmentMutationPort, DeployRuntimeAssignmentRepositoryPort,
    RuntimeAssignmentState, RuntimeObservationEvidence, RuntimeObservationPersistenceResult,
};
use sdkwork_intelligence_deploy_service::{DeployRepositoryPort, DomainVerificationChallenge};

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

    async fn list_domain_zones(
        &self,
        tenant_id: i64,
        query: &ListDomainZonesQuery,
    ) -> DeployServiceResult<DomainZonePage> {
        self.list_domain_zones_repo(tenant_id, query).await
    }

    async fn create_domain_zone(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        self.create_domain_zone_repo(tenant_id, organization_id, actor_id, request)
            .await
    }

    async fn retrieve_domain_zone(
        &self,
        tenant_id: i64,
        zone_id: &str,
    ) -> DeployServiceResult<DomainZoneResponse> {
        self.retrieve_domain_zone_repo(tenant_id, zone_id).await
    }

    async fn update_domain_zone(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        zone_id: &str,
        request: &UpdateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        self.update_domain_zone_repo(tenant_id, actor_id, zone_id, request)
            .await
    }

    async fn delete_domain_zone(&self, tenant_id: i64, zone_id: &str) -> DeployServiceResult<()> {
        self.delete_domain_zone_repo(tenant_id, zone_id).await
    }

    async fn list_domain_hostnames(
        &self,
        tenant_id: i64,
        zone_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainHostnamePage> {
        self.list_domain_hostnames_repo(tenant_id, zone_id, page, page_size)
            .await
    }

    async fn create_domain_hostname(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        zone_id: &str,
        request: &CreateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        self.create_domain_hostname_repo(tenant_id, actor_id, zone_id, request)
            .await
    }

    async fn retrieve_domain_hostname(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        self.retrieve_domain_hostname_repo(tenant_id, zone_id, hostname_id)
            .await
    }

    async fn delete_domain_hostname(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<()> {
        self.delete_domain_hostname_repo(tenant_id, zone_id, hostname_id)
            .await
    }

    async fn update_domain_hostname(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        zone_id: &str,
        hostname_id: &str,
        request: &UpdateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        self.update_domain_hostname_repo(tenant_id, actor_id, zone_id, hostname_id, request)
            .await
    }

    async fn domain_hostname_verification_challenge(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<DomainVerificationChallenge> {
        self.domain_hostname_verification_challenge_repo(tenant_id, zone_id, hostname_id)
            .await
    }

    async fn confirm_domain_hostname_verification(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
        verification_id: &str,
        observed_sha256: &str,
        verifier_identity: &str,
    ) -> DeployServiceResult<bool> {
        self.confirm_domain_hostname_verification_repo(
            tenant_id,
            zone_id,
            hostname_id,
            verification_id,
            observed_sha256,
            verifier_identity,
        )
        .await
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

    async fn list_deployments(
        &self,
        tenant_id: i64,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
        cursor: Option<&str>,
    ) -> DeployServiceResult<DeploymentPage> {
        self.list_deployments_repo(tenant_id, site_id, page, page_size, status, cursor)
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

    async fn create_artifact_from_drive(
        &self,
        tenant_id: i64,
        request: &CreateArtifactRequest,
    ) -> DeployServiceResult<ArtifactResponse> {
        self.create_artifact_from_drive_repo(tenant_id, request)
            .await
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
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        idempotency_key: &str,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse> {
        self.create_certificate_repo(
            tenant_id,
            organization_id,
            actor_id,
            idempotency_key,
            request,
        )
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
        cluster_id: Option<String>,
    ) -> DeployServiceResult<ServerPage> {
        self.list_servers_repo(tenant_id, page, page_size, cluster_id)
            .await
    }

    async fn create_server(
        &self,
        tenant_id: i64,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        self.create_server_repo(tenant_id, request).await
    }

    async fn update_server(
        &self,
        tenant_id: i64,
        server_id: &str,
        request: &UpdateServerRequest,
    ) -> DeployServiceResult<ServerResponse> {
        self.update_server_repo(tenant_id, server_id, request).await
    }

    async fn list_node_clusters(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<NodeClusterPage> {
        self.list_node_clusters_repo(tenant_id, page, page_size)
            .await
    }

    async fn create_node_cluster(
        &self,
        tenant_id: i64,
        request: &CreateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse> {
        self.create_node_cluster_repo(tenant_id, request).await
    }

    async fn update_node_cluster(
        &self,
        tenant_id: i64,
        cluster_id: &str,
        request: &UpdateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse> {
        self.update_node_cluster_repo(tenant_id, cluster_id, request)
            .await
    }

    async fn list_audit_logs(
        &self,
        tenant_id: Option<i64>,
        query: &sdkwork_deploy_contract::AuditLogQuery,
        cursor: Option<&str>,
    ) -> DeployServiceResult<AuditLogPage> {
        self.list_audit_logs_repo(tenant_id, query, cursor).await
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

    // -- unified app delivery (REQ-2026-0002) --------------------------------

    async fn create_app(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        self.create_app_repo(tenant_id, organization_id, actor_id, request)
            .await
    }

    async fn list_apps(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppPage> {
        self.list_apps_repo(tenant_id, page, page_size).await
    }

    async fn retrieve_app(&self, tenant_id: i64, app_id: &str) -> DeployServiceResult<AppResponse> {
        self.retrieve_app_repo(tenant_id, app_id).await
    }

    async fn update_app(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        app_id: &str,
        request: &UpdateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        self.update_app_repo(tenant_id, actor_id, app_id, request)
            .await
    }

    async fn create_platform_target(
        &self,
        tenant_id: i64,
        app_id: &str,
        actor_id: Option<i64>,
        request: &CreatePlatformTargetRequest,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        self.create_platform_target_repo(tenant_id, app_id, actor_id, request)
            .await
    }

    async fn list_platform_targets(
        &self,
        tenant_id: i64,
        app_id: &str,
    ) -> DeployServiceResult<PlatformTargetPage> {
        self.list_platform_targets_repo(tenant_id, app_id).await
    }

    async fn retrieve_platform_target(
        &self,
        tenant_id: i64,
        app_id: &str,
        target_id: &str,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        self.retrieve_platform_target_repo(tenant_id, app_id, target_id)
            .await
    }

    async fn create_source_repository(
        &self,
        tenant_id: i64,
        app_id: &str,
        actor_id: Option<i64>,
        request: &CreateSourceRepositoryRequest,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        self.create_source_repository_repo(tenant_id, app_id, actor_id, request)
            .await
    }

    async fn list_source_repositories(
        &self,
        tenant_id: i64,
        app_id: &str,
    ) -> DeployServiceResult<SourceRepositoryPage> {
        self.list_source_repositories_repo(tenant_id, app_id).await
    }

    async fn retrieve_source_repository(
        &self,
        tenant_id: i64,
        app_id: &str,
        repo_id: &str,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        self.retrieve_source_repository_repo(tenant_id, app_id, repo_id)
            .await
    }

    async fn create_build_template(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateBuildTemplateRequest,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        self.create_build_template_repo(tenant_id, actor_id, request)
            .await
    }

    async fn list_build_templates(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildTemplatePage> {
        self.list_build_templates_repo(tenant_id, page, page_size)
            .await
    }

    async fn retrieve_build_template(
        &self,
        tenant_id: i64,
        template_id: &str,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        self.retrieve_build_template_repo(tenant_id, template_id)
            .await
    }

    async fn create_build(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateBuildRequest,
    ) -> DeployServiceResult<BuildResponse> {
        self.create_build_repo(tenant_id, actor_id, request).await
    }

    async fn list_builds(
        &self,
        tenant_id: i64,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildPage> {
        self.list_builds_repo(tenant_id, app_id, page, page_size)
            .await
    }

    async fn retrieve_build(
        &self,
        tenant_id: i64,
        app_id: &str,
        build_id: &str,
    ) -> DeployServiceResult<BuildResponse> {
        self.retrieve_build_repo(tenant_id, app_id, build_id).await
    }

    async fn update_build_state(
        &self,
        tenant_id: i64,
        app_id: &str,
        build_id: &str,
        request: &UpdateBuildStateRequest,
    ) -> DeployServiceResult<BuildResponse> {
        self.update_build_state_repo(tenant_id, app_id, build_id, request)
            .await
    }

    async fn claim_next_build(
        &self,
        tenant_id: i64,
        runner_node_uuid: &str,
        runner_version: &str,
    ) -> DeployServiceResult<Option<BuildResponse>> {
        self.claim_next_build_repo(tenant_id, runner_node_uuid, runner_version)
            .await
    }

    async fn resolve_build_platform(
        &self,
        tenant_id: i64,
        build_id: &str,
    ) -> DeployServiceResult<(String, String, String)> {
        self.resolve_build_platform_repo(tenant_id, build_id).await
    }

    async fn register_package(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &RegisterPackageRequest,
    ) -> DeployServiceResult<PackageResponse> {
        self.register_package_repo(tenant_id, actor_id, request)
            .await
    }

    async fn list_packages(
        &self,
        tenant_id: i64,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<PackagePage> {
        self.list_packages_repo(tenant_id, app_id, page, page_size)
            .await
    }

    async fn retrieve_package(
        &self,
        tenant_id: i64,
        app_id: &str,
        package_id: &str,
    ) -> DeployServiceResult<PackageResponse> {
        self.retrieve_package_repo(tenant_id, app_id, package_id)
            .await
    }

    async fn create_app_release(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateAppReleaseRequest,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.create_app_release_repo(tenant_id, actor_id, request)
            .await
    }

    async fn list_app_releases(
        &self,
        tenant_id: i64,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppReleasePage> {
        self.list_app_releases_repo(tenant_id, app_id, page, page_size)
            .await
    }

    async fn retrieve_app_release(
        &self,
        tenant_id: i64,
        app_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.retrieve_app_release_repo(tenant_id, app_id, release_id)
            .await
    }

    async fn update_app_release_status(
        &self,
        tenant_id: i64,
        app_id: &str,
        release_id: &str,
        release_status: ReleaseStatus,
    ) -> DeployServiceResult<AppReleaseResponse> {
        self.update_app_release_status_repo(tenant_id, app_id, release_id, release_status)
            .await
    }

    async fn ensure_release_channel(
        &self,
        tenant_id: i64,
        app_id: &str,
        target_id: &str,
        channel_key: &str,
    ) -> DeployServiceResult<ChannelResponse> {
        self.ensure_release_channel_repo(tenant_id, app_id, target_id, channel_key)
            .await
    }

    async fn retrieve_channel(
        &self,
        tenant_id: i64,
        app_id: &str,
        channel_id: &str,
    ) -> DeployServiceResult<ChannelResponse> {
        self.retrieve_channel_repo(tenant_id, app_id, channel_id)
            .await
    }

    async fn list_channels(
        &self,
        tenant_id: i64,
        app_id: &str,
    ) -> DeployServiceResult<ChannelPage> {
        self.list_channels_repo(tenant_id, app_id).await
    }

    async fn promote_channel(
        &self,
        tenant_id: i64,
        app_id: &str,
        channel_id: &str,
        actor_id: Option<i64>,
        request: &PromoteChannelRequest,
    ) -> DeployServiceResult<ChannelRolloutResponse> {
        self.promote_channel_repo(tenant_id, app_id, channel_id, actor_id, request)
            .await
    }

    async fn list_channel_rollouts(
        &self,
        tenant_id: i64,
        app_id: &str,
        channel_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ChannelRolloutPage> {
        self.list_channel_rollouts_repo(tenant_id, app_id, channel_id, page, page_size)
            .await
    }

    async fn create_app_deployment(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateAppDeploymentRequest,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        self.create_app_deployment_repo(tenant_id, actor_id, request)
            .await
    }

    async fn list_app_deployments(
        &self,
        tenant_id: i64,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppDeploymentPage> {
        self.list_app_deployments_repo(tenant_id, app_id, page, page_size)
            .await
    }

    async fn retrieve_app_deployment(
        &self,
        tenant_id: i64,
        app_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        self.retrieve_app_deployment_repo(tenant_id, app_id, deployment_id)
            .await
    }

    async fn list_review_pending_deployments(
        &self,
        tenant_id: i64,
        limit: i64,
    ) -> DeployServiceResult<Vec<AppDeploymentResponse>> {
        self.list_review_pending_deployments_repo(tenant_id, limit)
            .await
    }

    async fn update_app_deployment_state(
        &self,
        tenant_id: i64,
        app_id: &str,
        deployment_id: &str,
        deployment_status: DeploymentStatus,
        platform_review_ref: Option<&str>,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        self.update_app_deployment_state_repo(
            tenant_id,
            app_id,
            deployment_id,
            deployment_status,
            platform_review_ref,
        )
        .await
    }

    async fn create_signing_identity(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        request: &CreateSigningIdentityRequest,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        self.create_signing_identity_repo(tenant_id, actor_id, request)
            .await
    }

    async fn list_signing_identities(
        &self,
        tenant_id: i64,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SigningIdentityPage> {
        self.list_signing_identities_repo(tenant_id, page, page_size)
            .await
    }

    async fn retrieve_signing_identity(
        &self,
        tenant_id: i64,
        identity_id: &str,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        self.retrieve_signing_identity_repo(tenant_id, identity_id)
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
