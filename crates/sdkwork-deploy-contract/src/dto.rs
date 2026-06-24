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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SitePage {
    pub items: Vec<SiteResponse>,
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
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
pub struct DomainResponse {
    pub id: String,
    pub hostname: String,
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "sslEnabled")]
    pub ssl_enabled: bool,
    #[serde(rename = "sslProvider", skip_serializing_if = "Option::is_none")]
    pub ssl_provider: Option<String>,
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DomainPage {
    pub items: Vec<DomainResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateDomainRequest {
    pub hostname: String,
    #[serde(rename = "isPrimary", default)]
    pub is_primary: bool,
    #[serde(rename = "sslEnabled", default = "default_true")]
    pub ssl_enabled: bool,
    #[serde(rename = "sslProvider", default)]
    pub ssl_provider: Option<String>,
}

fn default_true() -> bool {
    true
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
    #[serde(rename = "verifyToken", skip_serializing_if = "Option::is_none")]
    pub verify_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub id: String,
    #[serde(rename = "siteId")]
    pub site_id: String,
    pub status: i32,
    #[serde(rename = "deployType")]
    pub deploy_type: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeploymentPage {
    pub items: Vec<DeploymentResponse>,
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    #[serde(rename = "deployType", default = "default_deploy_type")]
    pub deploy_type: i32,
    #[serde(default)]
    pub environment: Option<String>,
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
    pub status: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CertificatePage {
    pub items: Vec<CertificateResponse>,
    pub total: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCertificateRequest {
    #[serde(rename = "certName")]
    pub cert_name: String,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "domainId", default)]
    pub domain_id: Option<String>,
}

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
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ListNginxConfigsQuery {
    #[serde(default = "crate::dto::default_page")]
    pub page: i32,
    #[serde(default = "crate::dto::default_page_size", rename = "pageSize")]
    pub page_size: i32,
    #[serde(rename = "siteId", default)]
    pub site_id: Option<String>,
    #[serde(rename = "configType", default)]
    pub config_type: Option<i32>,
    #[serde(rename = "isActive", default)]
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
    pub status: i32,
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
}

fn default_ssh_port() -> i32 {
    22
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub action: String,
    pub resource: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLogPage {
    pub items: Vec<AuditLogResponse>,
    pub total: i64,
    pub page: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
}
