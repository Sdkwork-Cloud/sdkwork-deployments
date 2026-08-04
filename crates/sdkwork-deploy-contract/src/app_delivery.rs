//! Unified application delivery DTOs: apps, platform targets, source
//! repositories, build templates, builds, packages, releases, channels,
//! rollouts, deployments, and signing identities (REQ-2026-0002).

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Enums (canonical string vocabulary; no ad hoc integer meanings)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppKind {
    StaticWeb,
    SpaWeb,
    ApiService,
    WechatMiniprogram,
    DouyinMiniprogram,
    IosApp,
    AndroidApp,
    HarmonyosApp,
}

impl AppKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaticWeb => "STATIC_WEB",
            Self::SpaWeb => "SPA_WEB",
            Self::ApiService => "API_SERVICE",
            Self::WechatMiniprogram => "WECHAT_MINIPROGRAM",
            Self::DouyinMiniprogram => "DOUYIN_MINIPROGRAM",
            Self::IosApp => "IOS_APP",
            Self::AndroidApp => "ANDROID_APP",
            Self::HarmonyosApp => "HARMONYOS_APP",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "STATIC_WEB" => Some(Self::StaticWeb),
            "SPA_WEB" => Some(Self::SpaWeb),
            "API_SERVICE" => Some(Self::ApiService),
            "WECHAT_MINIPROGRAM" => Some(Self::WechatMiniprogram),
            "DOUYIN_MINIPROGRAM" => Some(Self::DouyinMiniprogram),
            "IOS_APP" => Some(Self::IosApp),
            "ANDROID_APP" => Some(Self::AndroidApp),
            "HARMONYOS_APP" => Some(Self::HarmonyosApp),
            _ => None,
        }
    }

    pub fn is_web(self) -> bool {
        matches!(self, Self::StaticWeb | Self::SpaWeb)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Web,
    Api,
    Wechat,
    Douyin,
    Ios,
    Android,
    Harmonyos,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "WEB",
            Self::Api => "API",
            Self::Wechat => "WECHAT",
            Self::Douyin => "DOUYIN",
            Self::Ios => "IOS",
            Self::Android => "ANDROID",
            Self::Harmonyos => "HARMONYOS",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TechStack {
    Flutter,
    Native,
    UniApp,
    Node,
    Rust,
    Go,
    Java,
    Other,
}

impl TechStack {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flutter => "FLUTTER",
            Self::Native => "NATIVE",
            Self::UniApp => "UNI_APP",
            Self::Node => "NODE",
            Self::Rust => "RUST",
            Self::Go => "GO",
            Self::Java => "JAVA",
            Self::Other => "OTHER",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BuildStatus {
    Queued,
    Preparing,
    Compiling,
    Testing,
    Packaging,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
}

impl BuildStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "QUEUED",
            Self::Preparing => "PREPARING",
            Self::Compiling => "COMPILING",
            Self::Testing => "TESTING",
            Self::Packaging => "PACKAGING",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }

    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackageFormat {
    DistDir,
    Zip,
    Apk,
    Aab,
    Ipa,
    Xcarchive,
    Hap,
    App,
    OciImage,
    ProcessBundle,
    TarGz,
}

impl PackageFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DistDir => "DIST_DIR",
            Self::Zip => "ZIP",
            Self::Apk => "APK",
            Self::Aab => "AAB",
            Self::Ipa => "IPA",
            Self::Xcarchive => "XCARCHIVE",
            Self::Hap => "HAP",
            Self::App => "APP",
            Self::OciImage => "OCI_IMAGE",
            Self::ProcessBundle => "PROCESS_BUNDLE",
            Self::TarGz => "TAR_GZ",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "DIST_DIR" => Some(Self::DistDir),
            "ZIP" => Some(Self::Zip),
            "APK" => Some(Self::Apk),
            "AAB" => Some(Self::Aab),
            "IPA" => Some(Self::Ipa),
            "XCARCHIVE" => Some(Self::Xcarchive),
            "HAP" => Some(Self::Hap),
            "APP" => Some(Self::App),
            "OCI_IMAGE" => Some(Self::OciImage),
            "PROCESS_BUNDLE" => Some(Self::ProcessBundle),
            "TAR_GZ" => Some(Self::TarGz),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackageStatus {
    Draft,
    Validated,
    Ready,
    Superseded,
    Retired,
    Archived,
}

impl PackageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Validated => "VALIDATED",
            Self::Ready => "READY",
            Self::Superseded => "SUPERSEDED",
            Self::Retired => "RETIRED",
            Self::Archived => "ARCHIVED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseStatus {
    Draft,
    Active,
    Superseded,
    Deprecated,
    Retired,
    Archived,
}

