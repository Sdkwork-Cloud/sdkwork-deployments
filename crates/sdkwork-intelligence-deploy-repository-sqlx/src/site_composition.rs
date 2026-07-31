use std::collections::BTreeMap;

use async_trait::async_trait;
use sdkwork_deploy_content_provider_port::ValidatedContentProviderResource;
use sdkwork_deploy_contract::{
    DeployServiceError, DeployServiceResult, SiteBindingAction, SiteClientClass,
    SiteCompositionResponse, SiteMountHandler, SiteMountMode, SiteRedirectScheme,
    SiteRevisionResponse, SiteRuntimeAssignmentResponse, SiteVariantRuleMatcher,
};
use sdkwork_deploy_runtime_compiler::{
    canonical_sha256_excluding_field, compile_runtime_set, compile_site_revision,
    normalize_runtime_descriptors, runtime_set_size_bytes, RuntimeBinding, RuntimeBindingAction,
    RuntimeClientClass, RuntimeDeliveryPolicy, RuntimeEnvironment, RuntimeHandler, RuntimeLimits,
    RuntimeMount, RuntimeMountMode, RuntimeMountTranslation, RuntimeObservabilityPolicy,
    RuntimeProviderReference, RuntimeProviderType, RuntimeRedirectScheme, RuntimeResource,
    RuntimeSecurityPolicy, RuntimeSetCompilationInput, RuntimeVariant, RuntimeVariantRule,
    RuntimeVariantRuleMatcher, SiteRuntimeCompilationInput, DESCRIPTOR_COMPILER_VERSION,
    WEBSITE_RUNTIME_SCHEMA_VERSION,
};
use sdkwork_intelligence_deploy_service::{
    ReplaceSiteCompositionCommand, SiteCompositionRepositoryPort,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::support::{new_uuid, next_id};
use crate::DeployRepository;

const MAXIMUM_RUNTIME_GENERATION: i64 = 9_007_199_254_740_991;
const MAXIMUM_RUNTIME_SITES: usize = 10_000;

#[derive(Clone, Debug)]
struct StoredSite {
    id: i64,
    organization_id: i64,
    version: i64,
    desired_revision_id: Option<i64>,
}

#[derive(Clone, Debug)]
struct StoredVariant {
    id: i64,
    uuid: String,
}

#[derive(Clone, Debug)]
struct StoredResource {
    id: i64,
    uuid: String,
}

#[derive(Clone, Debug)]
struct StoredDomain {
    id: i64,
    hostname: String,
}

#[derive(Clone, Debug)]
struct StoredTarget {
    id: i64,
    uuid: String,
    node_uuid: String,
    tenant_scope_hash: String,
}

#[async_trait]
impl SiteCompositionRepositoryPort for DeployRepository {
    async fn replace_site_composition(
        &self,
        command: ReplaceSiteCompositionCommand,
    ) -> DeployServiceResult<SiteCompositionResponse> {
        self.replace_site_composition_repo(command).await
    }
}

impl DeployRepository {
    async fn replace_site_composition_repo(
        &self,
        command: ReplaceSiteCompositionCommand,
    ) -> DeployServiceResult<SiteCompositionResponse> {
        let mut transaction = begin_transaction(&self.pool).await?;
        if let Some(response) = load_idempotent_result(&mut transaction, &command).await? {
            transaction
                .commit()
                .await
                .map_err(|error| composition_store_error("commit idempotent composition", error))?;
            return Ok(response);
        }

        let site = lock_site(&mut transaction, &command).await?;
        reserve_site_version(&mut transaction, &command, &site).await?;
        let new_site_version = site.version + 1;
        let targets = load_targets(
            &mut transaction,
            command.tenant_id,
            command.request.environment.as_str(),
        )
        .await?;
        let tenant_scope_hash = consistent_tenant_scope_hash(&targets)?;

        delete_current_composition(&mut transaction, site.id).await?;
        let resources = insert_resources(self, &mut transaction, &command, &site).await?;
        let variants = insert_variants(self, &mut transaction, &command, &site).await?;
        let default_variant = variants
            .get(&command.request.default_variant_key)
            .ok_or_else(|| DeployServiceError::validation("default variant is missing"))?
            .clone();
        let runtime_rules =
            insert_variant_rules(self, &mut transaction, &command, &site, &variants).await?;
        let runtime_mounts = insert_mounts(
            self,
            &mut transaction,
            &command,
            &site,
            &variants,
            &resources,
        )
        .await?;
        let runtime_bindings =
            insert_bindings(self, &mut transaction, &command, &site, &variants).await?;

        let revision_id = next_id(self.id_generator())?;
        let revision_uuid = new_uuid();
        let revision_number = next_revision_number(&mut transaction, site.id).await?;
        let runtime_resources = command
            .resources
            .iter()
            .map(|resource| runtime_resource(resource, &resources))
            .collect::<DeployServiceResult<Vec<_>>>()?;
        let runtime_variants = command
            .request
            .variants
            .iter()
            .map(|variant| RuntimeVariant {
                variant_uuid: variants[&variant.key].uuid.clone(),
                label: variant.label.clone(),
            })
            .collect();
        let compiled = compile_site_revision(SiteRuntimeCompilationInput {
            revision_uuid: revision_uuid.clone(),
            site_uuid: command.site_uuid.clone(),
            tenant_scope_hash,
            environment: runtime_environment(command.request.environment),
            generated_at: command.generated_at.clone(),
            site_default_variant_uuid: default_variant.uuid.clone(),
            bindings: runtime_bindings,
            variants: runtime_variants,
            variant_rules: runtime_rules,
            resources: runtime_resources,
            mounts: runtime_mounts,
            delivery_policy: RuntimeDeliveryPolicy {
                provider_timeout_ms: command.request.delivery_policy.provider_timeout_ms,
                metadata_cache_ttl_seconds: command
                    .request
                    .delivery_policy
                    .metadata_cache_ttl_seconds,
                negative_cache_ttl_seconds: command
                    .request
                    .delivery_policy
                    .negative_cache_ttl_seconds,
                stale_while_revalidate_seconds: command
                    .request
                    .delivery_policy
                    .stale_while_revalidate_seconds,
                maximum_object_bytes: command.request.delivery_policy.maximum_object_bytes,
            },
            security_policy: RuntimeSecurityPolicy {
                force_https: command.request.security_policy.force_https,
                deny_dot_files: command.request.security_policy.deny_dot_files,
                denied_path_prefixes: command.request.security_policy.denied_path_prefixes.clone(),
            },
            limits: RuntimeLimits {
                maximum_bindings: command.request.limits.maximum_bindings,
                maximum_variants: command.request.limits.maximum_variants,
                maximum_variant_rules: command.request.limits.maximum_variant_rules,
                maximum_resources: command.request.limits.maximum_resources,
                maximum_mounts: command.request.limits.maximum_mounts,
                maximum_index_files_per_mount: command.request.limits.maximum_index_files_per_mount,
                maximum_path_bytes: command.request.limits.maximum_path_bytes,
                maximum_path_segments: command.request.limits.maximum_path_segments,
            },
            observability_policy: RuntimeObservabilityPolicy {
                access_log_enabled: command.request.observability_policy.access_log_enabled,
                usage_metering_enabled: command.request.observability_policy.usage_metering_enabled,
                trace_sample_rate_per_mille: command
                    .request
                    .observability_policy
                    .trace_sample_rate_per_mille,
            },
        })
        .map_err(|error| DeployServiceError::validation(error.to_string()))?;
        insert_revision(
            &mut transaction,
            &command,
            &site,
            revision_id,
            &revision_uuid,
            revision_number,
            new_site_version,
            &compiled.descriptor,
            &compiled.descriptor_sha256,
        )
        .await?;
        update_site_revision_pointers(
            &mut transaction,
            &command,
            site.id,
            default_variant.id,
            revision_id,
        )
        .await?;

        let mut descriptors = load_other_descriptors(
            &mut transaction,
            command.tenant_id,
            site.id,
            command.request.environment.as_str(),
        )
        .await?;
        descriptors.push(compiled.descriptor);
        normalize_runtime_descriptors(&mut descriptors);
        let assignments = insert_runtime_assignments(
            self,
            &mut transaction,
            &command,
            revision_id,
            &targets,
            &descriptors,
        )
        .await?;
        let response = SiteCompositionResponse {
            site_id: command.site_uuid.clone(),
            site_version: new_site_version.to_string(),
            revision: SiteRevisionResponse {
                id: revision_uuid.clone(),
                number: revision_number.to_string(),
                descriptor_sha256: compiled.descriptor_sha256,
                validation_status: "VALID".to_owned(),
            },
            runtime_assignments: assignments,
        };
        persist_command_result(&mut transaction, revision_id, &response).await?;
        insert_composition_audit(self, &mut transaction, &command, site.id, revision_id).await?;
        transaction
            .commit()
            .await
            .map_err(|error| composition_store_error("commit site composition", error))?;
        Ok(response)
    }
}

async fn begin_transaction(pool: &PgPool) -> DeployServiceResult<Transaction<'static, Postgres>> {
    pool.begin()
        .await
        .map_err(|error| composition_store_error("begin site composition transaction", error))
}

