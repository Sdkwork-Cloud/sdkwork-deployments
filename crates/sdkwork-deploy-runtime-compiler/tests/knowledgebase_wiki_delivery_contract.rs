use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use sdkwork_deploy_runtime_compiler::{
    compile_runtime_set, compile_site_revision, RuntimeBinding, RuntimeBindingAction,
    RuntimeClientClass, RuntimeDeliveryPolicy, RuntimeEnvironment, RuntimeHandler, RuntimeLimits,
    RuntimeMount, RuntimeMountMode, RuntimeMountTranslation, RuntimeObservabilityPolicy,
    RuntimeProviderReference, RuntimeProviderType, RuntimeResource, RuntimeResourceCapabilities,
    RuntimeSecurityPolicy, RuntimeSetCompilationInput, RuntimeVariant, RuntimeVariantRule,
    RuntimeVariantRuleMatcher, SiteRuntimeCompilationInput,
};
use sdkwork_knowledgebase_internal_sdk_generated_rust::{
    models::{
        ResolveWikiRouteRequest, WikiPublicPageListData, WikiPublicPageMetadata, WikiPublication,
        WikiRouteResolution,
    },
    SdkworkError,
};
use sdkwork_webserver_contract::provider::WebsiteRequestConditions;
use sdkwork_webserver_core::website_runtime::{
    compile_website_runtime_set_snapshot, WebsiteClientClass, WebsiteClientClassificationSource,
    WebsiteProviderType, WebsiteRuntimeEnvironment, WebsiteRuntimeRegistry,
};
use sdkwork_webserver_delivery_runtime::{
    WebsiteDeliveryContent, WebsiteDeliveryExecutor, WebsiteDeliveryMethod, WebsiteDeliveryOutcome,
    WebsiteDeliveryRequest, WebsiteDeliveryRoutingContext, WebsiteDeliveryScheme,
    WebsiteProviderRegistry,
};
use sdkwork_webserver_knowledgebase_provider::{
    FixedKnowledgebaseWikiSdkClientResolver, KnowledgebaseWikiSdkClient,
    KnowledgebaseWikiWebsiteProvider, KNOWLEDGEBASE_WIKI_PROVIDER_CONTRACT_VERSION,
};

const NODE_UUID: &str = "node-contract-0001";
const TENANT_SCOPE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const DESKTOP_PUBLICATION_UUID: &str = "11111111-1111-4111-8111-111111111501";
const MOBILE_PUBLICATION_UUID: &str = "11111111-1111-4111-8111-111111111502";

#[derive(Clone)]
struct PageState {
    content: Vec<u8>,
    public_version: u64,
}

struct FakeKnowledgebaseSdk {
    pages: Mutex<HashMap<String, PageState>>,
}

impl FakeKnowledgebaseSdk {
    fn new() -> Self {
        Self {
            pages: Mutex::new(HashMap::from([
                (
                    DESKTOP_PUBLICATION_UUID.to_owned(),
                    PageState {
                        content: b"# Desktop Wiki v1".to_vec(),
                        public_version: 1,
                    },
                ),
                (
                    MOBILE_PUBLICATION_UUID.to_owned(),
                    PageState {
                        content: b"# Mobile Wiki".to_vec(),
                        public_version: 1,
                    },
                ),
            ])),
        }
    }

    fn replace_content(&self, publication_uuid: &str, content: &[u8]) {
        let mut pages = self.pages.lock().expect("page state lock");
        let page = pages
            .get_mut(publication_uuid)
            .expect("publication fixture exists");
        page.content = content.to_vec();
        page.public_version += 1;
    }

    fn page(&self, publication_uuid: &str) -> Result<PageState, SdkworkError> {
        self.pages
            .lock()
            .expect("page state lock")
            .get(publication_uuid)
            .cloned()
            .ok_or_else(not_found)
    }
}

