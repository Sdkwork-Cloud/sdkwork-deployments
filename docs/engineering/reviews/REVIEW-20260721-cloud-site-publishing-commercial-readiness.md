# REVIEW-20260721 Cloud Site Publishing Commercial Readiness

Status: conditional-design-approval
Owner: SDKWork Deploy maintainers
Date: 2026-07-21
Requirement: REQ-2026-0001
Decision: ADR-20260721-unified-cloud-site-publishing-control-plane
Specs: CODE_REVIEW_SPEC.md, QUALITY_GATE_SPEC.md, REQUIREMENTS_SPEC.md,
ARCHITECTURE_DECISION_SPEC.md, DATABASE_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md,
PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, TEST_SPEC.md, RELEASE_SPEC.md, MIGRATION_SPEC.md

## 1. Review Scope

This review evaluates whether the cross-repository design can become a commercial cloud publishing
platform based on:

- `sdkwork-deployments` as the control-plane and database authority;
- `sdkwork-web-server` as the HTTP/TLS data plane;
- `sdkwork-drive` Website Spaces and directory resources;
- `sdkwork-knowledgebase` live `sources/raw` Wiki resources;
- multi-domain, path, client Variant, certificate, user-console, admin-console, entitlement,
  metering, security, reliability, and operational requirements.

It reviews design readiness. It does not approve migrations, API breakage, production TLS policy,
commercial prices/SLO contracts, or claim that the implementation already exists.

## 2. Verdict

**Conditionally approved as the target architecture; not ready for production implementation or
commercial launch until the human review and evidence gates below are closed.**

The design now has a coherent ownership model, live-content semantics, deterministic routing,
certificate lifecycle, database target, runtime descriptor, browser request flow, complete user and
admin view inventory, commercial dimensions, and production acceptance plan. The largest historical
design defects are explicitly identified and have migration paths.

## 3. Design Strengths

1. Content authoring is no longer coupled to a Deploy Release. Configuration, content, and TLS have
   separate lifecycle and rollback semantics.
2. Public eligibility is explicit: Drive requires Website Space plus mounted root; Knowledgebase
   requires active Wiki publication plus per-file state.
3. Nginx-style Space/folder/resource/Mount/Binding composition answers the directory-mount question
   without exposing an entire Space automatically.
4. Multi-domain and multi-application device routing are deterministic and explainable, with
   authorization isolated from client classification.
5. The certificate model covers account/order/challenge/version/distribution/observation and keeps
   private keys outside the database and website descriptor.
6. The `WebsiteRuntimeDescriptor` removes database joins/control-plane dependency from the request
   hot path and supports atomic activation and last-known-good service.
7. User, author, tenant-admin, and platform-admin views cover creation, operation, diagnostics,
   recovery, quotas, audit, and incident workflows rather than only a create-site happy path.
8. Commercial boundaries avoid billing duplication: source products meter source work, Deploy meters
   delivery, Commerce owns price/invoice/payment.
9. SLO, security, abuse, privacy, capacity, backup, migration, and release evidence are defined as
   gates rather than assumed from local tests.

## 4. P0 Blocking Findings

### P0-1 Dual Cloud Control-Plane Authority

Web Server currently has overlapping writable `web_site`, `web_domain`, `web_deployment`, and
`web_certificate` state while Deploy owns equivalent `deploy_*` capability. Commercial launch with
two writers would allow route/certificate divergence and unsafe recovery.

Required closure: approve and execute `MIG-2026-0001-cloud-site-control-plane-convergence`, prove a
single writer, shadow-compare active routes/TLS, and disable/retire the old write paths.

### P0-2 Live Wiki Replacement And Realtime Update Path Are Absent

The removed prelaunch `kb_site`/`kb_site_release`/`kb_site_host_binding` model is no longer the
current working authority, but the replacement is still absent. Knowledgebase OpenAPI/database has
no WikiPublication/provider implementation; the current `sources/raw` storage adapter rejects an
existing logical path as immutable; and its implemented outbox emits only ingestion success rather
than public Wiki provider events.

