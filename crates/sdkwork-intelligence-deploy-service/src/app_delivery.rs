//! Unified application delivery service surface (REQ-2026-0002): validation
//! of app-kind/platform rules, semantic versions, package formats and size
//! ceilings, then delegation to the repository port.

use sdkwork_deploy_contract::{
    AppKind, AppPage, AppReleasePage, AppReleaseResponse, AppResponse, BuildPage, BuildResponse,
    BuildTemplatePage, BuildTemplateResponse, ChannelKey, ChannelPage, ChannelResponse,
    ChannelRolloutPage, ChannelRolloutResponse, CreateAppDeploymentRequest,
    CreateAppReleaseRequest, CreateAppRequest, CreateBuildRequest, CreateBuildTemplateRequest,
    CreatePlatformTargetRequest, CreateSigningIdentityRequest, CreateSourceRepositoryRequest,
    DeployAppRequestContext, DeployServiceError, DeployServiceResult, DeploymentStatus,
    PackagePage, PackageResponse, PlatformTargetPage, PlatformTargetResponse,
    PromoteChannelRequest, RegisterPackageRequest, ReleaseStatus, SigningIdentityPage,
    SigningIdentityResponse, SourceRepositoryPage, SourceRepositoryResponse, UpdateAppRequest,
    UpdateBuildStateRequest,
};
use sdkwork_deploy_core::{
    required_identity_field, validate_app_kind_platform, validate_package_format_for_platform,
    validate_package_size, validate_platform_identity, validate_sha256_hex, RequiredIdentityField,
    SemanticVersion,
};

use crate::{repository::InsertAuditLogCommand, DeployService};

const CHANNEL_KEYS: &[&str] = &["stable", "beta", "alpha", "qa"];
const REPO_PROVIDERS: &[&str] = &["GITHUB", "GITEE", "GITLAB", "SELF_HOSTED"];
const CLONE_MODES: &[&str] = &["FULL", "SHALLOW"];

impl DeployService {
    fn tenant_id(context: &DeployAppRequestContext) -> DeployServiceResult<i64> {
        if context.tenant_id <= 0 {
            return Err(DeployServiceError::forbidden(
                "app delivery operations require tenant authorization",
            ));
        }
        Ok(context.tenant_id)
    }

    async fn audit_app_action(
        &self,
        context: &DeployAppRequestContext,
        action: &str,
        target_uuid: &str,
    ) -> DeployServiceResult<()> {
        self.repository
            .insert_audit_log(InsertAuditLogCommand {
                tenant_id: context.tenant_id,
                organization_id: context.organization_id.unwrap_or(0),
                operator_id: context.actor_id.unwrap_or(0),
                action: action.to_owned(),
                target_type: "app".to_owned(),
                target_id: None,
                target_uuid: Some(target_uuid.to_owned()),
            })
            .await
    }

    // -- apps -----------------------------------------------------------------

    pub async fn create_app(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.name.trim().is_empty() {
            return Err(DeployServiceError::validation("app name is required"));
        }
        if let Some(slug) = request.slug.as_deref() {
            if !is_bounded_slug(slug) {
                return Err(DeployServiceError::validation(
                    "app slug must be 1..=120 lowercase ascii letters, digits, and hyphens",
                ));
            }
        }
        let app = self
            .repository
            .create_app(
                tenant_id,
                context.organization_id,
                context.actor_id,
                request,
            )
            .await?;
        self.audit_app_action(context, "app.create", &app.id)
            .await?;
        Ok(app)
    }

    pub async fn list_apps(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository.list_apps(tenant_id, page, page_size).await
    }

    pub async fn retrieve_app(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<AppResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository.retrieve_app(tenant_id, app_id).await
    }

