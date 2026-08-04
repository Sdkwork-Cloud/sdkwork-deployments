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
        execute_composition_update(
            self.repository.as_ref(),
            self.content_provider.as_ref(),
            CompositionUpdateInput {
                context,
                tenant_id,
                site_id,
                expected_site_version,
                idempotency_key,
                request,
            },
        )
        .await
    }
}

struct CompositionUpdateInput<'a> {
    context: &'a DeployAppRequestContext,
    tenant_id: i64,
    site_id: &'a str,
    expected_site_version: i64,
    idempotency_key: &'a str,
    request: &'a UpdateSiteCompositionRequest,
}

async fn execute_composition_update<R, C>(
    repository: &R,
    content_provider: &C,
    input: CompositionUpdateInput<'_>,
) -> DeployServiceResult<SiteCompositionResponse>
where
    R: SiteCompositionRepositoryPort + ?Sized,
    C: sdkwork_deploy_content_provider_port::ContentProviderPort + ?Sized,
{
    let credentials = ProviderRequestCredentials {
        auth_token: input.context.auth_token.clone(),
        access_token: input.context.access_token.clone(),
    };
    let mut resources = Vec::with_capacity(input.request.resources.len());
    for resource in &input.request.resources {
        resources.push(
            content_provider
                .validate_resource(
                    &credentials,
                    ValidateContentProviderResourceCommand {
                        tenant_id: input.tenant_id,
                        site_uuid: input.site_id.to_owned(),
                        resource: resource.clone(),
                    },
                )
                .await?,
        );
    }
    let request_value = serde_json::to_value(input.request)
        .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
    let request_sha256 = canonical_sha256_excluding_field(&request_value, "__no_excluded_field")
        .map_err(|error| DeployServiceError::Internal(error.to_string()))?;
    repository
        .replace_site_composition(ReplaceSiteCompositionCommand {
            tenant_id: input.tenant_id,
            organization_id: input.context.organization_id.unwrap_or(0),
            actor_id: input.context.actor_id.unwrap_or(0),
            site_uuid: input.site_id.to_owned(),
            expected_site_version: input.expected_site_version,
            idempotency_key: input.idempotency_key.to_owned(),
            request_sha256,
            generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            request: input.request.clone(),
            resources,
        })
        .await
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use sdkwork_deploy_content_provider_port::{
        ContentProviderPort, ProviderRequestCredentials, ValidateContentProviderResourceCommand,
        ValidatedContentProviderResource,
    };
    use sdkwork_deploy_contract::{
        ContentProviderResourceSource, DriveWebsiteContentMode, DriveWebsiteRootSelector,
        SiteBindingAction, SiteBindingDefinition, SiteClientClass, SiteEnvironment,
        SiteMountDefinition, SiteMountHandler, SiteMountMode, SiteResourceDefinition,
        SiteVariantDefinition,
    };

    use super::*;

    struct RejectingContentProvider;

    #[async_trait]
    impl ContentProviderPort for RejectingContentProvider {
        async fn validate_resource(
            &self,
            _credentials: &ProviderRequestCredentials,
            _command: ValidateContentProviderResourceCommand,
        ) -> DeployServiceResult<ValidatedContentProviderResource> {
            Err(DeployServiceError::forbidden(
                "site composition update is forbidden for this tenant",
            ))
        }
    }

    #[derive(Default)]
    struct RecordingCompositionRepository {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl SiteCompositionRepositoryPort for RecordingCompositionRepository {
        async fn replace_site_composition(
            &self,
            _command: ReplaceSiteCompositionCommand,
        ) -> DeployServiceResult<SiteCompositionResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(DeployServiceError::Internal(
                "repository should not be called".to_owned(),
            ))
        }
    }

    fn request() -> UpdateSiteCompositionRequest {
        UpdateSiteCompositionRequest {
            environment: SiteEnvironment::Production,
            default_variant_key: "default".to_owned(),
            resources: vec![SiteResourceDefinition {
                key: "content".to_owned(),
                source: ContentProviderResourceSource::drive_directory(
                    "space-1".to_owned(),
                    DriveWebsiteRootSelector::SpaceRoot,
                    DriveWebsiteContentMode::LiveTree,
                ),
            }],
            variants: vec![SiteVariantDefinition {
                key: "default".to_owned(),
                label: "Default".to_owned(),
                client_class: SiteClientClass::Other,
                priority: 0,
            }],
            variant_rules: vec![],
            mounts: vec![SiteMountDefinition {
                key: "root".to_owned(),
                variant_key: "default".to_owned(),
                resource_key: "content".to_owned(),
                path_prefix: "/".to_owned(),
                resource_subpath: "/".to_owned(),
                mode: SiteMountMode::Root,
                handler: SiteMountHandler::Static,
                index_files: vec!["index.html".to_owned()],
                spa_fallback: None,
                priority: 0,
            }],
            bindings: vec![SiteBindingDefinition {
                key: "primary".to_owned(),
                domain_id: "domain-1".to_owned(),
                path_prefix: "/".to_owned(),
                action: SiteBindingAction::Serve {
                    default_variant_key: None,
                    forced_variant_key: None,
                },
            }],
            delivery_policy: Default::default(),
            security_policy: Default::default(),
            limits: Default::default(),
            observability_policy: Default::default(),
        }
    }

    #[tokio::test]
    async fn provider_validation_finishes_before_repository_mutation() {
        let repository = RecordingCompositionRepository::default();
        let context = DeployAppRequestContext {
            tenant_id: 7,
            actor_id: Some(11),
            organization_id: Some(9),
            auth_token: Some("auth-token".to_owned()),
            access_token: Some("access-token".to_owned()),
            ..Default::default()
        };
        let error = execute_composition_update(
            &repository,
            &RejectingContentProvider,
            CompositionUpdateInput {
                context: &context,
                tenant_id: 7,
                site_id: "site-1",
                expected_site_version: 0,
                idempotency_key: "composition-1",
                request: &request(),
            },
        )
        .await
        .expect_err("provider rejection must abort before persistence");
        assert!(matches!(error, DeployServiceError::Forbidden(_)));
        assert_eq!(repository.calls.load(Ordering::SeqCst), 0);
    }
}