async fn load_idempotent_result(
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
) -> DeployServiceResult<Option<SiteCompositionResponse>> {
    let row = sqlx::query(
        "SELECT r.request_sha256, CAST(r.result_json AS TEXT) AS result_json
         FROM deploy_site_revision r
         INNER JOIN deploy_site s ON s.id = r.site_id
         WHERE r.tenant_id = $1 AND s.uuid = $2 AND r.idempotency_key = $3",
    )
    .bind(command.tenant_id)
    .bind(&command.site_uuid)
    .bind(&command.idempotency_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("load site composition idempotency", error))?;
    let Some(row) = row else {
        return Ok(None);
    };
    let request_sha256: String = row
        .try_get("request_sha256")
        .map_err(|_| DeployServiceError::Internal("invalid idempotency record".to_owned()))?;
    if request_sha256 != command.request_sha256 {
        return Err(DeployServiceError::conflict(
            "Idempotency-Key was already used with another site composition",
        ));
    }
    let result_json: String = row
        .try_get("result_json")
        .map_err(|_| DeployServiceError::Internal("invalid idempotency result".to_owned()))?;
    serde_json::from_str(&result_json)
        .map(Some)
        .map_err(|_| DeployServiceError::Internal("invalid idempotency result".to_owned()))
}

async fn lock_site(
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
) -> DeployServiceResult<StoredSite> {
    let row = sqlx::query(
        "SELECT id, organization_id, version, desired_revision_id
         FROM deploy_site
         WHERE tenant_id = $1 AND uuid = $2 AND deleted_at IS NULL
         FOR UPDATE",
    )
    .bind(command.tenant_id)
    .bind(&command.site_uuid)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("lock site composition", error))?
    .ok_or_else(|| DeployServiceError::not_found("site not found"))?;
    let site = StoredSite {
        id: row
            .try_get("id")
            .map_err(|_| DeployServiceError::Internal("invalid site record".to_owned()))?,
        organization_id: row.try_get("organization_id").unwrap_or(0),
        version: row
            .try_get("version")
            .map_err(|_| DeployServiceError::Internal("invalid site version".to_owned()))?,
        desired_revision_id: row.try_get("desired_revision_id").ok(),
    };
    if site.version != command.expected_site_version {
        return Err(DeployServiceError::conflict(
            "site composition version changed; refresh and retry",
        ));
    }
    Ok(site)
}

