use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::app_delivery::*;
use crate::dto::*;
use crate::problem::DeployServiceResult;
use crate::site_composition::{SiteCompositionResponse, UpdateSiteCompositionRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAppRequestContext {
    pub tenant_id: i64,
    pub actor_id: Option<i64>,
    pub organization_id: Option<i64>,
    pub session_id: Option<String>,
    #[serde(default, skip_serializing)]
    pub auth_token: Option<String>,
    #[serde(default, skip_serializing)]
    pub access_token: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployBackendRequestContext {
    pub operator_id: Option<i64>,
    pub tenant_id: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListSitesQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    pub status: Option<i32>,
    // PAGINATION_SPEC §3：query 参数使用 lower_snake_case 规范词汇。
    pub site_type: Option<i32>,
    pub keyword: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListDomainZonesQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[async_trait]
pub trait DeployAppApi: Send + Sync {
    async fn list_domain_zones(
        &self,
        _context: &DeployAppRequestContext,
        _query: &ListDomainZonesQuery,
    ) -> DeployServiceResult<DomainZonePage> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn create_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn update_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _request: &UpdateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn delete_domain_zone(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
    ) -> DeployServiceResult<()> {
        Err(crate::DeployServiceError::Internal(
            "domain zone API is not implemented".to_owned(),
        ))
    }

    async fn list_domain_hostnames(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<DomainHostnamePage> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn create_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _request: &CreateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn delete_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<()> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn update_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
        _request: &UpdateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname API is not implemented".to_owned(),
        ))
    }

    async fn verify_domain_hostname(
        &self,
        _context: &DeployAppRequestContext,
        _zone_id: &str,
        _hostname_id: &str,
    ) -> DeployServiceResult<DomainVerifyResponse> {
        Err(crate::DeployServiceError::Internal(
            "domain hostname verification API is not implemented".to_owned(),
        ))
    }

    async fn list_sites(
        &self,
        context: &DeployAppRequestContext,
        query: &ListSitesQuery,
    ) -> DeployServiceResult<SitePage>;

    async fn create_site(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn retrieve_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn update_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &UpdateSiteRequest,
    ) -> DeployServiceResult<SiteResponse>;

    async fn update_site_composition(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        expected_site_version: i64,
        idempotency_key: &str,
        request: &UpdateSiteCompositionRequest,
    ) -> DeployServiceResult<SiteCompositionResponse>;

    async fn delete_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<()>;

    async fn activate_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn pause_site(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<SiteResponse>;

    async fn list_deployments(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
        status: Option<i32>,
        cursor: Option<&str>,
    ) -> DeployServiceResult<DeploymentPage>;

    async fn create_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateDeploymentRequest,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn retrieve_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn rollback_deployment(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<DeploymentResponse>;

    async fn list_artifacts(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ArtifactPage>;

    async fn create_artifact(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateArtifactRequest,
    ) -> DeployServiceResult<ArtifactResponse>;

    async fn retrieve_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<ArtifactResponse>;

    async fn retain_artifact(
        &self,
        context: &DeployAppRequestContext,
        artifact_id: &str,
    ) -> DeployServiceResult<()>;

    async fn list_releases(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ReleasePage>;

    async fn retrieve_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<ReleaseResponse>;

    async fn create_release(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateReleaseRequest,
    ) -> DeployServiceResult<ReleaseResponse>;

    async fn list_env_variables(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        environment: Option<&str>,
    ) -> DeployServiceResult<EnvVariablePage>;

    async fn create_env_variable(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateEnvVariableRequest,
    ) -> DeployServiceResult<EnvVariableResponse>;

    async fn list_certificates(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificatePage>;

    async fn create_certificate(
        &self,
        context: &DeployAppRequestContext,
        idempotency_key: &str,
        request: &CreateCertificateRequest,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn retrieve_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn delete_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<()>;

    async fn renew_certificate(
        &self,
        context: &DeployAppRequestContext,
        certificate_id: &str,
    ) -> DeployServiceResult<CertificateResponse>;

    async fn list_health_checks(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
    ) -> DeployServiceResult<HealthCheckPage>;

    async fn create_health_check(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        request: &CreateHealthCheckRequest,
    ) -> DeployServiceResult<HealthCheckResponse>;

    async fn create_upload_session(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn retrieve_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn complete_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
        request: &CompleteDeployUploadSessionRequest,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn cancel_upload_session(
        &self,
        context: &DeployAppRequestContext,
        upload_session_id: &str,
    ) -> DeployServiceResult<DeployUploadSessionResponse>;

    async fn list_apps(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppPage> {
        Err(crate::DeployServiceError::Internal(
            "list_apps API is not implemented".to_owned(),
        ))
    }

    async fn create_app(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
    ) -> DeployServiceResult<AppResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app API is not implemented".to_owned(),
        ))
    }

    async fn update_app(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _request: &UpdateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        Err(crate::DeployServiceError::Internal(
            "update_app API is not implemented".to_owned(),
        ))
    }

    async fn create_platform_target(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _request: &CreatePlatformTargetRequest,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_platform_target API is not implemented".to_owned(),
        ))
    }

    async fn list_platform_targets(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
    ) -> DeployServiceResult<PlatformTargetPage> {
        Err(crate::DeployServiceError::Internal(
            "list_platform_targets API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_platform_target(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _target_id: &str,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_platform_target API is not implemented".to_owned(),
        ))
    }

    async fn create_source_repository(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _request: &CreateSourceRepositoryRequest,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_source_repository API is not implemented".to_owned(),
        ))
    }

    async fn list_source_repositories(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
    ) -> DeployServiceResult<SourceRepositoryPage> {
        Err(crate::DeployServiceError::Internal(
            "list_source_repositories API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_source_repository(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _repo_id: &str,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_source_repository API is not implemented".to_owned(),
        ))
    }

    async fn create_build_template(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateBuildTemplateRequest,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_build_template API is not implemented".to_owned(),
        ))
    }

    async fn list_build_templates(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<BuildTemplatePage> {
        Err(crate::DeployServiceError::Internal(
            "list_build_templates API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_build_template(
        &self,
        _context: &DeployAppRequestContext,
        _template_id: &str,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_build_template API is not implemented".to_owned(),
        ))
    }

    async fn create_build(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateBuildRequest,
    ) -> DeployServiceResult<BuildResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_build API is not implemented".to_owned(),
        ))
    }

    async fn list_builds(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<BuildPage> {
        Err(crate::DeployServiceError::Internal(
            "list_builds API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_build(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _build_id: &str,
    ) -> DeployServiceResult<BuildResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_build API is not implemented".to_owned(),
        ))
    }

    async fn update_build_state(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _build_id: &str,
        _request: &UpdateBuildStateRequest,
    ) -> DeployServiceResult<BuildResponse> {
        Err(crate::DeployServiceError::Internal(
            "update_build_state API is not implemented".to_owned(),
        ))
    }

    async fn register_package(
        &self,
        _context: &DeployAppRequestContext,
        _request: &RegisterPackageRequest,
    ) -> DeployServiceResult<PackageResponse> {
        Err(crate::DeployServiceError::Internal(
            "register_package API is not implemented".to_owned(),
        ))
    }

    async fn list_packages(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<PackagePage> {
        Err(crate::DeployServiceError::Internal(
            "list_packages API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_package(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _package_id: &str,
    ) -> DeployServiceResult<PackageResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_package API is not implemented".to_owned(),
        ))
    }

    async fn create_app_release(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateAppReleaseRequest,
    ) -> DeployServiceResult<AppReleaseResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app_release API is not implemented".to_owned(),
        ))
    }

    async fn list_app_releases(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppReleasePage> {
        Err(crate::DeployServiceError::Internal(
            "list_app_releases API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app_release(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _release_id: &str,
    ) -> DeployServiceResult<AppReleaseResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app_release API is not implemented".to_owned(),
        ))
    }

    async fn update_app_release_status(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _release_id: &str,
        _release_status: ReleaseStatus,
    ) -> DeployServiceResult<AppReleaseResponse> {
        Err(crate::DeployServiceError::Internal(
            "update_app_release_status API is not implemented".to_owned(),
        ))
    }

    async fn list_channels(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
    ) -> DeployServiceResult<ChannelPage> {
        Err(crate::DeployServiceError::Internal(
            "list_channels API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_channel(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _channel_id: &str,
    ) -> DeployServiceResult<ChannelResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_channel API is not implemented".to_owned(),
        ))
    }

    async fn promote_channel(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _channel_id: &str,
        _request: &PromoteChannelRequest,
    ) -> DeployServiceResult<ChannelRolloutResponse> {
        Err(crate::DeployServiceError::Internal(
            "promote_channel API is not implemented".to_owned(),
        ))
    }

    async fn list_channel_rollouts(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _channel_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<ChannelRolloutPage> {
        Err(crate::DeployServiceError::Internal(
            "list_channel_rollouts API is not implemented".to_owned(),
        ))
    }

    async fn create_app_deployment(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateAppDeploymentRequest,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app_deployment API is not implemented".to_owned(),
        ))
    }

    async fn list_app_deployments(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppDeploymentPage> {
        Err(crate::DeployServiceError::Internal(
            "list_app_deployments API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app_deployment(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _deployment_id: &str,
    ) -> DeployServiceResult<AppDeploymentResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app_deployment API is not implemented".to_owned(),
        ))
    }

    async fn create_signing_identity(
        &self,
        _context: &DeployAppRequestContext,
        _request: &CreateSigningIdentityRequest,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_signing_identity API is not implemented".to_owned(),
        ))
    }

    async fn list_signing_identities(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<SigningIdentityPage> {
        Err(crate::DeployServiceError::Internal(
            "list_signing_identities API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_signing_identity(
        &self,
        _context: &DeployAppRequestContext,
        _identity_id: &str,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_signing_identity API is not implemented".to_owned(),
        ))
    }

    async fn list_usage_events(
        &self,
        _context: &DeployAppRequestContext,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<UsageEventPage> {
        Err(crate::DeployServiceError::Internal(
            "list_usage_events API is not implemented".to_owned(),
        ))
    }

    async fn create_app_database_profile(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _request: &CreateAppDatabaseProfileRequest,
    ) -> DeployServiceResult<AppDatabaseProfileResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app_database_profile API is not implemented".to_owned(),
        ))
    }

    async fn list_app_database_profiles(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppDatabaseProfilePage> {
        Err(crate::DeployServiceError::Internal(
            "list_app_database_profiles API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app_database_profile(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _profile_id: &str,
    ) -> DeployServiceResult<AppDatabaseProfileResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app_database_profile API is not implemented".to_owned(),
        ))
    }

    async fn update_app_database_profile(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _profile_id: &str,
        _request: &UpdateAppDatabaseProfileRequest,
    ) -> DeployServiceResult<AppDatabaseProfileResponse> {
        Err(crate::DeployServiceError::Internal(
            "update_app_database_profile API is not implemented".to_owned(),
        ))
    }

    async fn create_app_database_migration(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _profile_id: &str,
        _request: &CreateAppDatabaseMigrationRequest,
    ) -> DeployServiceResult<AppDatabaseMigrationResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app_database_migration API is not implemented".to_owned(),
        ))
    }

    async fn list_app_database_migrations(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _profile_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppDatabaseMigrationPage> {
        Err(crate::DeployServiceError::Internal(
            "list_app_database_migrations API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app_database_migration(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _profile_id: &str,
        _migration_id: &str,
    ) -> DeployServiceResult<AppDatabaseMigrationResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app_database_migration API is not implemented".to_owned(),
        ))
    }

    async fn create_app_environment(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _request: &CreateAppEnvironmentRequest,
    ) -> DeployServiceResult<AppEnvironmentResponse> {
        Err(crate::DeployServiceError::Internal(
            "create_app_environment API is not implemented".to_owned(),
        ))
    }

    async fn list_app_environments(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<AppEnvironmentPage> {
        Err(crate::DeployServiceError::Internal(
            "list_app_environments API is not implemented".to_owned(),
        ))
    }

    async fn retrieve_app_environment(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _environment_id: &str,
    ) -> DeployServiceResult<AppEnvironmentResponse> {
        Err(crate::DeployServiceError::Internal(
            "retrieve_app_environment API is not implemented".to_owned(),
        ))
    }

    async fn update_app_environment(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _environment_id: &str,
        _request: &UpdateAppEnvironmentRequest,
    ) -> DeployServiceResult<AppEnvironmentResponse> {
        Err(crate::DeployServiceError::Internal(
            "update_app_environment API is not implemented".to_owned(),
        ))
    }

    async fn promote_environment(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _environment_id: &str,
        _request: &PromoteEnvironmentRequest,
    ) -> DeployServiceResult<EnvironmentPromotionResponse> {
        Err(crate::DeployServiceError::Internal(
            "promote_environment API is not implemented".to_owned(),
        ))
    }

    async fn list_environment_promotions(
        &self,
        _context: &DeployAppRequestContext,
        _app_id: &str,
        _environment_id: &str,
        _page: i32,
        _page_size: i32,
    ) -> DeployServiceResult<EnvironmentPromotionPage> {
        Err(crate::DeployServiceError::Internal(
            "list_environment_promotions API is not implemented".to_owned(),
        ))
    }
}