Required closure: approve REQ-2026-0721/ADR/MIG, keep the old release builder absent, implement
stable-node/new-immutable-version updates, canonical WikiPublication/projections, Drive input
events, Knowledgebase output events and internal provider SDK, and prove no dual router remains.

### P0-3 Drive Website Space Contract Does Not Yet Exist

Drive lacks `website` in the type enum and currently has an owner/type uniqueness rule incompatible
with multiple Website projects for one owner.

Required closure: human-review the singleton catalog and dual-engine migration; add Website Space,
WebsiteRoot/generation/sync, SDK/UI/provider/events, and atomic-tree evidence.

### P0-4 Provider And Descriptor Contracts Are Not Implemented

No accepted owner OpenAPI/service contracts currently guarantee Drive directory validate/resolve/
open/events, Knowledgebase Wiki page/asset/navigation/search/events, or Web Server descriptor
ingestion.

Required closure: accept versioned machine contracts, implement owner SDK/service ports, add golden
descriptor fixtures, and certify cross-repository compatibility and last-known-good rollback.
The split-topology source authorities are the generated Drive and Knowledgebase `internal-sdk`
families; raw HTTP/manual auth or descriptor-carried provider endpoints are not substitutes.

### P0-5 Production Certificate Custody And Orchestration Are Incomplete

Web Server has useful bounded ACME/activation primitives, but commercial cloud requires durable ACME
accounts, KMS/Secret Manager custody, DNS-01/wildcard providers, immutable versions, fleet
distribution, served SNI fingerprint convergence, revocation, and renewal drills.

Required closure: approve TLS schema/policy, select CA/provider/KMS, implement Deploy orchestration
with Web execution, and attach real staging plus expiry/failure/rotation evidence.
The current `certificates.renew` operation only writes `renewal_status=planned`, and its OpenAPI
explicitly states that the ACME worker is not online.

### P0-6 Production Security/Abuse/Isolation Evidence Is Missing

Public static/Wiki hosting creates phishing, malware, XSS, domain takeover, cache poisoning,
traversal, quota abuse, and cross-tenant risk. Design controls exist, but evidence does not.

Required closure: threat model, permission/RLS/cache/provider tests, sanitizer and active-asset
policy, abuse/takedown/legal-hold process, external security review, and incident drills.

### P0-7 Commercial Entitlement And Meter Reconciliation Is Not Implemented

Dimensions and ownership are defined, but there is no certified entitlement projection, durable
deduplicated usage feed, Commerce reconciliation, overage behavior, or billing dispute evidence.

Required closure: approve meter definitions, implement replay/dedupe/finalization/export, reconcile
at scale, and obtain Finance/Commerce sign-off.

## 5. P1 Launch-Quality Findings

| Finding | Required closure |
| --- | --- |
| User/admin UI is specified but not built | generated-SDK-backed packages, full async/error/permission E2E, accessibility review |
| Domain verification/takeover lifecycle is not certified | DNS/HTTP proof, global conflict transaction, hold/reclaim, wildcard and IDNA tests |
| Cache/event consistency is not proven | public-to-private priority invalidation, event gap/replay, negative TTL, stale policy, stampede tests |
| Multi-device routing may fragment cache/SEO | domain strategy guidance, Vary/cookie policy, route simulator, bot/canonical behavior tests |
| Search/navigation rebuild needs scale evidence | large Wiki projection/index benchmarks, rebuild/checkpoint, degraded-search behavior |
| Atomic sync cleanup/retention needs evidence | quota reservation, crash/fence/idempotency, rollback, orphan cleanup, legal-hold tests |
| Backup/restore crosses multiple owners | coordinated Deploy/Drive/Knowledgebase/KMS restore and consistency validation |
| Multi-region remains a future tier | region/residency contract, traffic failover, KMS/DNS/CA/provider locality and drills |
| SLOs are targets, not contractual evidence | load/soak, production-like latency/freshness, dashboards, error budgets, credit policy |
| Supply-chain release evidence is not assembled | signed artifacts, SBOM, provenance, dependency pins, staged rollout per service |

## 6. Design Readiness Matrix