#[async_trait]
impl KnowledgebaseWikiSdkClient for FakeKnowledgebaseSdk {
    async fn retrieve_publication(
        &self,
        publication_uuid: &str,
    ) -> Result<WikiPublication, SdkworkError> {
        let page = self.page(publication_uuid)?;
        Ok(WikiPublication {
            publication_uuid: publication_uuid.to_owned(),
            title: "Contract Wiki".to_owned(),
            description: None,
            homepage_source_path: "README.md".to_owned(),
            default_locale: "en-US".to_owned(),
            supported_locales: vec!["en-US".to_owned()],
            navigation_mode: "DIRECTORY".to_owned(),
            theme_key: "sdkwork-wiki".to_owned(),
            theme_version: "theme-v1".to_owned(),
            renderer_policy_version: "renderer-v1".to_owned(),
            search_enabled: true,
            robots_policy: "INDEX_FOLLOW".to_owned(),
            sitemap_enabled: true,
            provider_generation: page.public_version.to_string(),
            navigation_generation: page.public_version.to_string(),
            search_generation: page.public_version.to_string(),
        })
    }

    async fn resolve_route(
        &self,
        publication_uuid: &str,
        request: &ResolveWikiRouteRequest,
    ) -> Result<WikiRouteResolution, SdkworkError> {
        if request.route == "/private/" || request.route == "/unpublished/" {
            return Err(not_found());
        }
        let page = self.page(publication_uuid)?;
        let version = page.public_version.to_string();
        Ok(WikiRouteResolution {
            disposition: "PAGE".to_owned(),
            page: Some(WikiPublicPageMetadata {
                projection_uuid: format!("projection-{version}"),
                canonical_route: request.route.clone(),
                file_kind: "PAGE".to_owned(),
                media_type: "text/markdown".to_owned(),
                size_bytes: page.content.len().to_string(),
                content_sha256: format!(
                    "sha256:{}",
                    sdkwork_utils_rust::sha256_hash(&page.content)
                ),
                title: Some("Guide".to_owned()),
                description: None,
                locale: Some("en-US".to_owned()),
                nav_order: Some(1),
                page_public_version: version.clone(),
                public_updated_at: "2026-07-23T00:00:00Z".to_owned(),
            }),
            content_handle: Some(format!("content-{publication_uuid}-{version}")),
            requested_route: None,
            canonical_route: None,
            status: None,
            page_public_version: None,
        })
    }

    async fn retrieve_content(
        &self,
        publication_uuid: &str,
        _content_handle: &str,
    ) -> Result<Vec<u8>, SdkworkError> {
        Ok(self.page(publication_uuid)?.content)
    }

    async fn list_navigation(
        &self,
        _publication_uuid: &str,
        _locale: Option<&str>,
        _cursor: Option<&str>,
        _page_size: i64,
    ) -> Result<WikiPublicPageListData, SdkworkError> {
        unreachable!("page delivery does not list navigation")
    }

    async fn search_pages(
        &self,
        _publication_uuid: &str,
        _query: &str,
        _locale: Option<&str>,
        _cursor: Option<&str>,
        _page_size: i64,
    ) -> Result<WikiPublicPageListData, SdkworkError> {
        unreachable!("page delivery does not search")
    }
}

