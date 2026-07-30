use sdkwork_deploy_contract::{
    CreateDomainHostnameRequest, CreateDomainZoneRequest, DeployServiceError, DeployServiceResult,
    DomainHostnamePage, DomainHostnameResponse, DomainZonePage, DomainZoneResponse,
    ListDomainZonesQuery, UpdateDomainZoneRequest,
};
use sqlx::{any::AnyRow, Row};

use crate::support::{new_uuid, next_id, pagination, store_error};
use crate::DeployRepository;

const ZONE_SELECT: &str =
    "z.uuid, z.apex_hostname, z.display_name, z.dns_provider, z.status, z.updated_at, z.version,
     (SELECT COUNT(*) FROM deploy_domain d WHERE d.zone_id = z.id AND d.deleted_at IS NULL) AS hostname_count,
     (SELECT COUNT(*) FROM deploy_domain d WHERE d.zone_id = z.id AND d.verification_status = 'VERIFIED' AND d.deleted_at IS NULL) AS verified_hostname_count,
     (SELECT COUNT(DISTINCT ci.certificate_id) FROM deploy_certificate_identifier ci JOIN deploy_domain d ON d.id = ci.domain_id WHERE d.zone_id = z.id AND d.deleted_at IS NULL) AS certificate_count,
     (SELECT COUNT(*) FROM deploy_site_binding b JOIN deploy_domain d ON d.id = b.domain_id WHERE d.zone_id = z.id AND d.deleted_at IS NULL AND b.deleted_at IS NULL) AS binding_count";

const HOSTNAME_SELECT: &str =
    "d.uuid, z.uuid AS zone_uuid, z.apex_hostname, d.hostname_ascii, d.hostname_type,
     d.verification_status, d.verified_at, d.status, d.created_at, d.updated_at, d.version,
     (SELECT COUNT(DISTINCT ci.certificate_id) FROM deploy_certificate_identifier ci WHERE ci.domain_id = d.id) AS certificate_count,
     (SELECT COUNT(*) FROM deploy_site_binding b WHERE b.domain_id = d.id AND b.deleted_at IS NULL) AS binding_count";

