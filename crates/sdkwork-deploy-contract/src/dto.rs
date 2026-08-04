use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SiteResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: i32,
    pub status: i32,
    #[serde(rename = "runtimeConfig", skip_serializing_if = "Option::is_none")]
    pub runtime_config: Option<Value>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SitePage {
    pub items: Vec<SiteResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "siteType")]
    pub site_type: i32,
    #[serde(rename = "runtimeConfig", default)]
    pub runtime_config: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateSiteRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "runtimeConfig", default)]
    pub runtime_config: Option<Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainZoneResponse {
    pub id: String,
    #[serde(rename = "apexHostname")]
    pub apex_hostname: String,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "dnsProvider", skip_serializing_if = "Option::is_none")]
    pub dns_provider: Option<String>,
    pub status: String,
    #[serde(rename = "hostnameCount")]
    pub hostname_count: i64,
    #[serde(rename = "verifiedHostnameCount")]
    pub verified_hostname_count: i64,
    #[serde(rename = "certificateCount")]
    pub certificate_count: i64,
    #[serde(rename = "bindingCount")]
    pub binding_count: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainZonePage {
    pub items: Vec<DomainZoneResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDomainZoneRequest {
    #[serde(rename = "apexHostname")]
    pub apex_hostname: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "dnsProvider", default)]
    pub dns_provider: Option<String>,
    #[serde(rename = "providerZoneRef", default)]
    pub provider_zone_ref: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateDomainZoneRequest {
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "dnsProvider", default)]
    pub dns_provider: Option<String>,
    #[serde(rename = "providerZoneRef", default)]
    pub provider_zone_ref: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainHostnameResponse {
    pub id: String,
    #[serde(rename = "zoneId")]
    pub zone_id: String,
    pub hostname: String,
    #[serde(rename = "relativeName")]
    pub relative_name: String,
    #[serde(rename = "hostnameType")]
    pub hostname_type: String,
    #[serde(rename = "verificationStatus")]
    pub verification_status: String,
    #[serde(rename = "verifiedAt", skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
    pub status: String,
    #[serde(rename = "certificateCount")]
    pub certificate_count: i64,
    #[serde(rename = "bindingCount")]
    pub binding_count: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainHostnamePage {
    pub items: Vec<DomainHostnameResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDomainHostnameRequest {
    #[serde(rename = "relativeName")]
    pub relative_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateDomainHostnameRequest {
    #[serde(rename = "relativeName")]
    pub relative_name: String,
}

pub(crate) fn default_page() -> i32 {
    1
}

pub(crate) fn default_page_size() -> i32 {
    20
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainVerifyResponse {
    pub verified: bool,
    pub method: String,
    #[serde(rename = "verificationId", skip_serializing_if = "Option::is_none")]
    pub verification_id: Option<String>,
    #[serde(rename = "recordName", skip_serializing_if = "Option::is_none")]
    pub record_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    pub status: i32,
    #[serde(rename = "deployType")]
    pub deploy_type: i32,
    #[serde(rename = "releaseId", skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentPage {
    pub items: Vec<DeploymentResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    /// Opaque keyset continuation for cursor mode;  in offset mode or on
    /// the last page (PAGINATION_SPEC §6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Exact page continuation flag for cursor mode;  in offset mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    #[serde(rename = "deployType", default = "default_deploy_type")]
    pub deploy_type: i32,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(rename = "releaseId", default)]
    pub release_id: Option<String>,
    #[serde(rename = "idempotencyKey", default)]
    pub idempotency_key: Option<String>,
}

fn default_deploy_type() -> i32 {
    1
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnvVariableResponse {
    pub id: String,
    pub key: String,
    pub value: String,
    pub environment: String,
    #[serde(rename = "isSecret")]
    pub is_secret: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EnvVariablePage {
    pub items: Vec<EnvVariableResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEnvVariableRequest {
    pub key: String,
    pub value: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(rename = "isSecret", default)]
    pub is_secret: bool,
}

fn default_environment() -> String {
    "production".to_string()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificateResponse {
    pub id: String,
    #[serde(rename = "certName")]
    pub cert_name: String,
    #[serde(rename = "certificateSource")]
    pub certificate_source: String,
    #[serde(rename = "caProfile")]
    pub ca_profile: String,
    #[serde(rename = "preferredKeyAlgorithm")]
    pub preferred_key_algorithm: String,
    pub identifiers: Vec<String>,
    #[serde(rename = "currentVersionId", skip_serializing_if = "Option::is_none")]
    pub current_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(rename = "notBefore", skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    #[serde(rename = "notAfter", skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,
    #[serde(rename = "autoRenew")]
    pub auto_renew: bool,
    #[serde(rename = "renewalStatus")]
    pub renewal_status: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificatePage {
    pub items: Vec<CertificateResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCertificateRequest {
    #[serde(rename = "certName")]
    pub cert_name: String,
    #[serde(rename = "domainIds")]
    pub domain_ids: Vec<String>,
    #[serde(rename = "caProfile", default = "default_certificate_ca_profile")]
    pub ca_profile: String,
    #[serde(
        rename = "preferredKeyAlgorithm",
        default = "default_certificate_key_algorithm"
    )]
    pub preferred_key_algorithm: String,
}

fn default_certificate_ca_profile() -> String {
    "LETS_ENCRYPT_PRODUCTION".to_owned()
}

fn default_certificate_key_algorithm() -> String {
    "ECDSA".to_owned()
}

pub fn is_deploy_package_artifact_type(package_type: i32) -> bool {
    (1..=5).contains(&package_type)
}

pub const ARTIFACT_STATUS_ACTIVE: i32 = 1;
pub const ARTIFACT_STATUS_RETAINED: i32 = 2;

pub const RELEASE_STATUS_ACTIVE: i32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateArtifactRequest {
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "packageType")]
    pub package_type: i32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentLength")]
    pub content_length: i64,
    #[serde(rename = "checksumSha256", default)]
    pub checksum_sha256: Option<String>,
    #[serde(rename = "driveUploadSessionId")]
    pub drive_upload_session_id: String,
    #[serde(rename = "driveUploadItemId", default)]
    pub drive_upload_item_id: Option<String>,
    #[serde(rename = "driveSpaceId")]
    pub drive_space_id: String,
    #[serde(rename = "driveNodeId")]
    pub drive_node_id: String,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactResponse {
    pub id: String,
    #[serde(rename = "siteId", skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(rename = "packageType")]
    pub package_type: i32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentLength")]
    pub content_length: i64,
    #[serde(rename = "checksumSha256", skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,
    #[serde(rename = "driveNodeId")]
    pub drive_node_id: String,
    #[serde(rename = "uploadSessionId")]
    pub upload_session_id: String,
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactPage {
    pub items: Vec<ArtifactResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReleaseResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    #[serde(rename = "versionTag", skip_serializing_if = "Option::is_none")]
    pub version_tag: Option<String>,
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReleasePage {
    pub items: Vec<ReleaseResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    #[serde(rename = "versionTag", default)]
    pub version_tag: Option<String>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

pub const UPLOAD_SESSION_STATUS_COMPLETED: i32 = 1;
pub const UPLOAD_SESSION_STATUS_CANCELLED: i32 = 2;

pub const CERTIFICATE_SOURCE_MANAGED: &str = "MANAGED";
pub const CERTIFICATE_STATUS_PENDING: &str = "PENDING";
pub const CERTIFICATE_STATUS_REVOKED: &str = "REVOKED";
pub const CERTIFICATE_RENEWAL_STATUS_NONE: &str = "NONE";
pub const CERTIFICATE_RENEWAL_STATUS_PLANNED: &str = "PLANNED";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub id: String,
    #[serde(rename = "checkType")]
    pub check_type: i32,
    pub url: String,
    pub status: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HealthCheckPage {
    pub items: Vec<HealthCheckResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateHealthCheckRequest {
    #[serde(rename = "checkType")]
    pub check_type: i32,
    pub url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NginxConfigResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    #[serde(rename = "configName")]
    pub config_name: String,
    #[serde(rename = "configType")]
    pub config_type: i32,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    pub status: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NginxConfigPage {
    pub items: Vec<NginxConfigResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListNginxConfigsQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    // PAGINATION_SPEC §3：query 参数使用 lower_snake_case 规范词汇。
    #[serde(default)]
    pub site_id: Option<String>,
    #[serde(default)]
    pub config_type: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateNginxConfigRequest {
    #[serde(rename = "siteId")]
    pub site_id: String,
    #[serde(rename = "configName")]
    pub config_name: String,
    #[serde(rename = "configType")]
    pub config_type: i32,
    #[serde(rename = "configContent")]
    pub config_content: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateNginxConfigRequest {
    #[serde(rename = "configName", default)]
    pub config_name: Option<String>,
    #[serde(rename = "configContent", default)]
    pub config_content: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxValidateResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxReloadResponse {
    pub reloaded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NginxStatusResponse {
    pub running: bool,
    #[serde(rename = "activeConfigs")]
    pub active_configs: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(rename = "sshPort")]
    pub ssh_port: i32,
    #[serde(rename = "clusterId", skip_serializing_if = "Option::is_none")]
    pub cluster_id: Option<String>,
    #[serde(rename = "clusterName", skip_serializing_if = "Option::is_none")]
    pub cluster_name: Option<String>,
    #[serde(rename = "nodeRole")]
    pub node_role: i32,
    pub status: i32,
    #[serde(rename = "sshUser", skip_serializing_if = "Option::is_none")]
    pub ssh_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerPage {
    pub items: Vec<ServerResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub host: String,
    #[serde(rename = "sshPort", default = "default_ssh_port")]
    pub ssh_port: i32,
    #[serde(rename = "clusterId", default)]
    pub cluster_id: Option<String>,
    #[serde(rename = "sshUser", default)]
    pub ssh_user: Option<String>,
    #[serde(rename = "sshKeyPath", default)]
    pub ssh_key_path: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateServerRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "sshPort", default)]
    pub ssh_port: Option<i32>,
    #[serde(rename = "clusterId", default)]
    pub cluster_id: Option<String>,
    #[serde(rename = "sshUser", default)]
    pub ssh_user: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
}

fn default_ssh_port() -> i32 {
    22
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeClusterResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub status: i32,
    #[serde(rename = "nodeCount")]
    pub node_count: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NodeClusterPage {
    pub items: Vec<NodeClusterResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateNodeClusterRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UpdateNodeClusterRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub action: String,
    pub resource: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// 审计日志列表过滤（PAGINATION_SPEC §4：声明即实现；wire 参数使用
/// lower_snake_case 规范词汇，禁止 camelCase 别名）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size")]
    pub page_size: i32,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub operator_id: Option<i64>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    /// Opaque keyset continuation token（PAGINATION_SPEC §6）；提供时走
    /// cursor 模式（过滤参数不适用于 cursor 模式）。
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogPage {
    pub items: Vec<AuditLogResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    /// Opaque keyset continuation for cursor mode;  in offset mode or on
    /// the last page (PAGINATION_SPEC §6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Exact page continuation flag for cursor mode;  in offset mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDeployUploadSessionRequest {
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "packageType")]
    pub package_type: i32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentLength")]
    pub content_length: i64,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeployUploadSessionResponse {
    pub id: String,
    #[serde(rename = "siteId", skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    #[serde(rename = "packageType")]
    pub package_type: i32,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentLength")]
    pub content_length: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    pub status: i32,
    #[serde(rename = "driveUploadSessionId")]
    pub drive_upload_session_id: String,
    #[serde(rename = "driveUploadItemId", skip_serializing_if = "Option::is_none")]
    pub drive_upload_item_id: Option<String>,
    #[serde(rename = "driveSpaceId", skip_serializing_if = "Option::is_none")]
    pub drive_space_id: Option<String>,
    #[serde(rename = "driveNodeId", skip_serializing_if = "Option::is_none")]
    pub drive_node_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletedUploadPartInput {
    #[serde(rename = "partNo")]
    pub part_no: i64,
    pub etag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteDeployUploadSessionRequest {
    #[serde(rename = "checksumSha256Hex")]
    pub checksum_sha256_hex: String,
    #[serde(rename = "contentLength", default)]
    pub content_length: Option<i64>,
    #[serde(rename = "contentType", default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub parts: Vec<CompletedUploadPartInput>,
}