async fn reserve_site_version(
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
) -> DeployServiceResult<()> {
    let result = sqlx::query(
        "UPDATE deploy_site SET version = version + 1, updated_at = CAST($3 AS TIMESTAMPTZ)
         WHERE id = $1 AND version = $2",
    )
    .bind(site.id)
    .bind(command.expected_site_version)
    .bind(&command.generated_at)
    .execute(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("reserve site composition version", error))?;
    if result.rows_affected() != 1 {
        return Err(DeployServiceError::conflict(
            "site composition version changed; refresh and retry",
        ));
    }
    Ok(())
}

async fn load_targets(
    transaction: &mut Transaction<'static, Postgres>,
    tenant_id: i64,
    environment: &str,
) -> DeployServiceResult<Vec<StoredTarget>> {
    let rows = sqlx::query(
        "SELECT id, uuid, node_uuid, tenant_scope_hash
         FROM deploy_web_node_target
         WHERE tenant_id = $1 AND environment = $2 AND status = 'ACTIVE'
           AND deleted_at IS NULL
         ORDER BY uuid FOR UPDATE",
    )
    .bind(tenant_id)
    .bind(environment)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("load Web Node targets", error))?;
    if rows.is_empty() {
        return Err(DeployServiceError::conflict(
            "no active Web Node target exists for the requested environment",
        ));
    }
    rows.into_iter()
        .map(|row| {
            Ok(StoredTarget {
                id: row.try_get("id").map_err(|_| invalid_stored_target())?,
                uuid: row.try_get("uuid").map_err(|_| invalid_stored_target())?,
                node_uuid: row
                    .try_get("node_uuid")
                    .map_err(|_| invalid_stored_target())?,
                tenant_scope_hash: row
                    .try_get("tenant_scope_hash")
                    .map_err(|_| invalid_stored_target())?,
            })
        })
        .collect()
}

fn consistent_tenant_scope_hash(targets: &[StoredTarget]) -> DeployServiceResult<String> {
    let scope = targets
        .first()
        .map(|target| target.tenant_scope_hash.clone())
        .ok_or_else(|| DeployServiceError::conflict("no active Web Node target"))?;
    if scope.len() != 64
        || !scope
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || targets
            .iter()
            .any(|target| target.tenant_scope_hash != scope)
    {
        return Err(DeployServiceError::conflict(
            "Web Node targets have inconsistent tenant scope",
        ));
    }
    Ok(scope)
}

async fn delete_current_composition(
    transaction: &mut Transaction<'static, Postgres>,
    site_id: i64,
) -> DeployServiceResult<()> {
    for (table, context) in [
        ("deploy_site_binding", "delete site bindings"),
        ("deploy_site_variant_rule", "delete site variant rules"),
        ("deploy_site_mount", "delete site mounts"),
        ("deploy_site_variant", "delete site variants"),
        ("deploy_site_resource", "delete site resources"),
    ] {
        let query = format!("DELETE FROM {table} WHERE site_id = $1");
        sqlx::query(&query)
            .bind(site_id)
            .execute(&mut **transaction)
            .await
            .map_err(|error| composition_store_error(context, error))?;
    }
    Ok(())
}

