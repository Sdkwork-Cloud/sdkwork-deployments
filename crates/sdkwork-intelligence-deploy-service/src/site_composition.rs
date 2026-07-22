use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{SecondsFormat, Utc};
use sdkwork_deploy_content_provider_port::{
    ProviderRequestCredentials, ValidateContentProviderResourceCommand,
    ValidatedContentProviderResource,
};
use sdkwork_deploy_contract::{
    DeployAppRequestContext, DeployServiceError, DeployServiceResult, SiteCompositionResponse,
    UpdateSiteCompositionRequest,
};
use sdkwork_deploy_runtime_compiler::canonical_sha256_excluding_field;

#[derive(Clone, Debug)]
pub struct ReplaceSiteCompositionCommand {
    pub tenant_id: i64,
    pub organization_id: i64,
    pub actor_id: i64,
    pub site_uuid: String,
    pub expected_site_version: i64,
    pub idempotency_key: String,
    pub request_sha256: String,
    pub generated_at: String,
    pub request: UpdateSiteCompositionRequest,
    pub resources: Vec<ValidatedContentProviderResource>,
}

#[async_trait]
pub trait SiteCompositionRepositoryPort: Send + Sync {
    async fn replace_site_composition(
        &self,
        command: ReplaceSiteCompositionCommand,
    ) -> DeployServiceResult<SiteCompositionResponse>;
}

impl crate::DeployService {
    pub(crate) async fn update_composition(
        &self,
        context: &DeployAppRequestContext,
        site_id: &str,
        expected_site_version: i64,
        idempotency_key: &str,
        request: &UpdateSiteCompositionRequest,
    ) -> DeployServiceResult<SiteCompositionResponse> {
        validate_composition_request(site_id, expected_site_version, idempotency_key, request)?;
        let tenant_id = Self::require_tenant(context)?;
        let credentials = ProviderRequestCredentials {
            auth_token: context.auth_token.clone(),
            access_token: context.access_token.clone(),
        };
        let mut resources = Vec::with_capacity(request.resources.len());
        for resource in &request.resources {
            resources.push(
                self.content_provider
                    .validate_resource(
                        &credentials,
                        ValidateContentProviderResourceCommand {
                            tenant_id,
                            site_uuid: site_id.to_owned(),
                            resource: resource.clone(),
                        },
                    )
                    .await?,
            );
        }
        let request_value = serde_json::to_value(request)
            .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
        let request_sha256 =
            canonical_sha256_excluding_field(&request_value, "__no_excluded_field")
                .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
        self.repository
            .replace_site_composition(ReplaceSiteCompositionCommand {
                tenant_id,
                organization_id: context.organization_id.unwrap_or(0),
                actor_id: context.actor_id.unwrap_or(0),
                site_uuid: site_id.to_owned(),
                expected_site_version,
                idempotency_key: idempotency_key.to_owned(),
                request_sha256,
                generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                request: request.clone(),
                resources,
            })
            .await
    }
}

fn validate_composition_request(
    site_id: &str,
    expected_site_version: i64,
    idempotency_key: &str,
    request: &UpdateSiteCompositionRequest,
) -> DeployServiceResult<()> {
    validate_identifier(site_id, 128, "siteId")?;
    if expected_site_version < 0 {
        return Err(DeployServiceError::validation(
            "If-Match site version must not be negative",
        ));
    }
    validate_identifier(idempotency_key, 128, "Idempotency-Key")?;
    if request.resources.is_empty()
        || request.variants.is_empty()
        || request.mounts.is_empty()
        || request.bindings.is_empty()
    {
        return Err(DeployServiceError::validation(
            "resources, variants, mounts, and bindings must not be empty",
        ));
    }
    if request.resources.len() > request.limits.maximum_resources
        || request.variants.len() > request.limits.maximum_variants
        || request.variant_rules.len() > request.limits.maximum_variant_rules
        || request.mounts.len() > request.limits.maximum_mounts
        || request.bindings.len() > request.limits.maximum_bindings
    {
        return Err(DeployServiceError::validation(
            "site composition exceeds its declared runtime limits",
        ));
    }

    let resource_keys = unique_keys(
        request
            .resources
            .iter()
            .map(|resource| resource.key.as_str()),
        "resource",
    )?;
    let variant_keys = unique_keys(
        request.variants.iter().map(|variant| variant.key.as_str()),
        "variant",
    )?;
    unique_keys(
        request.variant_rules.iter().map(|rule| rule.key.as_str()),
        "variant rule",
    )?;
    unique_keys(
        request.mounts.iter().map(|mount| mount.key.as_str()),
        "mount",
    )?;
    unique_keys(
        request.bindings.iter().map(|binding| binding.key.as_str()),
        "binding",
    )?;
    if !variant_keys.contains(request.default_variant_key.as_str()) {
        return Err(DeployServiceError::validation(
            "defaultVariantKey must reference a variant",
        ));
    }
    for rule in &request.variant_rules {
        if !variant_keys.contains(rule.target_variant_key.as_str()) {
            return Err(DeployServiceError::validation(
                "variant rule references an unknown variant",
            ));
        }
    }
    for mount in &request.mounts {
        if !variant_keys.contains(mount.variant_key.as_str())
            || !resource_keys.contains(mount.resource_key.as_str())
        {
            return Err(DeployServiceError::validation(
                "mount references an unknown variant or resource",
            ));
        }
    }
    Ok(())
}

fn unique_keys<'a>(
    keys: impl Iterator<Item = &'a str>,
    label: &str,
) -> DeployServiceResult<HashSet<&'a str>> {
    let mut unique = HashSet::new();
    for key in keys {
        validate_identifier(key, 64, &format!("{label} key"))?;
        if !unique.insert(key) {
            return Err(DeployServiceError::validation(format!(
                "duplicate {label} key"
            )));
        }
    }
    Ok(unique)
}

fn validate_identifier(value: &str, maximum_len: usize, field: &str) -> DeployServiceResult<()> {
    if value.is_empty()
        || value.len() > maximum_len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(DeployServiceError::validation(format!(
            "{field} is invalid"
        )));
    }
    Ok(())
}