impl ReleaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Active => "ACTIVE",
            Self::Superseded => "SUPERSEDED",
            Self::Deprecated => "DEPRECATED",
            Self::Retired => "RETIRED",
            Self::Archived => "ARCHIVED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelKey {
    Stable,
    Beta,
    Alpha,
    Qa,
}

impl ChannelKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Alpha => "alpha",
            Self::Qa => "qa",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "stable" => Some(Self::Stable),
            "beta" => Some(Self::Beta),
            "alpha" => Some(Self::Alpha),
            "qa" => Some(Self::Qa),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RolloutStrategy {
    Immediate,
    Percentage,
    ManualApproval,
}

impl RolloutStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Immediate => "IMMEDIATE",
            Self::Percentage => "PERCENTAGE",
            Self::ManualApproval => "MANUAL_APPROVAL",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RolloutStatus {
    Pending,
    Rolling,
    Completed,
    RolledBack,
    Failed,
    Cancelled,
}

impl RolloutStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Rolling => "ROLLING",
            Self::Completed => "COMPLETED",
            Self::RolledBack => "ROLLED_BACK",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeploymentKind {
    ArtifactRelease,
    SiteConfig,
    TlsConfig,
    MiniprogramReview,
    StoreSubmission,
    OtaDistribution,
    EnterpriseDistribution,
    ContainerRollout,
}

impl DeploymentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArtifactRelease => "ARTIFACT_RELEASE",
            Self::SiteConfig => "SITE_CONFIG",
            Self::TlsConfig => "TLS_CONFIG",
            Self::MiniprogramReview => "MINIPROGRAM_REVIEW",
            Self::StoreSubmission => "STORE_SUBMISSION",
            Self::OtaDistribution => "OTA_DISTRIBUTION",
            Self::EnterpriseDistribution => "ENTERPRISE_DISTRIBUTION",
            Self::ContainerRollout => "CONTAINER_ROLLOUT",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeploymentTarget {
    WebNode,
    Container,
    WechatReview,
    DouyinReview,
    AppStoreConnect,
    Testflight,
    Ota,
    Enterprise,
    HarmonyosStore,
}

impl DeploymentTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebNode => "WEB_NODE",
            Self::Container => "CONTAINER",
            Self::WechatReview => "WECHAT_REVIEW",
            Self::DouyinReview => "DOUYIN_REVIEW",
            Self::AppStoreConnect => "APP_STORE_CONNECT",
            Self::Testflight => "TESTFLIGHT",
            Self::Ota => "OTA",
            Self::Enterprise => "ENTERPRISE",
            Self::HarmonyosStore => "HARMONYOS_STORE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeploymentStatus {
    Pending,
    Submitting,
    PendingReview,
    InReview,
    Rejected,
    Approved,
    Live,
    Active,
    Degraded,
    Failed,
    RolledBack,
    Cancelled,
}