async fn insert_resources(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
) -> DeployServiceResult<BTreeMap<String, StoredResource>> {
    let mut stored = BTreeMap::new();
    for resource in &command.resources {
        let id = next_id(repository.id_generator())?;
        let uuid = new_uuid();
        let capabilities = serde_json::to_string(&resource.capabilities).map_err(|_| {
            DeployServiceError::Internal("serialize capabilities failed".to_owned())
        })?;
        sqlx::query(
            "INSERT INTO deploy_site_resource (
                id, uuid, tenant_id, organization_id, site_id, resource_key, provider_type,
                provider_resource_uuid, provider_contract_version, capabilities_json, status,
                last_validated_at, metadata, created_by, updated_by, created_at, updated_at, version
             ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,CAST($10 AS JSONB),'VALID',
                CAST($11 AS TIMESTAMPTZ),'{}',$12,$12,CAST($11 AS TIMESTAMPTZ),
                CAST($11 AS TIMESTAMPTZ),1)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(site.organization_id)
        .bind(site.id)
        .bind(&resource.key)
        .bind(provider_type_name(resource.provider_type))
        .bind(&resource.provider_resource_uuid)
        .bind(&resource.provider_contract_version)
        .bind(&capabilities)
        .bind(&command.generated_at)
        .bind(command.actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert site resource", error))?;
        stored.insert(resource.key.clone(), StoredResource { id, uuid });
    }
    Ok(stored)
}

async fn insert_variants(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
) -> DeployServiceResult<BTreeMap<String, StoredVariant>> {
    let mut stored = BTreeMap::new();
    for variant in &command.request.variants {
        let id = next_id(repository.id_generator())?;
        let uuid = new_uuid();
        sqlx::query(
            "INSERT INTO deploy_site_variant (
                id,uuid,tenant_id,site_id,variant_key,label,client_class,is_default,priority,
                status,metadata,created_by,updated_by,created_at,updated_at,version
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'ACTIVE','{}',$10,$10,
                CAST($11 AS TIMESTAMPTZ),CAST($11 AS TIMESTAMPTZ),1)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(site.id)
        .bind(&variant.key)
        .bind(&variant.label)
        .bind(client_class_name(variant.client_class))
        .bind(variant.key == command.request.default_variant_key)
        .bind(i32::from(variant.priority))
        .bind(command.actor_id)
        .bind(&command.generated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert site variant", error))?;
        stored.insert(variant.key.clone(), StoredVariant { id, uuid });
    }
    Ok(stored)
}

async fn insert_variant_rules(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
    variants: &BTreeMap<String, StoredVariant>,
) -> DeployServiceResult<Vec<RuntimeVariantRule>> {
    let mut runtime = Vec::with_capacity(command.request.variant_rules.len());
    for rule in &command.request.variant_rules {
        let id = next_id(repository.id_generator())?;
        let uuid = new_uuid();
        let variant = variants
            .get(&rule.target_variant_key)
            .ok_or_else(|| DeployServiceError::validation("unknown variant rule target"))?;
        let (rule_type, match_value, matcher) = match &rule.matcher {
            SiteVariantRuleMatcher::PathPrefix { path_prefix } => (
                "PATH_PREFIX",
                path_prefix.clone(),
                RuntimeVariantRuleMatcher::PathPrefix {
                    path_prefix: path_prefix.clone(),
                },
            ),
            SiteVariantRuleMatcher::ClientClass { client_class } => (
                "CLIENT_CLASS",
                client_class_name(*client_class).to_owned(),
                RuntimeVariantRuleMatcher::ClientClass {
                    client_class: runtime_client_class(*client_class),
                },
            ),
        };
        sqlx::query(
            "INSERT INTO deploy_site_variant_rule (
                id,uuid,tenant_id,site_id,rule_key,target_variant_id,rule_type,match_value,
                priority,status,created_by,updated_by,created_at,updated_at,version
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'ACTIVE',$10,$10,
                CAST($11 AS TIMESTAMPTZ),CAST($11 AS TIMESTAMPTZ),1)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(site.id)
        .bind(&rule.key)
        .bind(variant.id)
        .bind(rule_type)
        .bind(match_value)
        .bind(i32::from(rule.priority))
        .bind(command.actor_id)
        .bind(&command.generated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert site variant rule", error))?;
        runtime.push(RuntimeVariantRule {
            rule_uuid: uuid,
            variant_uuid: variant.uuid.clone(),
            priority: rule.priority,
            matcher,
        });
    }
    Ok(runtime)
}

async fn insert_mounts(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
    variants: &BTreeMap<String, StoredVariant>,
    resources: &BTreeMap<String, StoredResource>,
) -> DeployServiceResult<Vec<RuntimeMount>> {
    let mut runtime = Vec::with_capacity(command.request.mounts.len());
    for mount in &command.request.mounts {
        let id = next_id(repository.id_generator())?;
        let uuid = new_uuid();
        let variant = variants
            .get(&mount.variant_key)
            .ok_or_else(|| DeployServiceError::validation("unknown mount variant"))?;
        let resource = resources
            .get(&mount.resource_key)
            .ok_or_else(|| DeployServiceError::validation("unknown mount resource"))?;
        let index_files = serde_json::to_string(&mount.index_files)
            .map_err(|_| DeployServiceError::Internal("serialize index files failed".to_owned()))?;
        sqlx::query(
            "INSERT INTO deploy_site_mount (
                id,uuid,tenant_id,site_id,mount_key,variant_id,resource_id,path_prefix,
                resource_subpath,mount_mode,handler_type,index_files_json,spa_fallback_path,
                priority,status,created_by,updated_by,created_at,updated_at,version
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,CAST($12 AS JSONB),$13,$14,
                'ACTIVE',$15,$15,CAST($16 AS TIMESTAMPTZ),CAST($16 AS TIMESTAMPTZ),1)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(site.id)
        .bind(&mount.key)
        .bind(variant.id)
        .bind(resource.id)
        .bind(&mount.path_prefix)
        .bind(&mount.resource_subpath)
        .bind(mount_mode_name(mount.mode))
        .bind(mount_handler_name(mount.handler))
        .bind(&index_files)
        .bind(mount.spa_fallback.as_deref())
        .bind(i32::from(mount.priority))
        .bind(command.actor_id)
        .bind(&command.generated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert site mount", error))?;
        runtime.push(RuntimeMount {
            mount_uuid: uuid,
            variant_uuid: variant.uuid.clone(),
            path_prefix: mount.path_prefix.clone(),
            resource_uuid: resource.uuid.clone(),
            handler: runtime_handler(mount.handler),
            translation: RuntimeMountTranslation {
                mode: runtime_mount_mode(mount.mode),
                resource_subpath: mount.resource_subpath.clone(),
            },
            index_files: mount.index_files.clone(),
            spa_fallback: mount.spa_fallback.clone(),
        });
    }
    Ok(runtime)
}

async fn insert_bindings(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
    variants: &BTreeMap<String, StoredVariant>,
) -> DeployServiceResult<Vec<RuntimeBinding>> {
    let mut domains: BTreeMap<String, StoredDomain> = BTreeMap::new();
    let mut runtime = Vec::with_capacity(command.request.bindings.len());
    for binding in &command.request.bindings {
        let domain = if let Some(domain) = domains.get(&binding.domain_id) {
            domain.clone()
        } else {
            let domain =
                load_verified_domain(transaction, command.tenant_id, &binding.domain_id).await?;
            domains.insert(binding.domain_id.clone(), domain.clone());
            domain
        };
        let id = next_id(repository.id_generator())?;
        let uuid = new_uuid();
        let persisted = binding_action(&binding.action, variants)?;
        sqlx::query(
            "INSERT INTO deploy_site_binding (
                id,uuid,tenant_id,organization_id,site_id,binding_key,domain_id,hostname_ascii,
                environment,path_prefix,action_type,default_variant_id,forced_variant_id,
                redirect_scheme,redirect_hostname,redirect_path_prefix,redirect_status_code,
                preserve_path,preserve_query,status,verified_at,activated_at,created_by,updated_by,
                created_at,updated_at,version
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,
                $18,$19,'ACTIVE',CAST($20 AS TIMESTAMPTZ),CAST($20 AS TIMESTAMPTZ),$21,$21,
                CAST($20 AS TIMESTAMPTZ),CAST($20 AS TIMESTAMPTZ),1)",
        )
        .bind(id)
        .bind(&uuid)
        .bind(command.tenant_id)
        .bind(site.organization_id)
        .bind(site.id)
        .bind(&binding.key)
        .bind(domain.id)
        .bind(&domain.hostname)
        .bind(command.request.environment.as_str())
        .bind(&binding.path_prefix)
        .bind(persisted.action_type)
        .bind(persisted.default_variant_id)
        .bind(persisted.forced_variant_id)
        .bind(persisted.redirect_scheme)
        .bind(persisted.redirect_hostname)
        .bind(persisted.redirect_path_prefix)
        .bind(persisted.redirect_status_code)
        .bind(persisted.preserve_path)
        .bind(persisted.preserve_query)
        .bind(&command.generated_at)
        .bind(command.actor_id)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert site binding", error))?;
        runtime.push(RuntimeBinding {
            binding_uuid: uuid,
            hostname: domain.hostname,
            path_prefix: binding.path_prefix.clone(),
            action: persisted.runtime_action,
        });
    }
    Ok(runtime)
}

#[derive(Clone, Debug)]
struct PersistedBindingAction {
    action_type: &'static str,
    default_variant_id: Option<i64>,
    forced_variant_id: Option<i64>,
    redirect_scheme: Option<&'static str>,
    redirect_hostname: Option<String>,
    redirect_path_prefix: Option<String>,
    redirect_status_code: Option<i32>,
    preserve_path: bool,
    preserve_query: bool,
    runtime_action: RuntimeBindingAction,
}

fn binding_action(
    action: &SiteBindingAction,
    variants: &BTreeMap<String, StoredVariant>,
) -> DeployServiceResult<PersistedBindingAction> {
    match action {
        SiteBindingAction::Serve {
            default_variant_key,
            forced_variant_key,
        } => {
            let default = optional_variant(default_variant_key.as_deref(), variants)?;
            let forced = optional_variant(forced_variant_key.as_deref(), variants)?;
            Ok(PersistedBindingAction {
                action_type: "SERVE",
                default_variant_id: default.map(|variant| variant.id),
                forced_variant_id: forced.map(|variant| variant.id),
                redirect_scheme: None,
                redirect_hostname: None,
                redirect_path_prefix: None,
                redirect_status_code: None,
                preserve_path: true,
                preserve_query: true,
                runtime_action: RuntimeBindingAction::serve(
                    default.map(|variant| variant.uuid.clone()),
                    forced.map(|variant| variant.uuid.clone()),
                ),
            })
        }
        SiteBindingAction::Redirect {
            status_code,
            scheme,
            hostname,
            path_prefix,
            preserve_path,
            preserve_query,
        } => Ok(PersistedBindingAction {
            action_type: "REDIRECT",
            default_variant_id: None,
            forced_variant_id: None,
            redirect_scheme: Some(redirect_scheme_name(*scheme)),
            redirect_hostname: Some(hostname.clone()),
            redirect_path_prefix: Some(path_prefix.clone()),
            redirect_status_code: Some(i32::from(*status_code)),
            preserve_path: *preserve_path,
            preserve_query: *preserve_query,
            runtime_action: RuntimeBindingAction::Redirect {
                status_code: *status_code,
                scheme: match scheme {
                    SiteRedirectScheme::Http => RuntimeRedirectScheme::Http,
                    SiteRedirectScheme::Https => RuntimeRedirectScheme::Https,
                },
                hostname: hostname.clone(),
                path_prefix: path_prefix.clone(),
                preserve_path: *preserve_path,
                preserve_query: *preserve_query,
            },
        }),
    }
}

fn optional_variant<'a>(
    key: Option<&str>,
    variants: &'a BTreeMap<String, StoredVariant>,
) -> DeployServiceResult<Option<&'a StoredVariant>> {
    key.map(|key| {
        variants
            .get(key)
            .ok_or_else(|| DeployServiceError::validation("binding references unknown variant"))
    })
    .transpose()
}

async fn load_verified_domain(
    transaction: &mut Transaction<'static, Postgres>,
    tenant_id: i64,
    domain_uuid: &str,
) -> DeployServiceResult<StoredDomain> {
    let row = sqlx::query(
        "SELECT id, hostname_ascii FROM deploy_domain
         WHERE tenant_id = $1 AND uuid = $2
           AND verification_status = 'VERIFIED' AND status = 'ACTIVE'
           AND deleted_at IS NULL",
    )
    .bind(tenant_id)
    .bind(domain_uuid)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("load site binding domain", error))?
    .ok_or_else(|| DeployServiceError::not_found("domain not found for site"))?;
    Ok(StoredDomain {
        id: row
            .try_get("id")
            .map_err(|_| DeployServiceError::Internal("invalid domain record".to_owned()))?,
        hostname: row
            .try_get("hostname_ascii")
            .map_err(|_| DeployServiceError::Internal("invalid domain record".to_owned()))?,
    })
}