#[async_trait]
pub trait DeployBackendApi: Send + Sync {
    async fn list_nginx_configs(
        &self,
        context: &DeployBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<NginxConfigPage>;

    async fn create_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn retrieve_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn update_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn validate_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxValidateResponse>;

    async fn deploy_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<NginxConfigResponse>;

    async fn reload_nginx(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<NginxReloadResponse>;

    async fn retrieve_nginx_status(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<NginxStatusResponse>;

    async fn list_servers(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
        cluster_id: Option<String>,
    ) -> DeployServiceResult<ServerPage>;

    async fn create_server(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn update_server(
        &self,
        context: &DeployBackendRequestContext,
        server_id: &str,
        request: &UpdateServerRequest,
    ) -> DeployServiceResult<ServerResponse>;

    async fn list_node_clusters(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<NodeClusterPage>;

    async fn create_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse>;

    async fn update_node_cluster(
        &self,
        context: &DeployBackendRequestContext,
        cluster_id: &str,
        request: &UpdateNodeClusterRequest,
    ) -> DeployServiceResult<NodeClusterResponse>;

    async fn list_audit_logs(
        &self,
        context: &DeployBackendRequestContext,
        query: &AuditLogQuery,
        cursor: Option<&str>,
    ) -> DeployServiceResult<AuditLogPage>;

    async fn list_entitlement_projections(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<EntitlementProjectionPage>;

    async fn list_build_queue(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildQueuePage>;

    async fn list_runner_health(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<RunnerHealthPage>;

    async fn create_acme_account(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateAcmeAccountRequest,
    ) -> DeployServiceResult<AcmeAccountResponse>;

    async fn list_acme_accounts(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AcmeAccountPage>;

    async fn request_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        request: &RequestCertificateOrderRequest,
    ) -> DeployServiceResult<CertificateOrderResponse>;

    async fn advance_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
    ) -> DeployServiceResult<CertificateOrderResponse>;

    async fn fail_certificate_order(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        error_code: &str,
    ) -> DeployServiceResult<()>;

    async fn record_challenge_result(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        challenge_id: Option<&str>,
        valid: bool,
    ) -> DeployServiceResult<()>;

    async fn store_certificate_version(
        &self,
        context: &DeployBackendRequestContext,
        request: &StoreCertificateVersionRequest,
    ) -> DeployServiceResult<CertificateOrderResponse>;

    async fn list_certificate_orders(
        &self,
        context: &DeployBackendRequestContext,
        certificate_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateOrderPage>;

    async fn list_certificate_challenges(
        &self,
        context: &DeployBackendRequestContext,
        order_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<CertificateChallengePage>;

    async fn run_retention(
        &self,
        context: &DeployBackendRequestContext,
        request: &RetentionRunRequest,
    ) -> DeployServiceResult<RetentionRunResponse>;

    async fn rebuild_usage_daily(
        &self,
        context: &DeployBackendRequestContext,
        request: &UsageReconciliationRequest,
    ) -> DeployServiceResult<UsageReconciliationResponse>;

    async fn list_signing_identity_health(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SigningIdentityHealthPage>;

    /// Ingests a Git webhook push event (GitHub-compatible payload with
    /// `X-Hub-Signature-256` HMAC verification) and triggers builds for the
    /// matched repository's active targets on the default branch.
    async fn ingest_source_event(
        &self,
        context: &DeployBackendRequestContext,
        payload: &[u8],
        signature: Option<&str>,
    ) -> DeployServiceResult<SourceEventIngestResponse>;

    async fn list_source_events(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SourceEventPage>;
}
