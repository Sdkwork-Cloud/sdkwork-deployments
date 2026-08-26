//! Platform app publishing domains: idempotent provisioning of every app's
//! default publishable hostnames (`<slug>.app[-<env>].<suffix>`) and the
//! hostname → app resolution the Web Server fallback uses.
//!
//! The platform owns the apex domains (`app.<suffix>`), so provisioned
//! hostnames are automatically `VERIFIED`; custom domains keep the regular
//! DNS verification flow (`domain_zones.rs`).

use sdkwork_deploy_contract::{
    DeployServiceError, DeployServiceResult, ProvisionAppDomainsResult, ResolvedDeployServer,
};
use sdkwork_deploy_core::{app_domain_label, default_app_hostname, PLATFORM_APP_DOMAIN_SUFFIXES};
use sqlx::Row;

use crate::support::{new_uuid, next_id, now_rfc3339, store_error};
use crate::DeployRepository;

/// Batch traffic usage ingest for the Web Server usage metering
/// (inherent method so read-only consumers share one entry point).
impl DeployRepository {
    pub async fn ingest_usage_events_lookup(
        &self,
        events: &[sdkwork_deploy_contract::UsageEventIngestItem],
    ) -> sdkwork_deploy_contract::DeployServiceResult<sdkwork_deploy_contract::UsageIngestResult>
    {
        self.insert_usage_events_batch_repo(events).await
    }
}

/// Read-only hostname resolution for the Web Server app-domain fallback.
/// Inherent method so read-only consumers (for example the standalone
/// gateway, which shares the Deploy database) resolve servers without the
/// service port trait.
impl DeployRepository {
    pub async fn resolve_server_by_hostname_lookup(
        &self,
        hostname: &str,
        environment: &str,
    ) -> DeployServiceResult<Option<ResolvedDeployServer>> {
        self.resolve_active_app_by_hostname_repo(hostname, environment)
            .await
    }
}

/// Zone apex for one platform suffix: `app.<suffix>`.
fn platform_zone_apex(suffix: &str) -> String {
    format!("app.{suffix}")
}