fn runtime_resource(
    resource: &ValidatedContentProviderResource,
    stored: &BTreeMap<String, StoredResource>,
) -> DeployServiceResult<RuntimeResource> {
    let stored = stored
        .get(&resource.key)
        .ok_or_else(|| DeployServiceError::Internal("stored resource is missing".to_owned()))?;
    Ok(RuntimeResource {
        resource_uuid: stored.uuid.clone(),
        provider: RuntimeProviderReference {
            provider_type: resource.provider_type,
            provider_resource_uuid: resource.provider_resource_uuid.clone(),
            provider_contract_version: resource.provider_contract_version.clone(),
        },
        capabilities: resource.capabilities.clone(),
    })
}

async fn next_revision_number(
    transaction: &mut Transaction<'static, Postgres>,
    site_id: i64,
) -> DeployServiceResult<i64> {
    let row = sqlx::query(
        "SELECT COALESCE(MAX(revision_no), 0) + 1 AS revision_no
         FROM deploy_site_revision WHERE site_id = $1",
    )
    .bind(site_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("reserve site revision number", error))?;
    row.try_get("revision_no")
        .map_err(|_| DeployServiceError::Internal("invalid site revision number".to_owned()))
}

#[allow(clippy::too_many_arguments)]
async fn insert_revision(
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site: &StoredSite,
    revision_id: i64,
    revision_uuid: &str,
    revision_number: i64,
    source_config_version: i64,
    descriptor: &serde_json::Value,
    descriptor_sha256: &str,
) -> DeployServiceResult<()> {
    let descriptor_json = serde_json::to_string(descriptor)
        .map_err(|_| DeployServiceError::Internal("serialize site revision failed".to_owned()))?;
    sqlx::query(
        "INSERT INTO deploy_site_revision (
            id,uuid,tenant_id,organization_id,site_id,revision_no,environment,
            descriptor_schema_version,descriptor_json,descriptor_sha256,compiler_version,
            source_config_version,idempotency_key,request_sha256,result_json,validation_status,
            validation_report_json,supersedes_revision_id,created_by,created_at
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,CAST($9 AS JSONB),$10,$11,$12,$13,$14,
            '{}','VALID','{}',$15,$16,CAST($17 AS TIMESTAMPTZ))",
    )
    .bind(revision_id)
    .bind(revision_uuid)
    .bind(command.tenant_id)
    .bind(site.organization_id)
    .bind(site.id)
    .bind(revision_number)
    .bind(command.request.environment.as_str())
    .bind(WEBSITE_RUNTIME_SCHEMA_VERSION)
    .bind(&descriptor_json)
    .bind(descriptor_sha256)
    .bind(DESCRIPTOR_COMPILER_VERSION)
    .bind(source_config_version)
    .bind(&command.idempotency_key)
    .bind(&command.request_sha256)
    .bind(site.desired_revision_id)
    .bind(command.actor_id)
    .bind(&command.generated_at)
    .execute(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("insert site revision", error))?;
    Ok(())
}