    pub async fn update_app(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &UpdateAppRequest,
    ) -> DeployServiceResult<AppResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if let Some(name) = request.name.as_deref() {
            if name.trim().is_empty() {
                return Err(DeployServiceError::validation("app name must not be empty"));
            }
        }
        let app = self
            .repository
            .update_app(tenant_id, context.actor_id, app_id, request)
            .await?;
        self.audit_app_action(context, "app.update", &app.id)
            .await?;
        Ok(app)
    }

    // -- platform targets ------------------------------------------------------

    pub async fn create_platform_target(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &CreatePlatformTargetRequest,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.target_key.trim().is_empty() || request.target_key.len() > 120 {
            return Err(DeployServiceError::validation(
                "targetKey must be 1..=120 characters",
            ));
        }
        if let Some(channels) = request.allowed_channels.as_deref() {
            validate_channel_keys(channels)?;
        }
        // app-kind/platform compatibility requires the app's kind.
        let app = self.repository.retrieve_app(tenant_id, app_id).await?;
        let Some(app_kind) = AppKind::parse(&app.app_kind) else {
            return Err(DeployServiceError::validation(format!(
                "app kind {} is unknown",
                app.app_kind
            )));
        };
        validate_app_kind_platform(app_kind.as_str(), request.platform.as_str())
            .map_err(DeployServiceError::validation)?;

        // Platform identity is mandatory for platforms that require one.
        let identity = match request.platform.as_str() {
            "IOS" => request.bundle_id.as_deref(),
            "ANDROID" => request.package_name.as_deref(),
            "WECHAT" | "DOUYIN" => request.app_id.as_deref(),
            "HARMONYOS" => request.bundle_name.as_deref(),
            _ => None,
        };
        if required_identity_field(request.platform.as_str()) != RequiredIdentityField::None {
            let identity = identity.ok_or_else(|| {
                DeployServiceError::validation(format!(
                    "platform {} requires {}",
                    request.platform.as_str(),
                    required_identity_field(request.platform.as_str()).as_str()
                ))
            })?;
            validate_platform_identity(request.platform.as_str(), identity)
                .map_err(DeployServiceError::validation)?;
        }

        let target = self
            .repository
            .create_platform_target(tenant_id, app_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "platformTarget.create", &target.id)
            .await?;
        Ok(target)
    }

    pub async fn list_platform_targets(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<PlatformTargetPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_platform_targets(tenant_id, app_id)
            .await
    }

    pub async fn retrieve_platform_target(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        target_id: &str,
    ) -> DeployServiceResult<PlatformTargetResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_platform_target(tenant_id, app_id, target_id)
            .await
    }

    // -- source repositories ----------------------------------------------------

    pub async fn create_source_repository(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        request: &CreateSourceRepositoryRequest,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.repo_key.trim().is_empty() || request.repo_key.len() > 120 {
            return Err(DeployServiceError::validation(
                "repoKey must be 1..=120 characters",
            ));
        }
        if !REPO_PROVIDERS.contains(&request.repo_provider.as_str()) {
            return Err(DeployServiceError::validation(format!(
                "repoProvider must be one of {}",
                REPO_PROVIDERS.join(", ")
            )));
        }
        if !CLONE_MODES
            .iter()
            .any(|mode| Some(*mode) == request.clone_mode.as_deref())
        {
            return Err(DeployServiceError::validation(
                "cloneMode must be FULL or SHALLOW",
            ));
        }
        validate_repo_url(&request.repo_url)?;

        let repo = self
            .repository
            .create_source_repository(tenant_id, app_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "sourceRepository.create", &repo.id)
            .await?;
        Ok(repo)
    }

    pub async fn list_source_repositories(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<SourceRepositoryPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_source_repositories(tenant_id, app_id)
            .await
    }

    pub async fn retrieve_source_repository(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        repo_id: &str,
    ) -> DeployServiceResult<SourceRepositoryResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_source_repository(tenant_id, app_id, repo_id)
            .await
    }

    // -- build templates ---------------------------------------------------------

    pub async fn create_build_template(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateBuildTemplateRequest,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.template_name.trim().is_empty() || request.template_name.len() > 200 {
            return Err(DeployServiceError::validation(
                "templateName must be 1..=200 characters",
            ));
        }
        if request.template_version.trim().is_empty() || request.template_version.len() > 64 {
            return Err(DeployServiceError::validation(
                "templateVersion must be 1..=64 characters",
            ));
        }
        if let Some(commands) = request.commands.as_deref() {
            if commands.len() > 64 {
                return Err(DeployServiceError::validation(
                    "commands must contain at most 64 entries",
                ));
            }
            for command in commands {
                if command.trim().is_empty() || command.len() > 500 {
                    return Err(DeployServiceError::validation(
                        "each command must be 1..=500 characters",
                    ));
                }
            }
        }
        let template = self
            .repository
            .create_build_template(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "buildTemplate.create", &template.id)
            .await?;
        Ok(template)
    }

    pub async fn list_build_templates(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildTemplatePage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_build_templates(tenant_id, page, page_size)
            .await
    }

    pub async fn retrieve_build_template(
        &self,
        context: &DeployAppRequestContext,
        template_id: &str,
    ) -> DeployServiceResult<BuildTemplateResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_build_template(tenant_id, template_id)
            .await
    }

    // -- builds --------------------------------------------------------------------

    pub async fn create_build(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateBuildRequest,
    ) -> DeployServiceResult<BuildResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.idempotency_key.trim().is_empty() {
            return Err(DeployServiceError::validation("idempotencyKey is required"));
        }
        if let Some(version) = request.semantic_version.as_deref() {
            SemanticVersion::parse(version).map_err(|error| {
                DeployServiceError::validation(format!("semanticVersion: {error}"))
            })?;
        }
        let build = self
            .repository
            .create_build(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "build.create", &build.id)
            .await?;
        Ok(build)
    }

    pub async fn list_builds(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<BuildPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_builds(tenant_id, app_id, page, page_size)
            .await
    }

    pub async fn retrieve_build(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        build_id: &str,
    ) -> DeployServiceResult<BuildResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_build(tenant_id, app_id, build_id)
            .await
    }

    /// Runner-reported state transition (typed executor contract).
    pub async fn update_build_state(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        build_id: &str,
        request: &UpdateBuildStateRequest,
    ) -> DeployServiceResult<BuildResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.runner_node_uuid.trim().is_empty() {
            return Err(DeployServiceError::validation("runnerNodeUuid is required"));
        }
        let build = self
            .repository
            .update_build_state(tenant_id, app_id, build_id, request)
            .await?;
        self.audit_app_action(context, "build.stateUpdate", &build.id)
            .await?;
        Ok(build)
    }

    // -- packages --------------------------------------------------------------------

    pub async fn register_package(
        &self,
        context: &DeployAppRequestContext,
        request: &RegisterPackageRequest,
    ) -> DeployServiceResult<PackageResponse> {
        let tenant_id = Self::tenant_id(context)?;
        validate_sha256(&request.checksum_sha256, "checksumSha256")?;
        validate_sha256(&request.manifest_sha256, "manifestSha256")?;
        SemanticVersion::parse(&request.semantic_version)
            .map_err(|error| DeployServiceError::validation(format!("semanticVersion: {error}")))?;

        // Platform/format compatibility and size ceilings are enforced here
        // and again by the package validator on the byte boundary.
        let (app_uuid, target_uuid, platform) = self
            .repository
            .resolve_build_platform(tenant_id, &request.build_id)
            .await?;
        if target_uuid != request.platform_target_id {
            return Err(DeployServiceError::validation(
                "platform target does not match the build platform target",
            ));
        }
        validate_package_format_for_platform(&platform, request.package_format.as_str())
            .map_err(DeployServiceError::validation)?;
        validate_package_size(
            &platform,
            request.package_format.as_str(),
            request.package_size_bytes as u64,
        )
        .map_err(DeployServiceError::validation)?;

        let package = self
            .repository
            .register_package(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "package.register", &package.id)
            .await?;
        let _ = app_uuid;
        Ok(package)
    }

    pub async fn list_packages(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<PackagePage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_packages(tenant_id, app_id, page, page_size)
            .await
    }

    pub async fn retrieve_package(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        package_id: &str,
    ) -> DeployServiceResult<PackageResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_package(tenant_id, app_id, package_id)
            .await
    }

    // -- releases ----------------------------------------------------------------------

    pub async fn create_app_release(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppReleaseRequest,
    ) -> DeployServiceResult<AppReleaseResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.idempotency_key.trim().is_empty() {
            return Err(DeployServiceError::validation("idempotencyKey is required"));
        }
        SemanticVersion::parse(&request.semantic_version)
            .map_err(|error| DeployServiceError::validation(format!("semanticVersion: {error}")))?;
        let release = self
            .repository
            .create_app_release(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "release.create", &release.id)
            .await?;
        Ok(release)
    }

    pub async fn list_app_releases(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<AppReleasePage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_app_releases(tenant_id, app_id, page, page_size)
            .await
    }

    pub async fn retrieve_app_release(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        release_id: &str,
    ) -> DeployServiceResult<AppReleaseResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_app_release(tenant_id, app_id, release_id)
            .await
    }

    pub async fn update_app_release_status(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        release_id: &str,
        release_status: ReleaseStatus,
    ) -> DeployServiceResult<AppReleaseResponse> {
        let tenant_id = Self::tenant_id(context)?;
        let release = self
            .repository
            .update_app_release_status(tenant_id, app_id, release_id, release_status)
            .await?;
        self.audit_app_action(context, "release.statusUpdate", &release.id)
            .await?;
        Ok(release)
    }

    // -- channels -----------------------------------------------------------------------

    pub async fn list_channels(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
    ) -> DeployServiceResult<ChannelPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository.list_channels(tenant_id, app_id).await
    }

    pub async fn retrieve_channel(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
    ) -> DeployServiceResult<ChannelResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_channel(tenant_id, app_id, channel_id)
            .await
    }

    pub async fn promote_channel(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
        request: &PromoteChannelRequest,
    ) -> DeployServiceResult<ChannelRolloutResponse> {
        let tenant_id = Self::tenant_id(context)?;
        let rollout = self
            .repository
            .promote_channel(tenant_id, app_id, channel_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "channel.promote", &rollout.id)
            .await?;
        Ok(rollout)
    }

    pub async fn list_channel_rollouts(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        channel_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<ChannelRolloutPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_channel_rollouts(tenant_id, app_id, channel_id, page, page_size)
            .await
    }

    // -- deployments ----------------------------------------------------------------------

    pub async fn create_app_deployment(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateAppDeploymentRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AppDeploymentResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.idempotency_key.trim().is_empty() {
            return Err(DeployServiceError::validation("idempotencyKey is required"));
        }
        validate_deployment_pair(&request)?;
        let deployment = self
            .repository
            .create_app_deployment(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "deployment.create", &deployment.id)
            .await?;
        Ok(deployment)
    }

    pub async fn list_app_deployments(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AppDeploymentPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_app_deployments(tenant_id, app_id, page, page_size)
            .await
    }

    pub async fn retrieve_app_deployment(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        deployment_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AppDeploymentResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_app_deployment(tenant_id, app_id, deployment_id)
            .await
    }

    pub async fn update_app_deployment_state(
        &self,
        context: &DeployAppRequestContext,
        app_id: &str,
        deployment_id: &str,
        deployment_status: DeploymentStatus,
        platform_review_ref: Option<&str>,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AppDeploymentResponse> {
        let tenant_id = Self::tenant_id(context)?;
        let deployment = self
            .repository
            .update_app_deployment_state(
                tenant_id,
                app_id,
                deployment_id,
                deployment_status,
                platform_review_ref,
            )
            .await?;
        self.audit_app_action(context, "deployment.stateUpdate", &deployment.id)
            .await?;
        Ok(deployment)
    }

    // -- signing identities -----------------------------------------------------------------

    pub async fn create_signing_identity(
        &self,
        context: &DeployAppRequestContext,
        request: &CreateSigningIdentityRequest,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        let tenant_id = Self::tenant_id(context)?;
        if request.identity_name.trim().is_empty() || request.identity_name.len() > 200 {
            return Err(DeployServiceError::validation(
                "identityName must be 1..=200 characters",
            ));
        }
        let identity = self
            .repository
            .create_signing_identity(tenant_id, context.actor_id, request)
            .await?;
        self.audit_app_action(context, "signingIdentity.create", &identity.id)
            .await?;
        Ok(identity)
    }

    pub async fn list_signing_identities(
        &self,
        context: &DeployAppRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<SigningIdentityPage> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .list_signing_identities(tenant_id, page, page_size)
            .await
    }

    pub async fn retrieve_signing_identity(
        &self,
        context: &DeployAppRequestContext,
        identity_id: &str,
    ) -> DeployServiceResult<SigningIdentityResponse> {
        let tenant_id = Self::tenant_id(context)?;
        self.repository
            .retrieve_signing_identity(tenant_id, identity_id)
            .await
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn is_bounded_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 120
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_channel_keys(channels: &[String]) -> DeployServiceResult<()> {
    if channels.is_empty() || channels.len() > 8 {
        return Err(DeployServiceError::validation(
            "allowedChannels must contain 1..=8 channel keys",
        ));
    }
    for channel in channels {
        if ChannelKey::parse(channel).is_none() {
            return Err(DeployServiceError::validation(format!(
                "channel key {channel} must be one of {}",
                CHANNEL_KEYS.join(", ")
            )));
        }
    }
    Ok(())
}

fn validate_repo_url(url: &str) -> DeployServiceResult<()> {
    if url.is_empty() || url.len() > 1000 {
        return Err(DeployServiceError::validation(
            "repoUrl must be 1..=1000 characters",
        ));
    }
    if url.contains("://") {
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(DeployServiceError::validation(
                "repoUrl must use https:// or http://",
            ));
        }
        if let Some(credentials_end) = url.find("://") {
            let scheme = &url[..credentials_end + 3];
            let rest = &url[credentials_end + 3..];
            if rest.contains('@') && !rest.starts_with("git@") {
                return Err(DeployServiceError::validation(
                    "repoUrl must not embed credentials",
                ));
            }
            let _ = scheme;
        }
    } else if !url.starts_with("git@") {
        return Err(DeployServiceError::validation(
            "repoUrl must be an https://, http://, or git@ scp-style URL",
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, field: &str) -> DeployServiceResult<()> {
    validate_sha256_hex(value, field).map_err(DeployServiceError::validation)
}

fn validate_deployment_pair(request: &CreateAppDeploymentRequest) -> DeployServiceResult<()> {
    let compatible = matches!(
        (
            request.deployment_kind.as_str(),
            request.deployment_target.as_str()
        ),
        ("MINIPROGRAM_REVIEW", "WECHAT_REVIEW" | "DOUYIN_REVIEW")
            | (
                "STORE_SUBMISSION",
                "APP_STORE_CONNECT" | "TESTFLIGHT" | "HARMONYOS_STORE"
            )
            | ("OTA_DISTRIBUTION", "OTA")
            | ("ENTERPRISE_DISTRIBUTION", "ENTERPRISE")
            | ("CONTAINER_ROLLOUT", "CONTAINER")
            | ("ARTIFACT_RELEASE", "WEB_NODE")
            | ("SITE_CONFIG", "WEB_NODE")
            | ("TLS_CONFIG", "WEB_NODE")
    );
    if !compatible {
        return Err(DeployServiceError::validation(format!(
            "deployment kind {} is not compatible with target {}",
            request.deployment_kind.as_str(),
            request.deployment_target.as_str()
        )));
    }
    Ok(())
}
