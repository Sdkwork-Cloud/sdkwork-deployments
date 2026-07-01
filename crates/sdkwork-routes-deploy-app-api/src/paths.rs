pub const PREFIX: &str = "/app/v3/api";

pub const SITES: &str = "/app/v3/api/sites";
pub const SITE: &str = "/app/v3/api/sites/{siteId}";
pub const SITE_ACTIVATE: &str = "/app/v3/api/sites/{siteId}/activate";
pub const SITE_PAUSE: &str = "/app/v3/api/sites/{siteId}/pause";
pub const SITE_DOMAINS: &str = "/app/v3/api/sites/{siteId}/domains";
pub const SITE_DOMAIN: &str = "/app/v3/api/sites/{siteId}/domains/{domainId}";
pub const SITE_DOMAIN_VERIFY: &str = "/app/v3/api/sites/{siteId}/domains/{domainId}/verify";
pub const SITE_DEPLOYMENTS: &str = "/app/v3/api/sites/{siteId}/deployments";
pub const SITE_DEPLOYMENT: &str = "/app/v3/api/sites/{siteId}/deployments/{deploymentId}";
pub const SITE_DEPLOYMENT_ROLLBACK: &str =
    "/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback";
pub const SITE_ENV_VARIABLES: &str = "/app/v3/api/sites/{siteId}/env_variables";
pub const CERTIFICATES: &str = "/app/v3/api/certificates";
pub const CERTIFICATE: &str = "/app/v3/api/certificates/{certificateId}";
pub const CERTIFICATE_RENEW: &str = "/app/v3/api/certificates/{certificateId}/renew";
pub const CERTIFICATES_UPLOAD: &str = "/app/v3/api/certificates/upload";
pub const UPLOAD_SESSIONS: &str = "/app/v3/api/upload_sessions";
pub const UPLOAD_SESSION: &str = "/app/v3/api/upload_sessions/{uploadSessionId}";
pub const UPLOAD_SESSION_COMPLETE: &str = "/app/v3/api/upload_sessions/{uploadSessionId}/complete";
pub const UPLOAD_SESSION_CANCEL: &str = "/app/v3/api/upload_sessions/{uploadSessionId}/cancel";
pub const ARTIFACTS: &str = "/app/v3/api/artifacts";
pub const ARTIFACT: &str = "/app/v3/api/artifacts/{artifactId}";
pub const SITE_RELEASES: &str = "/app/v3/api/sites/{siteId}/releases";
pub const SITE_RELEASE: &str = "/app/v3/api/sites/{siteId}/releases/{releaseId}";
pub const SITE_HEALTH_CHECKS: &str = "/app/v3/api/sites/{siteId}/health_checks";