async fn update_site_revision_pointers(
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site_id: i64,
    default_variant_id: i64,
    revision_id: i64,
) -> DeployServiceResult<()> {
    sqlx::query(
        "UPDATE deploy_site SET default_variant_id = $2, desired_revision_id = $3,
            updated_at = CAST($4 AS TIMESTAMPTZ) WHERE id = $1",
    )
    .bind(site_id)
    .bind(default_variant_id)
    .bind(revision_id)
    .bind(&command.generated_at)
    .execute(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("update site revision pointers", error))?;
    Ok(())
}

async fn load_other_descriptors(
    transaction: &mut Transaction<'static, Postgres>,
    tenant_id: i64,
    excluded_site_id: i64,
    environment: &str,
) -> DeployServiceResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        "SELECT CAST(r.descriptor_json AS TEXT) AS descriptor_json
         FROM deploy_site s
         INNER JOIN deploy_site_revision r ON r.id = s.desired_revision_id
         WHERE s.tenant_id = $1 AND s.id <> $2 AND s.status = 1
           AND s.deleted_at IS NULL AND r.environment = $3
         ORDER BY s.uuid",
    )
    .bind(tenant_id)
    .bind(excluded_site_id)
    .bind(environment)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("load desired site descriptors", error))?;
    rows.into_iter()
        .map(|row| {
            let text: String = row.try_get("descriptor_json").map_err(|_| {
                DeployServiceError::Internal("invalid desired site descriptor".to_owned())
            })?;
            serde_json::from_str(&text).map_err(|_| {
                DeployServiceError::Internal("invalid desired site descriptor".to_owned())
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
async fn insert_runtime_assignments(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    revision_id: i64,
    targets: &[StoredTarget],
    descriptors: &[serde_json::Value],
) -> DeployServiceResult<Vec<SiteRuntimeAssignmentResponse>> {
    let desired_state_sha256 = canonical_sha256_excluding_field(
        &serde_json::json!({"descriptors": descriptors}),
        "__no_excluded_field",
    )
    .map_err(|_| DeployServiceError::Internal("hash desired runtime state failed".to_owned()))?;
    let mut responses = Vec::with_capacity(targets.len());
    for target in targets {
        let generation = next_target_generation(transaction, target.id).await?;
        if generation > MAXIMUM_RUNTIME_GENERATION {
            return Err(DeployServiceError::conflict(
                "runtime assignment generation is exhausted",
            ));
        }
        let snapshot_uuid = new_uuid();
        let compiled = compile_runtime_set(RuntimeSetCompilationInput {
            snapshot_uuid: snapshot_uuid.clone(),
            node_uuid: target.node_uuid.clone(),
            environment: runtime_environment(command.request.environment),
            generation: generation as u64,
            generated_at: command.generated_at.clone(),
            maximum_sites: MAXIMUM_RUNTIME_SITES,
            descriptors: descriptors.to_vec(),
        })
        .map_err(|error| DeployServiceError::validation(error.to_string()))?;
        let runtime_set_json = serde_json::to_string(&compiled.snapshot)
            .map_err(|_| DeployServiceError::Internal("serialize runtime set failed".to_owned()))?;
        let runtime_set_bytes = runtime_set_size_bytes(&compiled.snapshot)
            .map_err(|error| DeployServiceError::validation(error.to_string()))?
            as i64;
        let assignment_id = next_id(repository.id_generator())?;
        let assignment_uuid = new_uuid();
        sqlx::query(
            "INSERT INTO deploy_runtime_assignment (
                id,uuid,tenant_id,node_target_id,trigger_site_revision_id,generation,
                snapshot_uuid,snapshot_sha256,desired_state_sha256,runtime_set_json,
                runtime_set_bytes,publish_status,attempt_count,created_at,updated_at,version
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,CAST($10 AS JSONB),$11,'PENDING',0,
                CAST($12 AS TIMESTAMPTZ),CAST($12 AS TIMESTAMPTZ),1)",
        )
        .bind(assignment_id)
        .bind(&assignment_uuid)
        .bind(command.tenant_id)
        .bind(target.id)
        .bind(revision_id)
        .bind(generation)
        .bind(&snapshot_uuid)
        .bind(&compiled.snapshot_sha256)
        .bind(&desired_state_sha256)
        .bind(&runtime_set_json)
        .bind(runtime_set_bytes)
        .bind(&command.generated_at)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("insert runtime assignment", error))?;
        sqlx::query(
            "UPDATE deploy_runtime_assignment SET publish_status = 'SUPERSEDED',
                lease_owner = NULL, lease_expires_at = NULL,
                updated_at = CAST($1 AS TIMESTAMPTZ), version = version + 1
             WHERE node_target_id = $2 AND generation < $3
               AND publish_status <> 'SUPERSEDED'",
        )
        .bind(&command.generated_at)
        .bind(target.id)
        .bind(generation)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("supersede runtime assignments", error))?;
        responses.push(SiteRuntimeAssignmentResponse {
            target_id: target.uuid.clone(),
            assignment_id: assignment_uuid,
            generation: generation.to_string(),
            status: "PENDING".to_owned(),
        });
    }
    Ok(responses)
}

async fn next_target_generation(
    transaction: &mut Transaction<'static, Postgres>,
    target_id: i64,
) -> DeployServiceResult<i64> {
    let row = sqlx::query(
        "SELECT COALESCE(MAX(generation), 0) + 1 AS generation
         FROM deploy_runtime_assignment WHERE node_target_id = $1",
    )
    .bind(target_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("reserve runtime generation", error))?;
    row.try_get("generation")
        .map_err(|_| DeployServiceError::Internal("invalid runtime generation".to_owned()))
}

async fn persist_command_result(
    transaction: &mut Transaction<'static, Postgres>,
    revision_id: i64,
    response: &SiteCompositionResponse,
) -> DeployServiceResult<()> {
    let result_json = serde_json::to_string(response).map_err(|_| {
        DeployServiceError::Internal("serialize composition result failed".to_owned())
    })?;
    sqlx::query("UPDATE deploy_site_revision SET result_json = CAST($2 AS JSONB) WHERE id = $1")
        .bind(revision_id)
        .bind(result_json)
        .execute(&mut **transaction)
        .await
        .map_err(|error| composition_store_error("store composition result", error))?;
    Ok(())
}

async fn insert_composition_audit(
    repository: &DeployRepository,
    transaction: &mut Transaction<'static, Postgres>,
    command: &ReplaceSiteCompositionCommand,
    site_id: i64,
    revision_id: i64,
) -> DeployServiceResult<()> {
    let audit_id = next_id(repository.id_generator())?;
    let audit_uuid = new_uuid();
    let metadata = serde_json::json!({
        "revisionId": revision_id.to_string(),
        "requestSha256": command.request_sha256,
        "environment": command.request.environment.as_str(),
    })
    .to_string();
    sqlx::query(
        "INSERT INTO deploy_audit_log (
            id,uuid,tenant_id,organization_id,operator_id,operator_type,action,target_type,
            target_id,target_uuid,metadata,created_at
         ) VALUES ($1,$2,$3,$4,$5,'USER','sites.composition.update','site',$6,$7,
            CAST($8 AS JSONB),CAST($9 AS TIMESTAMPTZ))",
    )
    .bind(audit_id)
    .bind(audit_uuid)
    .bind(command.tenant_id)
    .bind(command.organization_id)
    .bind(command.actor_id)
    .bind(site_id)
    .bind(&command.site_uuid)
    .bind(metadata)
    .bind(&command.generated_at)
    .execute(&mut **transaction)
    .await
    .map_err(|error| composition_store_error("insert site composition audit", error))?;
    Ok(())
}