#[tokio::test]
async fn deploy_compiler_output_delivers_live_device_specific_wiki_content() {
    let site = compile_site_revision(site_input()).expect("Deploy compiles the Site revision");
    let runtime_set = compile_runtime_set(RuntimeSetCompilationInput {
        snapshot_uuid: "snapshot-contract-0001".to_owned(),
        node_uuid: NODE_UUID.to_owned(),
        environment: RuntimeEnvironment::Production,
        generation: 41,
        generated_at: "2026-07-23T00:00:00Z".to_owned(),
        maximum_sites: 8,
        descriptors: vec![site.descriptor],
    })
    .expect("Deploy compiles the runtime set");
    let runtime_bytes = serde_json::to_vec(&runtime_set.snapshot).expect("serialize runtime set");
    let web_runtime = Arc::new(
        compile_website_runtime_set_snapshot(&runtime_bytes)
            .expect("Web accepts the exact Deploy compiler output"),
    );

    let sdk = Arc::new(FakeKnowledgebaseSdk::new());
    let sdk_client: Arc<dyn KnowledgebaseWikiSdkClient> = sdk.clone();
    let resolver = FixedKnowledgebaseWikiSdkClientResolver::new(TENANT_SCOPE_HASH, sdk_client)
        .expect("tenant-scoped SDK resolver");
    let provider = Arc::new(KnowledgebaseWikiWebsiteProvider::new(Arc::new(resolver)));
    let mut providers = WebsiteProviderRegistry::new();
    providers
        .register_wiki(WebsiteProviderType::Knowledgebase, provider)
        .expect("register Knowledgebase provider");
    providers
        .validate_runtime_set(&web_runtime, 2)
        .await
        .expect("provider validates both Wiki resources before activation");

    let registry = Arc::new(WebsiteRuntimeRegistry::new(
        NODE_UUID,
        WebsiteRuntimeEnvironment::Production,
    ));
    registry
        .activate(web_runtime)
        .expect("activate exact Deploy runtime set");
    let executor = WebsiteDeliveryExecutor::new(registry.clone(), Arc::new(providers));

    let desktop = expect_content(
        executor
            .execute(delivery_request(None, "/portal/guide/"))
            .await
            .expect("desktop Wiki request"),
    );
    assert_eq!(desktop.route.variant_uuid, "variant-desktop");
    assert_eq!(
        desktop.route.provider.provider_resource_uuid,
        DESKTOP_PUBLICATION_UUID
    );
    assert_eq!(read_body(desktop).await, b"# Desktop Wiki v1");

    let mobile = expect_content(
        executor
            .execute(delivery_request(
                Some(WebsiteClientClass::Mobile),
                "/portal/guide/",
            ))
            .await
            .expect("mobile Wiki request"),
    );
    assert_eq!(mobile.route.variant_uuid, "variant-mobile");
    assert_eq!(
        mobile.route.provider.provider_resource_uuid,
        MOBILE_PUBLICATION_UUID
    );
    assert_eq!(read_body(mobile).await, b"# Mobile Wiki");

    for path in ["/portal/private/", "/portal/unpublished/"] {
        let hidden = executor
            .execute(delivery_request(None, path))
            .await
            .expect("private Wiki route fails closed without provider disclosure");
        assert!(matches!(hidden, WebsiteDeliveryOutcome::NotFound));
    }

    sdk.replace_content(DESKTOP_PUBLICATION_UUID, b"# Desktop Wiki v2 live");
    let updated = expect_content(
        executor
            .execute(delivery_request(None, "/portal/guide/"))
            .await
            .expect("updated Wiki request"),
    );
    assert_eq!(updated.route.runtime_set_generation, 41);
    assert_eq!(updated.route.revision_uuid, "revision-contract-0001");
    assert_eq!(updated.public_page_version.as_deref(), Some("2"));
    assert_eq!(read_body(updated).await, b"# Desktop Wiki v2 live");
    assert_eq!(
        registry
            .current()
            .expect("active runtime")
            .snapshot_sha256(),
        runtime_set.snapshot_sha256
    );
}