| Capability | Design | Implementation | Production evidence | Commercial status |
| --- | --- | --- | --- | --- |
| Ownership/bounded contexts | complete, proposed | partial/legacy conflicts | missing | blocked |
| Database target | complete, proposed | not migrated | missing | blocked |
| Drive Website Space/root/sync | complete, proposed | absent | missing | blocked |
| Live Wiki state/provider | complete, proposed | conflicting release model | missing | blocked |
| Runtime descriptor/routing | complete, proposed | partial Web foundations | missing | blocked |
| Domain/path/Variant | complete, proposed | partial management primitives | missing | blocked |
| TLS/certificate lifecycle | complete target; useful runtime primitives | partial | real fleet evidence missing | blocked |
| User/admin UX | complete view/workflow inventory | not implemented | E2E missing | blocked |
| Security/privacy/abuse | complete requirements | partial foundations | external/drill evidence missing | blocked |
| Observability/SLO | complete signals/targets | partial | dashboards/load/error budget missing | blocked |
| Entitlement/metering | complete ownership/dimensions | absent/partial | reconciliation missing | blocked |
| Release/migration/rollback | complete plans | not executed | staged evidence missing | blocked |

## 7. Definition Of Ready For Implementation

- Product owners accept REQ-2026-0001, Drive REQ-2026-0004, Knowledgebase REQ-2026-0721, and Web
  Server REQ-2026-0060.
- Architecture owners accept the four linked ADRs and resolve old ADR supersession.
- Database owners approve exact table/column/enum/index/RLS/migration contracts.
- API/SDK owners approve operation ownership, compatibility windows, and generation plan.
- Security/Privacy approve threat model, active asset policy, certificate custody, abuse, retention,
  and support access.
- Commerce/Finance approve entitlement/meter ownership and reconciliation contract.
- SRE approve target topology, capacity ceilings, SLO measurement, rollout, rollback, backup, and
  incident runbooks.
- UI owners approve package placement, permission matrix, route/view inventory, and accessibility.

## 8. Release Gates

### Pilot Gate

One region; system/custom exact domains; one-name managed certificate; Drive STATIC/SPA;
WebsiteRoot atomic sync; descriptor activation/rollback; basic logs/metrics/usage; internal tenants;
no unreviewed active assets.

### Wiki Beta Gate

Live Markdown/assets, review/auto-public, navigation/search/SEO, page state and private-transition
tests, Web WIKI handler, device Variants, domain/TLS integration, author/admin UI, backup/rebuild.

### Commercial GA Gate

All P0 closed; entitlement/meter reconciliation; abuse/legal/support; external security and load;
real ACME renewal/distribution; SLO dashboards/error budgets; backup/restore and incident drills;
signed release/SBOM/provenance; staged cohort rollback; documented contractual limitations.

### Enterprise Gate

Multi-region/residency, enterprise SSO/RBAC/approval, audit export, advanced support controls,
regional failover, and certified enterprise SLO. These are not implied by standard GA.

## 9. Required Final Evidence Bundle

- accepted requirements/ADRs and exact migration approvals;
- database/API/SDK/descriptor/provider contracts and compatibility matrix;
- unit/contract/integration/E2E/security/fuzz/load/soak/fault-injection reports;
- tenant isolation, domain takeover, certificate, provider outage, cache revocation, and abuse drills;
- backup/restore/RPO/RTO, rollout/canary/rollback, Web Node drift/upgrade evidence;
- usage dedupe/replay/reconciliation and plan/quota behavior;
- UI accessibility/permission/error-state acceptance for tenant and admin views;
- release artifacts, checksums, signing, SBOM, provenance, changelog, and support/runbook handoff.

## 10. Review Outcome

The target system is now architecturally coherent and commercially scoped. The design can proceed to
human contract approval and phased implementation. It must not be represented as production-ready,
commercially available, or migration-approved until the P0 findings and release gates are closed
with evidence.

Knowledgebase-specific implementation evidence and realtime claim gates are maintained in
`../sdkwork-knowledgebase/docs/engineering/reviews/REVIEW-20260721-live-wiki-deployment-integration-readiness.md`.