fn runtime_environment(
    environment: sdkwork_deploy_contract::SiteEnvironment,
) -> RuntimeEnvironment {
    match environment {
        sdkwork_deploy_contract::SiteEnvironment::Development => RuntimeEnvironment::Development,
        sdkwork_deploy_contract::SiteEnvironment::Test => RuntimeEnvironment::Test,
        sdkwork_deploy_contract::SiteEnvironment::Staging => RuntimeEnvironment::Staging,
        sdkwork_deploy_contract::SiteEnvironment::Production => RuntimeEnvironment::Production,
    }
}

fn provider_type_name(provider_type: RuntimeProviderType) -> &'static str {
    match provider_type {
        RuntimeProviderType::Drive => "DRIVE",
        RuntimeProviderType::Knowledgebase => "KNOWLEDGEBASE",
    }
}

fn client_class_name(client_class: SiteClientClass) -> &'static str {
    match client_class {
        SiteClientClass::Desktop => "DESKTOP",
        SiteClientClass::Mobile => "MOBILE",
        SiteClientClass::Tablet => "TABLET",
        SiteClientClass::Tv => "TV",
        SiteClientClass::Bot => "BOT",
        SiteClientClass::Other => "OTHER",
    }
}

fn runtime_client_class(client_class: SiteClientClass) -> RuntimeClientClass {
    match client_class {
        SiteClientClass::Desktop => RuntimeClientClass::Desktop,
        SiteClientClass::Mobile => RuntimeClientClass::Mobile,
        SiteClientClass::Tablet => RuntimeClientClass::Tablet,
        SiteClientClass::Tv => RuntimeClientClass::Tv,
        SiteClientClass::Bot => RuntimeClientClass::Bot,
        SiteClientClass::Other => RuntimeClientClass::Other,
    }
}