fn site_input() -> SiteRuntimeCompilationInput {
    SiteRuntimeCompilationInput {
        revision_uuid: "revision-contract-0001".to_owned(),
        site_uuid: "site-contract-0001".to_owned(),
        tenant_scope_hash: TENANT_SCOPE_HASH.to_owned(),
        environment: RuntimeEnvironment::Production,
        generated_at: "2026-07-23T00:00:00Z".to_owned(),
        site_default_variant_uuid: "variant-desktop".to_owned(),
        bindings: vec![RuntimeBinding {
            binding_uuid: "binding-docs".to_owned(),
            hostname: "docs.example.com".to_owned(),
            path_prefix: "/portal".to_owned(),
            action: RuntimeBindingAction::serve(None, None),
        }],
        variants: vec![
            RuntimeVariant {
                variant_uuid: "variant-desktop".to_owned(),
                label: "Desktop Wiki".to_owned(),
            },
            RuntimeVariant {
                variant_uuid: "variant-mobile".to_owned(),
                label: "Mobile Wiki".to_owned(),
            },
        ],
        variant_rules: vec![RuntimeVariantRule {
            rule_uuid: "rule-mobile".to_owned(),
            variant_uuid: "variant-mobile".to_owned(),
            priority: 100,
            matcher: RuntimeVariantRuleMatcher::ClientClass {
                client_class: RuntimeClientClass::Mobile,
            },
        }],
        resources: vec![
            wiki_resource("resource-desktop", DESKTOP_PUBLICATION_UUID),
            wiki_resource("resource-mobile", MOBILE_PUBLICATION_UUID),
        ],
        mounts: vec![
            wiki_mount("mount-desktop", "variant-desktop", "resource-desktop"),
            wiki_mount("mount-mobile", "variant-mobile", "resource-mobile"),
        ],
        delivery_policy: RuntimeDeliveryPolicy {
            provider_timeout_ms: 2_500,
            metadata_cache_ttl_seconds: 60,
            negative_cache_ttl_seconds: 5,
            stale_while_revalidate_seconds: 30,
            maximum_object_bytes: 1_024,
        },
        security_policy: RuntimeSecurityPolicy {
            force_https: false,
            deny_dot_files: true,
            denied_path_prefixes: vec!["/.git".to_owned(), "/.sdkwork".to_owned()],
        },
        limits: RuntimeLimits::default(),
        observability_policy: RuntimeObservabilityPolicy::default(),
    }
}

fn wiki_resource(resource_uuid: &str, publication_uuid: &str) -> RuntimeResource {
    RuntimeResource {
        resource_uuid: resource_uuid.to_owned(),
        provider: RuntimeProviderReference {
            provider_type: RuntimeProviderType::Knowledgebase,
            provider_resource_uuid: publication_uuid.to_owned(),
            provider_contract_version: KNOWLEDGEBASE_WIKI_PROVIDER_CONTRACT_VERSION.to_owned(),
        },
        capabilities: RuntimeResourceCapabilities {
            static_content: true,
            wiki_routes: true,
            wiki_search: true,
            range_requests: false,
        },
    }
}

fn wiki_mount(mount_uuid: &str, variant_uuid: &str, resource_uuid: &str) -> RuntimeMount {
    RuntimeMount {
        mount_uuid: mount_uuid.to_owned(),
        variant_uuid: variant_uuid.to_owned(),
        path_prefix: "/".to_owned(),
        resource_uuid: resource_uuid.to_owned(),
        handler: RuntimeHandler::Wiki,
        translation: RuntimeMountTranslation {
            mode: RuntimeMountMode::Root,
            resource_subpath: "/".to_owned(),
        },
        index_files: Vec::new(),
        spa_fallback: None,
    }
}

fn delivery_request(
    client_class: Option<WebsiteClientClass>,
    path: &str,
) -> WebsiteDeliveryRequest {
    WebsiteDeliveryRequest {
        authority: "docs.example.com".to_owned(),
        path: path.to_owned(),
        scheme: WebsiteDeliveryScheme::Https,
        method: WebsiteDeliveryMethod::Get,
        request_id: format!("request-{}", path.replace('/', "-")),
        trace_id: format!("trace-{}", path.replace('/', "-")),
        routing: WebsiteDeliveryRoutingContext {
            verified_preferred_variant_uuid: None,
            client_class,
            client_classification_source: client_class
                .map(|_| WebsiteClientClassificationSource::ClientHint),
        },
        conditions: WebsiteRequestConditions::default(),
        range: None,
        locale: Some("en-US".to_owned()),
        spa_fallback_eligible: false,
    }
}

fn expect_content(outcome: WebsiteDeliveryOutcome) -> Box<WebsiteDeliveryContent> {
    let WebsiteDeliveryOutcome::Content(content) = outcome else {
        panic!("expected Wiki content")
    };
    content
}

async fn read_body(mut content: Box<WebsiteDeliveryContent>) -> Vec<u8> {
    let mut body = content.body.take().expect("GET response body");
    let mut bytes = Vec::new();
    while let Some(chunk) = body.next_chunk().await.expect("read Wiki body") {
        bytes.extend_from_slice(&chunk);
    }
    bytes
}

fn not_found() -> SdkworkError {
    SdkworkError::HttpStatus {
        status: 404,
        body: "{}".to_owned(),
    }
}