impl DeployRepository {
    pub(super) async fn list_domain_zones_repo(
        &self,
        tenant_id: i64,
        query: &ListDomainZonesQuery,
    ) -> DeployServiceResult<DomainZonePage> {
        let (page, page_size, offset) = pagination(query.page, query.page_size);
        let status = query.status.as_deref().unwrap_or("");
        let keyword = query
            .keyword
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value.to_ascii_lowercase()))
            .unwrap_or_default();
        let predicate = "z.tenant_id = $1 AND z.deleted_at IS NULL
            AND ($2 = '' OR z.status = $2)
            AND ($3 = '' OR LOWER(z.apex_hostname) LIKE $3 OR LOWER(COALESCE(z.display_name, '')) LIKE $3)";
        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM deploy_dns_zone z WHERE {predicate}"
        ))
        .bind(tenant_id)
        .bind(status)
        .bind(&keyword)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_dns_zone", error))?;
        let rows = sqlx::query(&format!(
            "SELECT {ZONE_SELECT} FROM deploy_dns_zone z WHERE {predicate}
             ORDER BY z.updated_at DESC, z.id DESC LIMIT $4 OFFSET $5"
        ))
        .bind(tenant_id)
        .bind(status)
        .bind(keyword)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_dns_zone", error))?;
        let items = rows
            .iter()
            .map(map_zone_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                DeployServiceError::Internal(format!("map deploy_dns_zone: {error}"))
            })?;
        Ok(DomainZonePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_domain_zone_repo(
        &self,
        tenant_id: i64,
        organization_id: Option<i64>,
        actor_id: Option<i64>,
        request: &CreateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        let zone_id = next_id(self.id_generator())?;
        let zone_uuid = new_uuid();
        let hostname_id = next_id(self.id_generator())?;
        let hostname_uuid = new_uuid();
        let organization_id = organization_id.unwrap_or_default();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("begin create deploy_dns_zone", error))?;
        sqlx::query(
            "INSERT INTO deploy_dns_zone (
                id, uuid, tenant_id, organization_id, apex_hostname, display_name, dns_provider,
                provider_zone_ref, status, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'ACTIVE', $9, $9)",
        )
        .bind(zone_id)
        .bind(&zone_uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&request.apex_hostname)
        .bind(request.display_name.as_deref())
        .bind(request.dns_provider.as_deref())
        .bind(request.provider_zone_ref.as_deref())
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert deploy_dns_zone", error))?;
        sqlx::query(
            "INSERT INTO deploy_domain (
                id, uuid, tenant_id, organization_id, zone_id, hostname_ascii, hostname_type,
                verification_status, status, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, 'EXACT', 'PENDING', 'ACTIVE', $7, $7)",
        )
        .bind(hostname_id)
        .bind(hostname_uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(zone_id)
        .bind(&request.apex_hostname)
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| store_error("insert apex deploy_domain", error))?;
        transaction
            .commit()
            .await
            .map_err(|error| store_error("commit create deploy_dns_zone", error))?;
        self.retrieve_domain_zone_repo(tenant_id, &zone_uuid).await
    }

    pub(super) async fn retrieve_domain_zone_repo(
        &self,
        tenant_id: i64,
        zone_id: &str,
    ) -> DeployServiceResult<DomainZoneResponse> {
        let row = sqlx::query(&format!(
            "SELECT {ZONE_SELECT} FROM deploy_dns_zone z
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND z.deleted_at IS NULL"
        ))
        .bind(tenant_id)
        .bind(zone_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_dns_zone", error))?;
        row.as_ref()
            .map(map_zone_row)
            .transpose()
            .map_err(|error| DeployServiceError::Internal(format!("map deploy_dns_zone: {error}")))?
            .ok_or_else(|| DeployServiceError::not_found("domain zone not found"))
    }

    pub(super) async fn update_domain_zone_repo(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        zone_id: &str,
        request: &UpdateDomainZoneRequest,
    ) -> DeployServiceResult<DomainZoneResponse> {
        let result = sqlx::query(
            "UPDATE deploy_dns_zone SET
                display_name = COALESCE($3, display_name), dns_provider = COALESCE($4, dns_provider),
                provider_zone_ref = COALESCE($5, provider_zone_ref), status = COALESCE($6, status),
                updated_by = $7, updated_at = CURRENT_TIMESTAMP, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .bind(request.display_name.as_deref())
        .bind(request.dns_provider.as_deref())
        .bind(request.provider_zone_ref.as_deref())
        .bind(request.status.as_deref())
        .bind(actor_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("update deploy_dns_zone", error))?;
        if result.rows_affected() != 1 {
            return Err(DeployServiceError::not_found("domain zone not found"));
        }
        self.retrieve_domain_zone_repo(tenant_id, zone_id).await
    }

    pub(super) async fn delete_domain_zone_repo(
        &self,
        tenant_id: i64,
        zone_id: &str,
    ) -> DeployServiceResult<()> {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM deploy_domain d JOIN deploy_dns_zone z ON z.id = d.zone_id
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND z.deleted_at IS NULL AND d.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_dns_zone children", error))?;
        if child_count > 0 {
            return Err(DeployServiceError::conflict(
                "domain zone still contains hostnames",
            ));
        }
        let result = sqlx::query(
            "UPDATE deploy_dns_zone
             SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP, version = version + 1
             WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("delete deploy_dns_zone", error))?;
        if result.rows_affected() != 1 {
            return Err(DeployServiceError::not_found("domain zone not found"));
        }
        Ok(())
    }
}

impl DeployRepository {
    pub(super) async fn list_domain_hostnames_repo(
        &self,
        tenant_id: i64,
        zone_id: &str,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<DomainHostnamePage> {
        let (page, page_size, offset) = pagination(page, page_size);
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM deploy_domain d JOIN deploy_dns_zone z ON z.id = d.zone_id
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND z.deleted_at IS NULL AND d.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_domain by zone", error))?;
        let rows = sqlx::query(&format!(
            "SELECT {HOSTNAME_SELECT} FROM deploy_domain d
             JOIN deploy_dns_zone z ON z.id = d.zone_id
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND z.deleted_at IS NULL AND d.deleted_at IS NULL
             ORDER BY CASE WHEN d.hostname_ascii = z.apex_hostname THEN 0 ELSE 1 END,
                      d.hostname_ascii, d.id LIMIT $3 OFFSET $4"
        ))
        .bind(tenant_id)
        .bind(zone_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("list deploy_domain by zone", error))?;
        let items = rows
            .iter()
            .map(map_hostname_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| DeployServiceError::Internal(format!("map deploy_domain: {error}")))?;
        Ok(DomainHostnamePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub(super) async fn create_domain_hostname_repo(
        &self,
        tenant_id: i64,
        actor_id: Option<i64>,
        zone_id: &str,
        request: &CreateDomainHostnameRequest,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        let zone = sqlx::query(
            "SELECT id, organization_id, apex_hostname FROM deploy_dns_zone
             WHERE tenant_id = $1 AND uuid = $2 AND status = 'ACTIVE' AND deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("resolve deploy_dns_zone", error))?
        .ok_or_else(|| DeployServiceError::not_found("domain zone not found"))?;
        let zone_internal_id: i64 = zone.try_get("id").map_err(|error| {
            DeployServiceError::Internal(format!("map deploy_dns_zone id: {error}"))
        })?;
        let organization_id: i64 = zone.try_get("organization_id").map_err(|error| {
            DeployServiceError::Internal(format!("map deploy_dns_zone organization: {error}"))
        })?;
        let apex_hostname: String = zone.try_get("apex_hostname").map_err(|error| {
            DeployServiceError::Internal(format!("map deploy_dns_zone apex: {error}"))
        })?;
        let hostname = hostname_from_relative_name(&request.relative_name, &apex_hostname)?;
        let hostname_type = if hostname.starts_with("*.") {
            "WILDCARD"
        } else {
            "EXACT"
        };
        let id = next_id(self.id_generator())?;
        let uuid = new_uuid();
        sqlx::query(
            "INSERT INTO deploy_domain (
                id, uuid, tenant_id, organization_id, zone_id, hostname_ascii, hostname_type,
                verification_status, status, created_by, updated_by
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'PENDING', 'ACTIVE', $8, $8)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(zone_internal_id)
        .bind(hostname)
        .bind(hostname_type)
        .bind(actor_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("insert deploy_domain hostname", error))?;
        self.retrieve_domain_hostname_repo(tenant_id, zone_id, &uuid)
            .await
    }

    pub(super) async fn retrieve_domain_hostname_repo(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<DomainHostnameResponse> {
        let row = sqlx::query(&format!(
            "SELECT {HOSTNAME_SELECT} FROM deploy_domain d JOIN deploy_dns_zone z ON z.id = d.zone_id
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND d.uuid = $3
               AND z.deleted_at IS NULL AND d.deleted_at IS NULL"
        ))
        .bind(tenant_id)
        .bind(zone_id)
        .bind(hostname_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("retrieve deploy_domain hostname", error))?;
        row.as_ref()
            .map(map_hostname_row)
            .transpose()
            .map_err(|error| DeployServiceError::Internal(format!("map deploy_domain: {error}")))?
            .ok_or_else(|| DeployServiceError::not_found("domain hostname not found"))
    }

    pub(super) async fn delete_domain_hostname_repo(
        &self,
        tenant_id: i64,
        zone_id: &str,
        hostname_id: &str,
    ) -> DeployServiceResult<()> {
        let references: i64 = sqlx::query_scalar(
            "SELECT
                (SELECT COUNT(*) FROM deploy_site_binding b WHERE b.domain_id = d.id AND b.deleted_at IS NULL)
                + (SELECT COUNT(*) FROM deploy_certificate_identifier ci WHERE ci.domain_id = d.id)
             FROM deploy_domain d JOIN deploy_dns_zone z ON z.id = d.zone_id
             WHERE z.tenant_id = $1 AND z.uuid = $2 AND d.uuid = $3
               AND z.deleted_at IS NULL AND d.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .bind(hostname_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("count deploy_domain references", error))?
        .ok_or_else(|| DeployServiceError::not_found("domain hostname not found"))?;
        if references > 0 {
            return Err(DeployServiceError::conflict(
                "domain hostname is still bound to an application or certificate",
            ));
        }
        let result = sqlx::query(
            "UPDATE deploy_domain d
             SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP, version = version + 1
             FROM deploy_dns_zone z
             WHERE d.zone_id = z.id AND z.tenant_id = $1 AND z.uuid = $2 AND d.uuid = $3
               AND z.deleted_at IS NULL AND d.deleted_at IS NULL",
        )
        .bind(tenant_id)
        .bind(zone_id)
        .bind(hostname_id)
        .execute(&self.pool)
        .await
        .map_err(|error| store_error("delete deploy_domain hostname", error))?;
        if result.rows_affected() != 1 {
            return Err(DeployServiceError::not_found("domain hostname not found"));
        }
        Ok(())
    }
}

fn hostname_from_relative_name(
    relative_name: &str,
    apex_hostname: &str,
) -> DeployServiceResult<String> {
    let relative_name = relative_name.trim().to_ascii_lowercase();
    if relative_name == "@" {
        return Ok(apex_hostname.to_owned());
    }
    if relative_name.is_empty()
        || relative_name.ends_with('.')
        || relative_name.contains("..")
        || relative_name.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || (label == "*" && relative_name != "*")
                || (label != "*"
                    && (label.starts_with('-')
                        || label.ends_with('-')
                        || !label
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')))
        })
    {
        return Err(DeployServiceError::validation(
            "relative domain name is invalid",
        ));
    }
    let hostname = format!("{relative_name}.{apex_hostname}");
    if hostname.len() > 253 {
        return Err(DeployServiceError::validation("hostname is too long"));
    }
    Ok(hostname)
}

fn map_zone_row(row: &AnyRow) -> Result<DomainZoneResponse, sqlx::Error> {
    Ok(DomainZoneResponse {
        id: row.try_get("uuid")?,
        apex_hostname: row.try_get("apex_hostname")?,
        display_name: row.try_get("display_name").ok(),
        dns_provider: row.try_get("dns_provider").ok(),
        status: row.try_get("status")?,
        hostname_count: row.try_get("hostname_count")?,
        verified_hostname_count: row.try_get("verified_hostname_count")?,
        certificate_count: row.try_get("certificate_count")?,
        binding_count: row.try_get("binding_count")?,
        updated_at: row.try_get("updated_at")?,
        version: row.try_get::<i64, _>("version")?.to_string(),
    })
}

fn map_hostname_row(row: &AnyRow) -> Result<DomainHostnameResponse, sqlx::Error> {
    let hostname: String = row.try_get("hostname_ascii")?;
    let apex: String = row.try_get("apex_hostname")?;
    let relative_name = if hostname == apex {
        "@".to_owned()
    } else {
        hostname
            .strip_suffix(&format!(".{apex}"))
            .unwrap_or(&hostname)
            .to_owned()
    };
    Ok(DomainHostnameResponse {
        id: row.try_get("uuid")?,
        zone_id: row.try_get("zone_uuid")?,
        hostname,
        relative_name,
        hostname_type: row.try_get("hostname_type")?,
        verification_status: row.try_get("verification_status")?,
        verified_at: row.try_get("verified_at").ok(),
        status: row.try_get("status")?,
        certificate_count: row.try_get("certificate_count")?,
        binding_count: row.try_get("binding_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        version: row.try_get::<i64, _>("version")?.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::hostname_from_relative_name;

    #[test]
    fn relative_names_are_scoped_to_the_zone() {
        assert_eq!(
            hostname_from_relative_name("@", "example.com").unwrap(),
            "example.com"
        );
        assert_eq!(
            hostname_from_relative_name("docs", "example.com").unwrap(),
            "docs.example.com"
        );
        assert_eq!(
            hostname_from_relative_name("*", "example.com").unwrap(),
            "*.example.com"
        );
        for name in ["", "foo..bar", "foo.*", "-bad", "bad-"] {
            assert!(hostname_from_relative_name(name, "example.com").is_err());
        }
    }
}
