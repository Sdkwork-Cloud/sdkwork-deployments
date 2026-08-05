pub const PREFIX: &str = "/app/v3/api";

pub const DOMAIN_ZONES: &str = "/app/v3/api/domain_zones";
pub const DOMAIN_ZONE: &str = "/app/v3/api/domain_zones/{zoneId}";
pub const DOMAIN_ZONE_HOSTNAMES: &str = "/app/v3/api/domain_zones/{zoneId}/hostnames";
pub const DOMAIN_ZONE_HOSTNAME: &str = "/app/v3/api/domain_zones/{zoneId}/hostnames/{hostnameId}";
pub const DOMAIN_ZONE_HOSTNAME_VERIFY: &str =
    "/app/v3/api/domain_zones/{zoneId}/hostnames/{hostnameId}/verify";
pub const SITES: &str = "/app/v3/api/sites";
pub const SITE: &str = "/app/v3/api/sites/{siteId}";
pub const SITE_COMPOSITION: &str = "/app/v3/api/sites/{siteId}/composition";
pub const SITE_ACTIVATE: &str = "/app/v3/api/sites/{siteId}/activate";
pub const SITE_PAUSE: &str = "/app/v3/api/sites/{siteId}/pause";
pub const SITE_DEPLOYMENTS: &str = "/app/v3/api/sites/{siteId}/deployments";
pub const SITE_DEPLOYMENT: &str = "/app/v3/api/sites/{siteId}/deployments/{deploymentId}";
pub const SITE_DEPLOYMENT_ROLLBACK: &str =
    "/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback";
pub const SITE_ENV_VARIABLES: &str = "/app/v3/api/sites/{siteId}/env_variables";
pub const CERTIFICATES: &str = "/app/v3/api/certificates";
pub const CERTIFICATE: &str = "/app/v3/api/certificates/{certificateId}";
pub const CERTIFICATE_RENEW: &str = "/app/v3/api/certificates/{certificateId}/renew";
pub const UPLOAD_SESSIONS: &str = "/app/v3/api/upload_sessions";
pub const UPLOAD_SESSION: &str = "/app/v3/api/upload_sessions/{uploadSessionId}";
pub const UPLOAD_SESSION_COMPLETE: &str = "/app/v3/api/upload_sessions/{uploadSessionId}/complete";
pub const UPLOAD_SESSION_CANCEL: &str = "/app/v3/api/upload_sessions/{uploadSessionId}/cancel";
pub const ARTIFACTS: &str = "/app/v3/api/artifacts";
pub const ARTIFACT: &str = "/app/v3/api/artifacts/{artifactId}";
pub const SITE_RELEASES: &str = "/app/v3/api/sites/{siteId}/releases";
pub const SITE_RELEASE: &str = "/app/v3/api/sites/{siteId}/releases/{releaseId}";
pub const SITE_HEALTH_CHECKS: &str = "/app/v3/api/sites/{siteId}/health_checks";

pub const APPS: &str = "/app/v3/api/apps";
pub const APP: &str = "/app/v3/api/apps/{appId}";
pub const APP_PLATFORM_TARGETS: &str = "/app/v3/api/apps/{appId}/platform_targets";
pub const APP_PLATFORM_TARGET: &str =
    "/app/v3/api/apps/{appId}/platform_targets/{platformTargetId}";
pub const APP_SOURCE_REPOSITORIES: &str = "/app/v3/api/apps/{appId}/source_repositories";
pub const APP_SOURCE_REPOSITORY: &str =
    "/app/v3/api/apps/{appId}/source_repositories/{sourceRepositoryId}";
pub const BUILD_TEMPLATES: &str = "/app/v3/api/build_templates";
pub const BUILD_TEMPLATE: &str = "/app/v3/api/build_templates/{buildTemplateId}";
pub const APP_BUILDS: &str = "/app/v3/api/apps/{appId}/builds";
pub const APP_BUILD: &str = "/app/v3/api/apps/{appId}/builds/{buildId}";
pub const APP_BUILD_STATE: &str = "/app/v3/api/apps/{appId}/builds/{buildId}/state";
pub const APP_PACKAGES: &str = "/app/v3/api/apps/{appId}/packages";
pub const APP_PACKAGE: &str = "/app/v3/api/apps/{appId}/packages/{packageId}";
pub const APP_RELEASES: &str = "/app/v3/api/apps/{appId}/releases";
pub const APP_RELEASE: &str = "/app/v3/api/apps/{appId}/releases/{releaseId}";
pub const APP_CHANNELS: &str = "/app/v3/api/apps/{appId}/channels";
pub const APP_CHANNEL: &str = "/app/v3/api/apps/{appId}/channels/{channelId}";
pub const APP_CHANNEL_PROMOTIONS: &str = "/app/v3/api/apps/{appId}/channels/{channelId}/promotions";
pub const APP_CHANNEL_ROLLOUTS: &str = "/app/v3/api/apps/{appId}/channels/{channelId}/rollouts";
pub const APP_DEPLOYMENTS: &str = "/app/v3/api/apps/{appId}/deployments";
pub const APP_DEPLOYMENT: &str = "/app/v3/api/apps/{appId}/deployments/{deploymentId}";
pub const SIGNING_IDENTITIES: &str = "/app/v3/api/signing_identities";
pub const SIGNING_IDENTITY: &str = "/app/v3/api/signing_identities/{signingIdentityId}";
pub const USAGE_EVENTS: &str = "/app/v3/api/usage_events";
pub const APP_DATABASE_PROFILES: &str = "/app/v3/api/apps/{appId}/database_profiles";
pub const APP_DATABASE_PROFILE: &str = "/app/v3/api/apps/{appId}/database_profiles/{profileId}";
pub const APP_DATABASE_MIGRATIONS: &str =
    "/app/v3/api/apps/{appId}/database_profiles/{profileId}/migrations";
pub const APP_DATABASE_MIGRATION: &str =
    "/app/v3/api/apps/{appId}/database_profiles/{profileId}/migrations/{migrationId}";
pub const APP_ENVIRONMENTS: &str = "/app/v3/api/apps/{appId}/environments";
pub const APP_ENVIRONMENT: &str = "/app/v3/api/apps/{appId}/environments/{environmentId}";
pub const APP_ENVIRONMENT_PROMOTIONS: &str =
    "/app/v3/api/apps/{appId}/environments/{environmentId}/promotions";
