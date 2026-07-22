# MIG-2026-0001 Cloud Site Control-Plane Convergence

Status: active prelaunch convergence
Requirement: REQ-2026-0001
Owner: SDKWork Deploy maintainers
Updated: 2026-07-22
Specs: MIGRATION_SPEC.md, DATABASE_SPEC.md, API_SPEC.md, SDK_SPEC.md, DEPLOYMENT_SPEC.md,
RELEASE_SPEC.md, SECURITY_SPEC.md, TEST_SPEC.md

## 1. Scope

Establish `sdkwork-deployments` as the single writable authority for Site composition, domains,
certificate policy, immutable configuration revisions, and desired Web runtime assignments.
Drive remains the file/directory authority, Knowledgebase remains the Wiki publication authority,
and Web Server remains the runtime projection and delivery executor.

The application has not launched and has no production compatibility population to preserve.
Convergence therefore updates the initialization baseline directly and removes obsolete authority
instead of adding backfill columns, dual writes, compatibility shims, or a legacy migration window.
Frozen package/Git artifacts continue to use `deploy_release`; live Drive/Wiki content never does.

## 2. Implemented Prelaunch Baseline

- `deploy_site_resource`, `deploy_site_variant`, `deploy_site_variant_rule`,
  `deploy_site_mount`, and `deploy_site_binding` own normalized Site composition.
- `deploy_site_revision` owns immutable, hash-addressed Web runtime descriptors.
- `deploy_web_node_target` owns Deploy's tenant/environment target inventory.
- `deploy_runtime_assignment` is the durable desired-state/outbox record; it does not replace Web
  Server's delivery projection or Node observation store.
- PostgreSQL and SQLite initialization DDL have the same logical tables, constraints, and indexes.
- Deploy compiles canonical runtime documents and publishes only through the generated Web Internal
  Rust SDK using a rotatable secret-file ingress credential.
- Same desired state is idempotent, generations are monotonic and JSON-safe, retries are bounded,
  and remote receipts must match UUID/hash/generation before publication is committed.

## 3. Remaining Convergence Work

### Stage 1 - Management Contract

Replace opaque `runtimeConfig` authoring with typed Site Resource, Variant, rule, Mount, Binding,
policy, validation, preview, revision, activation, pause, and rollback app/backend API resources.
Update owner OpenAPI documents, regenerate Deploy SDK families, and implement tenant/admin views.

Exit evidence: generated SDK-only clients, bounded pagination, optimistic concurrency, tenant
isolation, route simulation, and API/service/repository transaction tests.

### Stage 2 - Provider Attachment

Resolve Drive `SPACE_ROOT`/`FOLDER` selectors to stable WebsiteRoot UUIDs and Knowledgebase selectors
to the canonical WikiPublication through owner-generated internal SDKs. Persist opaque provider
identity and bounded capabilities only.

Exit evidence: website-Space eligibility, folder confinement, active Wiki eligibility, tenant
scope, idempotent reuse, revocation, and negative tests. No raw HTTP or cross-database reads.

### Stage 3 - Revision And Assignment Orchestration

Within one Deploy transaction, validate optimistic Site state, store a SiteRevision, update desired
state, and enqueue complete assignments for every affected Node. Run the bounded outbox publisher
continuously and reconcile failed or stale assignments.

Exit evidence: PostgreSQL and SQLite parity, concurrent generation conflict tests, crash recovery,
same-state replay, rollback, empty-set removal, and multi-Node assignment evidence.

### Stage 4 - Observation And Single Writer

Add an owner-generated Web observation event/read contract so Deploy can evaluate staged load,
probe, quorum, drift, and rollback without reading `web_*` tables. Remove writable Web Site/domain/
certificate business APIs and authority tables before any public launch.

Exit evidence: Web rejects obsolete writes, one-way projection tests pass, desired/observed drift is
visible, and no compatibility or dual-write code remains.

### Stage 5 - TLS And Production Gates

Complete ACME/custom certificate secret custody, version validation, lossless SNI hot activation,
renewal, rollback, metering, backup/restore, load, security, provider outage, and Kubernetes
multi-Node drills.

Exit evidence: every commercial gate in the PRD has measured production-like evidence. A status
flag or planned renewal row is not accepted as operational proof.

## 4. Rollback Semantics

Configuration rollback creates a new monotonic runtime assignment containing a previously accepted
Site descriptor set. Source rollback remains Drive/Knowledgebase-owned. Certificate rollback uses a
separate TLS snapshot. Database baseline contraction is not a runtime rollback mechanism.

## 5. Human Review Gates

- public API and generated SDK contract changes;
- tenant isolation, global hostname/path claim, and RLS decisions;
- disabling and deleting obsolete Web writable authority;
- certificate secret custody and ACME provider policy;
- destructive data/filesystem operations and production rollout.

## 6. Verification

Required evidence includes database contract validation, real PostgreSQL and SQLite repository
tests, descriptor/runtime-set golden compatibility, generated SDK transport tests, concurrency and
idempotency tests, source-owner negative authorization, Web observation/quorum tests, TLS/SNI
drills, backup/restore, load/security testing, and staged production smoke checks.