fn mount_mode_name(mode: SiteMountMode) -> &'static str {
    match mode {
        SiteMountMode::Root => "ROOT",
        SiteMountMode::Alias => "ALIAS",
    }
}

fn runtime_mount_mode(mode: SiteMountMode) -> RuntimeMountMode {
    match mode {
        SiteMountMode::Root => RuntimeMountMode::Root,
        SiteMountMode::Alias => RuntimeMountMode::Alias,
    }
}

fn mount_handler_name(handler: SiteMountHandler) -> &'static str {
    match handler {
        SiteMountHandler::Static => "STATIC",
        SiteMountHandler::Spa => "SPA",
        SiteMountHandler::Wiki => "WIKI",
    }
}

fn runtime_handler(handler: SiteMountHandler) -> RuntimeHandler {
    match handler {
        SiteMountHandler::Static => RuntimeHandler::Static,
        SiteMountHandler::Spa => RuntimeHandler::Spa,
        SiteMountHandler::Wiki => RuntimeHandler::Wiki,
    }
}

fn redirect_scheme_name(scheme: SiteRedirectScheme) -> &'static str {
    match scheme {
        SiteRedirectScheme::Http => "http",
        SiteRedirectScheme::Https => "https",
    }
}

fn invalid_stored_target() -> DeployServiceError {
    DeployServiceError::Internal("invalid Web Node target".to_owned())
}

fn composition_store_error(context: &str, error: sqlx::Error) -> DeployServiceError {
    tracing::error!(error = %error, "{context}");
    match &error {
        sqlx::Error::Database(database) if database.is_unique_violation() => {
            DeployServiceError::conflict("site composition conflicts with current state")
        }
        _ => DeployServiceError::Internal(format!("{context} failed")),
    }
}