impl DeployRepository {
    /// Create the platform app-domain DNS zones for a tenant (one per
    /// platform suffix, apex `app.<suffix>`) and their apex hostname rows.
    /// Idempotent: existing zones are kept and not counted. Returns the
    /// number of newly created zones.
    pub(super) async fn ensure_platform_app_zones_repo(
        &self,
        tenant_id: i64,
        organization_id: i64,
        actor_id: Option<i64>,
    ) -> DeployServiceResult<usize> {
        let mut created = 0;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin platform app zones", error))?;
        for suffix in PLATFORM_APP_DOMAIN_SUFFIXES {
            let apex = platform_zone_apex(suffix);
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM deploy_dns_zone
                 WHERE tenant_id = $1 AND apex_hostname = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id)
            .bind(&apex)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("lookup platform app zone", error))?;
            if exists.is_some() {
                continue;
            }
            let zone_id = next_id(self.id_generator())?;
            let zone_uuid = new_uuid();
            let apex_domain_id = next_id(self.id_generator())?;
            sqlx::query(
                "INSERT INTO deploy_dns_zone (
                    id, uuid, tenant_id, organization_id, apex_hostname, display_name,
                    dns_provider, provider_zone_ref, status, created_by, updated_by
                 ) VALUES ($1, $2, $3, $4, $5, $6, 'platform', $7, 'ACTIVE', $8, $8)",
            )
            .bind(zone_id)
            .bind(&zone_uuid)
            .bind(tenant_id)
            .bind(organization_id)
            .bind(&apex)
            .bind(format!("Platform app domain zone {}", suffix))
            .bind(format!("app.*.{suffix}"))
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert platform app zone", error))?;
            // The zone apex is platform-owned and therefore auto-verified.
            let now = now_rfc3339();
            sqlx::query(
                "INSERT INTO deploy_domain (
                    id, uuid, tenant_id, organization_id, zone_id, hostname_ascii, hostname_type,
                    verification_status, verified_at, status, created_by, updated_by
                 ) VALUES ($1, $2, $3, $4, $5, $6, 'EXACT', 'VERIFIED', $7, 'ACTIVE', $8, $8)",
            )
            .bind(apex_domain_id)
            .bind(new_uuid())
            .bind(tenant_id)
            .bind(organization_id)
            .bind(zone_id)
            .bind(&apex)
            .bind(&now)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert platform app zone apex domain", error))?;
            created += 1;
        }
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit platform app zones", error))?;
        Ok(created)
    }

    /// Idempotently provision an app's default publishing domains for one
    /// lifecycle environment: for every platform suffix, an EXACT
    /// `deploy_domain` (`<slug>.app[-<env>].<suffix>`, auto-verified) and a
    /// `SERVE` binding on the app's binding. The first suffix (`sdkwork.com`)
    /// binding is the canonical one.
    pub(super) async fn provision_app_default_domains_repo(
        &self,
        tenant_id: i64,
        organization_id: i64,
        actor_id: Option<i64>,
        app_id: &str,
        app_slug: &str,
        environment: &str,
    ) -> DeployServiceResult<ProvisionAppDomainsResult> {
        let app_id = crate::support::resolve_app_internal_id(&self.pool, tenant_id, app_id).await?;
        let mut result = ProvisionAppDomainsResult::default();
        let label = app_domain_label(environment);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin provision app default domains", error))?;
        for (index, suffix) in PLATFORM_APP_DOMAIN_SUFFIXES.iter().enumerate() {
            let hostname = default_app_hostname(app_slug, suffix, environment);
            let zone_id: i64 = sqlx::query_scalar(
                "SELECT id FROM deploy_dns_zone
                 WHERE tenant_id = $1 AND apex_hostname = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id)
            .bind(platform_zone_apex(suffix))
            .fetch_one(&mut *transaction)
            .await
            .map_err(|error| store_error("lookup platform app zone", error))?;
            let domain_id: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM deploy_domain
                 WHERE tenant_id = $1 AND hostname_ascii = $2 AND deleted_at IS NULL",
            )
            .bind(tenant_id)
            .bind(&hostname)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("lookup app default domain", error))?;
            // `hostname_ascii` is unique across all active domains, so a
            // default publishing hostname claimed by another tenant must
            // fail with a clear conflict instead of a generic constraint
            // violation (the app slug must be unique platform-wide for the
            // default `<slug>.app[-<env>].<suffix>` catalog).
            let cross_tenant: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM deploy_domain
                 WHERE hostname_ascii = $1 AND deleted_at IS NULL AND tenant_id <> $2
                 LIMIT 1",
            )
            .bind(&hostname)
            .bind(tenant_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("lookup cross-tenant app default domain", error))?;
            if domain_id.is_none() && cross_tenant.is_some() {
                return Err(DeployServiceError::conflict(format!(
                    "default publishing hostname {hostname} is already registered by another tenant; choose a different app slug"
                )));
            }
            let domain_id = match domain_id {
                Some(existing) => {
                    result.existing_domains += 1;
                    existing
                }
                None => {
                    let id = next_id(self.id_generator())?;
                    let now = now_rfc3339();
                    sqlx::query(
                        "INSERT INTO deploy_domain (
                            id, uuid, tenant_id, organization_id, zone_id, hostname_ascii,
                            hostname_type, verification_status, verified_at, status,
                            created_by, updated_by
                         ) VALUES ($1, $2, $3, $4, $5, $6, 'EXACT', 'VERIFIED', $7, 'ACTIVE', $8, $8)",
                    )
                    .bind(id)
                    .bind(new_uuid())
                    .bind(tenant_id)
                    .bind(organization_id)
                    .bind(zone_id)
                    .bind(&hostname)
                    .bind(&now)
                    .bind(actor_id)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|error| store_error("insert app default domain", error))?;
                    result.created_domains += 1;
                    id
                }
            };
            let binding_exists: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM deploy_app_binding
                 WHERE app_id = $1 AND hostname_ascii = $2 AND deleted_at IS NULL",
            )
            .bind(app_id)
            .bind(&hostname)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|error| store_error("lookup app default binding", error))?;
            if binding_exists.is_some() {
                result.existing_bindings += 1;
                result.hostnames.push(hostname);
                continue;
            }
            let binding_id = next_id(self.id_generator())?;
            let now = now_rfc3339();
            let binding_key = format!("appd-{label}-{index}");
            sqlx::query(
                "INSERT INTO deploy_app_binding (
                    id, uuid, tenant_id, organization_id, app_id, binding_key, domain_id,
                    hostname_ascii, environment, path_prefix, action_type, is_canonical,
                    status, verified_at, activated_at, created_by, updated_by,
                    created_at, updated_at, version
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '/', 'SERVE', $10,
                    'ACTIVE', $11, $11, $12, $12, $11, $11, 1)",
            )
            .bind(binding_id)
            .bind(new_uuid())
            .bind(tenant_id)
            .bind(organization_id)
            .bind(app_id)
            .bind(&binding_key)
            .bind(domain_id)
            .bind(&hostname)
            .bind(environment)
            .bind(index == 0)
            .bind(&now)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(|error| store_error("insert app default binding", error))?;
            result.created_bindings += 1;
            result.hostnames.push(hostname);
        }
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit provision app default domains", error))?;
        Ok(result)
    }

    /// Resolve an active app binding by its exact hostname in one lifecycle
    /// environment and return the site's latest compiled website runtime
    /// descriptor (`deploy_app_revision.descriptor_json`). This is the Web
    /// Server fallback lookup: custom domains and default app domains both
    /// land here (default app bindings are explicit rows).
    pub(super) async fn resolve_active_app_by_hostname_repo(
        &self,
        hostname: &str,
        environment: &str,
    ) -> DeployServiceResult<Option<ResolvedDeployServer>> {
        let hostname = hostname.trim().to_ascii_lowercase();
        if hostname.is_empty() || hostname.len() > 253 || hostname.ends_with('.') {
            return Err(DeployServiceError::validation(
                "hostname must be normalized lowercase ASCII without a trailing dot",
            ));
        }
        let row = sqlx::query(
            "SELECT s.uuid AS app_uuid, s.slug AS app_slug, b.tenant_id,
                    b.hostname_ascii, b.path_prefix, b.action_type, b.uuid AS binding_uuid,
                    r.descriptor_json, r.descriptor_sha256, r.revision_no, b.environment
             FROM deploy_app_binding b
             JOIN deploy_app s ON s.id = b.app_id AND s.deleted_at IS NULL
             JOIN deploy_app_revision r ON r.id = s.current_revision_id
             WHERE b.hostname_ascii = $1 AND b.environment = $2
               AND b.status = 'ACTIVE' AND b.deleted_at IS NULL
               AND r.validation_status = 'VALID'
             ORDER BY b.id DESC
             LIMIT 1",
        )
        .bind(&hostname)
        .bind(environment)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve active app by hostname", error))?;
        let Some(row) = row else {
            return Ok(None);
        };
        let app_uuid: String = row
            .try_get("app_uuid")
            .map_err(|error| DeployServiceError::Internal(format!("read app uuid: {error}")))?;
        let app_slug: String = row
            .try_get("app_slug")
            .map_err(|error| DeployServiceError::Internal(format!("read app slug: {error}")))?;
        let tenant_id: i64 = row
            .try_get("tenant_id")
            .map_err(|error| DeployServiceError::Internal(format!("read tenant id: {error}")))?;
        let binding_uuid: Option<String> = row.try_get("binding_uuid").ok();
        let hostname: String = row
            .try_get("hostname_ascii")
            .map_err(|error| DeployServiceError::Internal(format!("read hostname: {error}")))?;
        let path_prefix: String = row
            .try_get("path_prefix")
            .map_err(|error| DeployServiceError::Internal(format!("read path prefix: {error}")))?;
        let action_type: String = row
            .try_get("action_type")
            .map_err(|error| DeployServiceError::Internal(format!("read action type: {error}")))?;
        let descriptor_json: serde_json::Value = row
            .try_get("descriptor_json")
            .map_err(|error| DeployServiceError::Internal(format!("read descriptor: {error}")))?;
        let descriptor_sha256: String = row.try_get("descriptor_sha256").map_err(|error| {
            DeployServiceError::Internal(format!("read descriptor hash: {error}"))
        })?;
        let revision_no: i64 = row
            .try_get("revision_no")
            .map_err(|error| DeployServiceError::Internal(format!("read revision no: {error}")))?;
        let environment: String = row
            .try_get("environment")
            .map_err(|error| DeployServiceError::Internal(format!("read environment: {error}")))?;
        Ok(Some(ResolvedDeployServer {
            app_uuid: app_uuid.clone(),
            app_slug,
            hostname,
            path_prefix,
            action_type,
            tenant_id,
            app_id: Some(app_uuid),
            binding_id: binding_uuid,
            descriptor_json,
            descriptor_sha256,
            revision_no,
            environment,
        }))
    }
}
