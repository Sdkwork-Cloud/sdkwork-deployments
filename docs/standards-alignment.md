# Standards Alignment

SDKWork Deploy standards alignment for `sdkwork-deployments`, updated 2026-07-23.

## Integrated Frameworks

| Framework or contract | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | Auth layers, route manifests, `WebRequestContext`, success/problem response mapping |
| `sdkwork-database` | Integrated | One PostgreSQL/SQLite database contract, lifecycle host, materialization and drift validation |
| `sdkwork-utils-rust` | Integrated | API envelopes, pagination, parsing, hashing, and shared utilities |
| Deploy App/Backend SDK families | Generated and buildable | Owner-only sdkgen inputs, family manifests, composed TypeScript facades, generated transports |
| Drive App/Internal SDKs | Integrated | WebsiteRoot create/reuse, exact Internal revalidation, Node-scoped event-channel registration/renewal, and Drive-backed artifact uploads |
| Knowledgebase Internal SDK | Integrated | Exact ACTIVE canonical WikiPublication validation and bounded capability projection |
| Web Internal SDK | Integrated | Immutable runtime-set publication with per-attempt ingress-token-file loading |
| `sdkwork-discovery` | Deferred by topology | HTTP-only application gateway; required when RPC services are introduced |

## Live Composition Contract

The active mutation is `sites.composition.update` on
`PUT /app/v3/api/sites/{siteId}/composition`. It requires dual-token authentication,
`deploy.sites.write`, `If-Match`, and `Idempotency-Key`.

Provider calls complete before database locking. PostgreSQL and SQLite then use the same atomic
sequence: idempotency replay check, tenant Site lock/version check, target validation, normalized
composition replacement, descriptor compilation, immutable SiteRevision insert,
`desired_revision_id` update, complete runtime assignment insert, replay result, and audit commit.
The Site version is a decimal string. `current_revision_id` is reserved for verified Web
observation/quorum and is not advanced by the composition transaction.

Ordinary Drive and Knowledgebase content changes never call this mutation and do not create
`deploy_release`, `deploy_deployment`, or `deploy_site_revision` records.
The runtime worker registers and renews referenced Drive WebsiteRoot channels through the generated
Drive Internal SDK before Web publication. Drive then delivers ordinary events directly to the
Node-qualified Web callback; Deploy is not the event relay or acknowledgement authority.

## API And SDK Contract

All App and Backend handlers use SDKWork v3:

- success: `{ "code": 0, "data": { "item" | "items" + "pageInfo" }, "traceId": "..." }`;
- error: HTTP 4xx/5xx `application/problem+json` with numeric `code` and `traceId`;
- owner OpenAPI under `apis/`, materialized authority and deterministic `*.sdkgen.json` under the
  owning family, generated transport under `generated/server-openapi`;
- consumers import only `@sdkwork/deploy-app-sdk` or explicit backend-admin
  `@sdkwork/deploy-backend-sdk`.

Backend composition mutation is intentionally absent until an approved trusted operator
credential-delegation or resolved-resource contract exists.

## Runtime And Secret Configuration

Production uses `SDKWORK_DEPLOY_USE_MEMORY_CONTENT_PROVIDER=false` and explicit Drive,
Knowledgebase, and Web Internal API URLs. Ingress credentials are read from projected files under
`/run/secrets/sdkwork/`; values are not stored in environment variables or runtime descriptors.
Drive/Knowledgebase files are read per provider request and the Web file per publication attempt,
which supports atomic rotation without restart.

## Artifact Pipeline Boundary

Drive upload sessions, `deploy_artifact`, `deploy_release`, and `deploy_deployment` remain valid for
Git, package, image, and frozen-bundle workflows. They are not the publication mechanism for a live
WebsiteRoot or WikiPublication. Certificate upload sessions are metadata registration inputs and do
not move private-key custody into ordinary Deploy columns.

## Verification

```powershell
pnpm db:validate
pnpm api:materialize
pnpm api:check
pnpm sdk:generate
node ../sdkwork-specs/tools/check-sdk-standard.mjs --workspace .
node ../sdkwork-specs/tools/check-component-port-bindings.mjs --root . --strict
cargo test -p sdkwork-deploy-runtime-compiler --test knowledgebase_wiki_delivery_contract
cargo test --workspace --offline
```

The focused cross-repository contract test compiles a real Deploy Site and runtime set, activates
the exact bytes in Web Server, executes host/path/device routing through the Knowledgebase provider
adapter and a fake generated-SDK boundary, fails private/unpublished routes closed, and observes a
live content update without changing the SiteRevision, runtime-set generation, or snapshot hash.

## Remaining Product Gates

These are explicit launch scope, not hidden compatibility debt. Authenticated Web
observation/quorum, current-revision advancement, Drive/Wiki live reads, and direct provider-event
processing are implemented. The compiler-to-Wiki execution contract is also verified locally;
production-shaped evidence remains required:

- external public-domain probes, drift dashboards, and multi-node rollout/rollback drills;
- provider-aware cache implementation plus invalidation, private revocation, and freshness evidence;
- certificate secret custody, ACME issue/renew/distribute/hot-activate/SNI verification;
- tenant console, platform admin console, metering, entitlement, abuse, and incident workflows;
- continuous topology evidence that cloud Web workloads cannot activate standalone management authority;
- production PostgreSQL backup/restore, multi-node rollout, load, security, and recovery evidence;
- governed publication of the already generated App/Backend SDK packages.