impl DeploymentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Submitting => "SUBMITTING",
            Self::PendingReview => "PENDING_REVIEW",
            Self::InReview => "IN_REVIEW",
            Self::Rejected => "REJECTED",
            Self::Approved => "APPROVED",
            Self::Live => "LIVE",
            Self::Active => "ACTIVE",
            Self::Degraded => "DEGRADED",
            Self::Failed => "FAILED",
            Self::RolledBack => "ROLLED_BACK",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SigningKind {
    IosSigning,
    AndroidKeystore,
    HarmonyosCertProfile,
    MiniprogramUploadKey,
    ApiRepoToken,
}

impl SigningKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IosSigning => "IOS_SIGNING",
            Self::AndroidKeystore => "ANDROID_KEYSTORE",
            Self::HarmonyosCertProfile => "HARMONYOS_CERT_PROFILE",
            Self::MiniprogramUploadKey => "MINIPROGRAM_UPLOAD_KEY",
            Self::ApiRepoToken => "API_REPO_TOKEN",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppStatus {
    Draft,
    Ready,
    Active,
    Paused,
    Archived,
    Failed,
}

impl AppStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Ready => "READY",
            Self::Active => "ACTIVE",
            Self::Paused => "PAUSED",
            Self::Archived => "ARCHIVED",
            Self::Failed => "FAILED",
        }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAppRequest {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "appKind")]
    pub app_kind: AppKind,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "defaultEnvironment", default)]
    pub default_environment: Option<String>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateAppRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "appStatus", default)]
    pub app_status: Option<AppStatus>,
    #[serde(rename = "defaultEnvironment", default)]
    pub default_environment: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "appKind")]
    pub app_kind: String,
    #[serde(rename = "appStatus")]
    pub app_status: String,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "siteId", skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(rename = "defaultEnvironment")]
    pub default_environment: String,
    #[serde(rename = "platformTargetCount")]
    pub platform_target_count: i64,
    #[serde(rename = "latestReleaseTag", skip_serializing_if = "Option::is_none")]
    pub latest_release_tag: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppPage {
    pub items: Vec<AppResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Platform target
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePlatformTargetRequest {
    #[serde(rename = "targetKey")]
    pub target_key: String,
    pub platform: Platform,
    #[serde(rename = "techStack", default)]
    pub tech_stack: Option<TechStack>,
    #[serde(rename = "bundleId", default)]
    pub bundle_id: Option<String>,
    #[serde(rename = "packageName", default)]
    pub package_name: Option<String>,
    #[serde(rename = "appId", default)]
    pub app_id: Option<String>,
    #[serde(rename = "bundleName", default)]
    pub bundle_name: Option<String>,
    #[serde(rename = "buildTemplateId", default)]
    pub build_template_id: Option<String>,
    #[serde(rename = "allowedChannels", default)]
    pub allowed_channels: Option<Vec<String>>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlatformTargetResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "targetKey")]
    pub target_key: String,
    pub platform: String,
    #[serde(rename = "techStack")]
    pub tech_stack: String,
    #[serde(rename = "bundleId", skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(rename = "packageName", skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(rename = "appIdValue", skip_serializing_if = "Option::is_none")]
    pub app_id_value: Option<String>,
    #[serde(rename = "bundleName", skip_serializing_if = "Option::is_none")]
    pub bundle_name: Option<String>,
    #[serde(rename = "buildTemplateId", skip_serializing_if = "Option::is_none")]
    pub build_template_id: Option<String>,
    #[serde(rename = "allowedChannels")]
    pub allowed_channels: Vec<String>,
    #[serde(rename = "targetStatus")]
    pub target_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlatformTargetPage {
    pub items: Vec<PlatformTargetResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Source repository
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSourceRepositoryRequest {
    #[serde(rename = "repoKey")]
    pub repo_key: String,
    #[serde(rename = "repoProvider")]
    pub repo_provider: String,
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    #[serde(rename = "defaultBranch", default)]
    pub default_branch: Option<String>,
    #[serde(rename = "cloneMode", default)]
    pub clone_mode: Option<String>,
    #[serde(rename = "credentialSecretRef", default)]
    pub credential_secret_ref: Option<String>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceRepositoryResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "repoKey")]
    pub repo_key: String,
    #[serde(rename = "repoProvider")]
    pub repo_provider: String,
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    #[serde(rename = "defaultBranch")]
    pub default_branch: String,
    #[serde(rename = "cloneMode")]
    pub clone_mode: String,
    #[serde(
        rename = "credentialSecretRef",
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_secret_ref: Option<String>,
    #[serde(rename = "repoStatus")]
    pub repo_status: String,
    #[serde(rename = "lastErrorCode", skip_serializing_if = "Option::is_none")]
    pub last_error_code: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceRepositoryPage {
    pub items: Vec<SourceRepositoryResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Build template
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBuildTemplateRequest {
    #[serde(rename = "templateName")]
    pub template_name: String,
    #[serde(rename = "templateVersion")]
    pub template_version: String,
    pub platform: Platform,
    #[serde(rename = "techStack", default)]
    pub tech_stack: Option<TechStack>,
    #[serde(rename = "toolchain", default)]
    pub toolchain: Option<Value>,
    #[serde(rename = "commands", default)]
    pub commands: Option<Vec<String>>,
    #[serde(rename = "artifactOutputs", default)]
    pub artifact_outputs: Option<Vec<String>>,
    #[serde(rename = "qualityGates", default)]
    pub quality_gates: Option<Value>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildTemplateResponse {
    pub id: String,
    #[serde(rename = "templateName")]
    pub template_name: String,
    #[serde(rename = "templateVersion")]
    pub template_version: String,
    pub platform: String,
    #[serde(rename = "techStack")]
    pub tech_stack: String,
    #[serde(rename = "toolchain", skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<Value>,
    #[serde(rename = "commands", skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(rename = "artifactOutputs", skip_serializing_if = "Option::is_none")]
    pub artifact_outputs: Option<Vec<String>>,
    #[serde(rename = "qualityGates", skip_serializing_if = "Option::is_none")]
    pub quality_gates: Option<Value>,
    #[serde(rename = "templateStatus")]
    pub template_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildTemplatePage {
    pub items: Vec<BuildTemplateResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Build
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBuildRequest {
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "sourceRepositoryId", default)]
    pub source_repository_id: Option<String>,
    #[serde(rename = "sourceRef", default)]
    pub source_ref: Option<String>,
    #[serde(rename = "templateId", default)]
    pub template_id: Option<String>,
    #[serde(rename = "semanticVersion", default)]
    pub semantic_version: Option<String>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "templateId", skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(rename = "buildNumber")]
    pub build_number: i64,
    #[serde(rename = "sourceRepositoryId", skip_serializing_if = "Option::is_none")]
    pub source_repository_id: Option<String>,
    #[serde(rename = "sourceRef", skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(rename = "sourceSnapshot", skip_serializing_if = "Option::is_none")]
    pub source_snapshot: Option<Value>,
    #[serde(rename = "buildStatus")]
    pub build_status: String,
    #[serde(rename = "logRef", skip_serializing_if = "Option::is_none")]
    pub log_ref: Option<String>,
    #[serde(rename = "producedPackageId", skip_serializing_if = "Option::is_none")]
    pub produced_package_id: Option<String>,
    #[serde(rename = "qualityGate", skip_serializing_if = "Option::is_none")]
    pub quality_gate: Option<Value>,
    #[serde(rename = "runnerNodeUuid", skip_serializing_if = "Option::is_none")]
    pub runner_node_uuid: Option<String>,
    #[serde(rename = "errorCode", skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(rename = "startedAt", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(rename = "finishedAt", skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BuildPage {
    pub items: Vec<BuildResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Runner-reported build state transition (typed executor contract).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateBuildStateRequest {
    #[serde(rename = "buildStatus")]
    pub build_status: BuildStatus,
    #[serde(rename = "runnerNodeUuid")]
    pub runner_node_uuid: String,
    #[serde(rename = "runnerVersion", default)]
    pub runner_version: Option<String>,
    #[serde(rename = "logRef", default)]
    pub log_ref: Option<String>,
    #[serde(rename = "sourceSnapshot", default)]
    pub source_snapshot: Option<Value>,
    #[serde(rename = "qualityGate", default)]
    pub quality_gate: Option<Value>,
    #[serde(rename = "errorCode", default)]
    pub error_code: Option<String>,
    #[serde(rename = "startedAt", default)]
    pub started_at: Option<String>,
    #[serde(rename = "finishedAt", default)]
    pub finished_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Package
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterPackageRequest {
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "buildId")]
    pub build_id: String,
    #[serde(rename = "packageFormat")]
    pub package_format: PackageFormat,
    #[serde(rename = "semanticVersion")]
    pub semantic_version: String,
    #[serde(rename = "packageSizeBytes")]
    pub package_size_bytes: i64,
    #[serde(rename = "checksumSha256")]
    pub checksum_sha256: String,
    #[serde(rename = "manifestSha256")]
    pub manifest_sha256: String,
    #[serde(rename = "driveNodeId")]
    pub drive_node_id: String,
    #[serde(rename = "driveSpaceId", default)]
    pub drive_space_id: Option<String>,
    #[serde(rename = "signingIdentityId", default)]
    pub signing_identity_id: Option<String>,
    #[serde(rename = "minPlatformVersion", default)]
    pub min_platform_version: Option<String>,
    #[serde(rename = "architectures", default)]
    pub architectures: Option<Vec<String>>,
    #[serde(rename = "bundleIdentity", default)]
    pub bundle_identity: Option<Value>,
    #[serde(rename = "validationReport", default)]
    pub validation_report: Option<Value>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackageResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "buildId")]
    pub build_id: String,
    #[serde(rename = "packageFormat")]
    pub package_format: String,
    #[serde(rename = "semanticVersion")]
    pub semantic_version: String,
    #[serde(rename = "packageSizeBytes")]
    pub package_size_bytes: i64,
    #[serde(rename = "checksumSha256")]
    pub checksum_sha256: String,
    #[serde(rename = "manifestSha256")]
    pub manifest_sha256: String,
    #[serde(rename = "driveNodeId", skip_serializing_if = "Option::is_none")]
    pub drive_node_id: Option<String>,
    #[serde(rename = "signingIdentityId", skip_serializing_if = "Option::is_none")]
    pub signing_identity_id: Option<String>,
    #[serde(rename = "minPlatformVersion", skip_serializing_if = "Option::is_none")]
    pub min_platform_version: Option<String>,
    #[serde(rename = "architectures", skip_serializing_if = "Option::is_none")]
    pub architectures: Option<Vec<String>>,
    #[serde(rename = "packageStatus")]
    pub package_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackagePage {
    pub items: Vec<PackageResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Release
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAppReleaseRequest {
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "packageId")]
    pub package_id: String,
    #[serde(rename = "semanticVersion")]
    pub semantic_version: String,
    #[serde(rename = "releaseNotes", default)]
    pub release_notes: Option<Value>,
    #[serde(rename = "releaseStatus", default)]
    pub release_status: Option<ReleaseStatus>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppReleaseResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "packageId")]
    pub package_id: String,
    #[serde(rename = "semanticVersion")]
    pub semantic_version: String,
    #[serde(rename = "buildNumber")]
    pub build_number: i64,
    #[serde(rename = "releaseStatus")]
    pub release_status: String,
    #[serde(rename = "releaseNotes", skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<Value>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppReleasePage {
    pub items: Vec<AppReleaseResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Channel and rollout
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromoteChannelRequest {
    #[serde(rename = "releaseId")]
    pub release_id: String,
    #[serde(rename = "strategy", default)]
    pub strategy: Option<RolloutStrategy>,
    #[serde(default)]
    pub percentage: Option<u32>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "channelKey")]
    pub channel_key: String,
    #[serde(rename = "currentReleaseId", skip_serializing_if = "Option::is_none")]
    pub current_release_id: Option<String>,
    #[serde(
        rename = "currentReleaseVersion",
        skip_serializing_if = "Option::is_none"
    )]
    pub current_release_version: Option<String>,
    #[serde(rename = "channelStatus")]
    pub channel_status: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelPage {
    pub items: Vec<ChannelResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelRolloutResponse {
    pub id: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "releaseId")]
    pub release_id: String,
    #[serde(rename = "releaseVersion")]
    pub release_version: String,
    pub strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<u32>,
    #[serde(rename = "rolloutStatus")]
    pub rollout_status: String,
    #[serde(
        rename = "supersedesRolloutId",
        skip_serializing_if = "Option::is_none"
    )]
    pub supersedes_rollout_id: Option<String>,
    #[serde(rename = "requestedAt")]
    pub requested_at: String,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelRolloutPage {
    pub items: Vec<ChannelRolloutResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Deployment
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAppDeploymentRequest {
    #[serde(rename = "platformTargetId")]
    pub platform_target_id: String,
    #[serde(rename = "releaseId")]
    pub release_id: String,
    #[serde(rename = "deploymentKind")]
    pub deployment_kind: DeploymentKind,
    #[serde(rename = "deploymentTarget")]
    pub deployment_target: DeploymentTarget,
    #[serde(rename = "environment", default)]
    pub environment: Option<String>,
    #[serde(rename = "strategy", default)]
    pub strategy: Option<RolloutStrategy>,
    #[serde(default)]
    pub percentage: Option<u32>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppDeploymentResponse {
    pub id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "platformTargetId", skip_serializing_if = "Option::is_none")]
    pub platform_target_id: Option<String>,
    #[serde(rename = "siteId", skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(rename = "releaseId", skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(rename = "deploymentKind", skip_serializing_if = "Option::is_none")]
    pub deployment_kind: Option<String>,
    #[serde(rename = "deploymentTarget", skip_serializing_if = "Option::is_none")]
    pub deployment_target: Option<String>,
    #[serde(rename = "environment")]
    pub environment: String,
    #[serde(rename = "strategy", skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<u32>,
    #[serde(rename = "platformReviewRef", skip_serializing_if = "Option::is_none")]
    pub platform_review_ref: Option<String>,
    #[serde(rename = "deploymentStatus")]
    pub deployment_status: String,
    #[serde(
        rename = "rollbackFromDeploymentId",
        skip_serializing_if = "Option::is_none"
    )]
    pub rollback_from_deployment_id: Option<String>,
    #[serde(rename = "startedAt", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppDeploymentPage {
    pub items: Vec<AppDeploymentResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Signing identity
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSigningIdentityRequest {
    #[serde(rename = "identityName")]
    pub identity_name: String,
    #[serde(rename = "signingKind")]
    pub signing_kind: SigningKind,
    #[serde(rename = "platformTargetId", default)]
    pub platform_target_id: Option<String>,
    #[serde(rename = "fingerprintSha256", default)]
    pub fingerprint_sha256: Option<String>,
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<String>,
    #[serde(rename = "secretRef", default)]
    pub secret_ref: Option<String>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SigningIdentityResponse {
    pub id: String,
    #[serde(rename = "identityName")]
    pub identity_name: String,
    #[serde(rename = "signingKind")]
    pub signing_kind: String,
    #[serde(rename = "platformTargetId", skip_serializing_if = "Option::is_none")]
    pub platform_target_id: Option<String>,
    #[serde(rename = "fingerprintSha256", skip_serializing_if = "Option::is_none")]
    pub fingerprint_sha256: Option<String>,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(rename = "secretRef", skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<String>,
    #[serde(rename = "identityStatus")]
    pub identity_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SigningIdentityPage {
    pub items: Vec<SigningIdentityResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}
