# REVIEW-20260721 Cloud Site Publishing Commercial Readiness

Status: implementation-active-commercial-evidence-blocked
Owner: SDKWork Deploy maintainers
Date: 2026-07-23
Requirement: REQ-2026-0001
Decisions: ADR-20260721-unified-cloud-site-publishing-control-plane,
ADR-20260723-managed-domain-tls-control-plane (proposed)
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

It reviews the current implementation and production evidence boundary. It does not approve the
production TLS policy, commercial prices/SLO contracts, or a commercial launch claim.

## 2. Verdict

**The target architecture is active; provider-to-desired-runtime, cloud single-writer isolation,
Drive/Wiki public delivery, provider-event processing, device routing, authenticated Node
observations, immutable convergence evidence, strict all-target quorum, and current-revision
advancement are implemented. Commercial launch remains blocked until external public probes, cloud
TLS, UI, commercial operations, and production evidence gates below are closed.**

Portable composition storage, Drive/Knowledgebase owner SDK validation, App composition mutation,
immutable descriptor/runtime-set compilation, desired assignments, generated Deploy SDKs,
Knowledgebase public provider, and Web Wiki delivery foundations now have executable evidence. The
review must not continue to describe those capabilities as absent. The remaining blockers are named
below and must not be hidden behind a general production-readiness claim.

## 3. Design Strengths

1. Content authoring is no longer coupled to a Deploy Release. Configuration, content, and TLS have
   separate lifecycle and rollback semantics.
2. Public eligibility is explicit: Drive requires Website Space plus mounted root; Knowledgebase
   requires active Wiki publication plus per-file state.
3. Nginx-style Space/folder/resource/Mount/Binding composition answers the directory-mount question
   without exposing an entire Space automatically.
4. Multi-domain and multi-application device routing are deterministic and explainable, with
   authorization isolated from client classification.
5. The proposed certificate model covers account/order/challenge/version/distribution/observation
   and keeps private keys outside the database and website descriptor. It is not implemented or
   approved production evidence.
6. The `WebsiteRuntimeDescriptor` removes database joins/control-plane dependency from the request
   hot path and supports atomic activation and last-known-good service.
7. User, author, tenant-admin, and platform-admin views cover creation, operation, diagnostics,
   recovery, quotas, audit, and incident workflows rather than only a create-site happy path.
8. Commercial boundaries avoid billing duplication: source products meter source work, Deploy meters
   delivery, Commerce owns price/invoice/payment.
9. SLO, security, abuse, privacy, capacity, backup, migration, and release evidence are defined as
   gates rather than assumed from local tests.

## 4. P0 Status And Blocking Findings

### P0-1 Cloud Control-Plane Authority Boundary - Closed

Deploy is the sole cloud Site/domain/TLS metadata writer. `cloud.production` packages and starts
only the Website Edge Runtime with management composition disabled; the standalone gateway and its
local `web_site`, `web_domain`, `web_deployment`, and `web_certificate` authority are not present in
the cloud process graph. Standalone data is not imported or dual-written into cloud assignments.

Retained gate: keep artifact/topology tests proving this profile boundary and reject any future
cross-profile data import, shared authority database, or management entrypoint in the cloud image.

### P0-2 Wiki Rendition And Deployed Freshness Evidence Is Incomplete

Knowledgebase now owns one canonical WikiPublication, lifecycle/projection tables, Drive input
processing, typed Internal API, generated Rust/TypeScript SDKs, public route/content/navigation/search
reads, optimistic page publication controls, and output events. Web Server has the generated-SDK Wiki
adapter, browser mapping, durable event checkpoints, duplicate/order/gap fencing, reconciliation,
and route-scoped invalidation. A focused test now feeds real Deploy compiler output into Web
activation and the Knowledgebase adapter, covering host/path/device routing, private/unpublished
failure closure, and live content refresh without a new revision. The remaining public product gate
is complete safe rendition/full-text processing and deployed Site-to-Wiki
freshness/private-revocation evidence.

Required closure: implement the production rendition/sanitizer/full-text chain and execute deployed
end-to-end freshness, provider-outage, and private-revocation tests. Certify the implemented bounded
Web resolution metadata cache under capacity, event-storm, positive/negative lookup, priority
revocation, uncertainty, and stale revalidation scenarios; shared/edge body caching remains a
separate future capability.

### P0-3 Drive Atomic Publication Production Evidence Is Incomplete

Drive Website Space, stable WebsiteRoot root/folder selectors, generated App/Internal SDK contracts,
Deploy create-plus-revalidate integration, and Node/root channel registration plus bounded renewal
exist. Publication is fenced when Drive channel assurance fails, secret rotation forces channel
replacement, and the callback is routed to an exact Node rather than a fleet Service. Web Server generated-SDK delivery, exact
generation/version revalidation, range/condition handling, path confinement, visibility failure,
event checkpoint/reconciliation, and browser mapping are implemented. The remaining gate is a
production-shaped owner-to-edge `ATOMIC_SYNC` drill, mixed-generation prevention under failure,
retention/orphan cleanup, callback-ingress mTLS/source-policy evidence, and end-to-end React bundle evidence.

Required closure: certify atomic-tree switch/rollback under failure and multi-Site root reuse without
tenant/path escape in a deployed topology, including retention and orphan cleanup.

### P0-4 Internal Runtime Convergence Is Closed; Public Activation Proof Is Incomplete

Deploy now validates owner resources before database locking, atomically replaces normalized
composition, creates immutable SiteRevisions, advances `desired_revision_id`, and enqueues complete
runtime-set assignments. Web Server descriptor/runtime-set schema, hash, collision, generation, and
atomic activation foundations have golden compatibility evidence. Web exposes latest authenticated
observations through its owner Internal API; Deploy consumes that method through the generated SDK,
revalidates the full frozen assignment identity, and stores immutable evidence. Web reports
`ACTIVE` only after a bounded node-local `HEAD` probe succeeds in an isolated candidate registry.
Deploy advances `current_revision_id` transactionally only after every frozen target is `ACTIVE` and
the revision remains desired. SQLite integration evidence proves partial quorum does not advance the
pointer and complete quorum does. The compiler-to-Wiki contract test additionally activates the
exact Deploy-produced runtime bytes in Web, executes desktop/mobile Knowledgebase routes, and keeps
the revision, generation, and snapshot hash stable across a live content update.

Required closure: add external public-domain multi-vantage probes, production-shaped multi-node
retry/rollback/drift exercises, and a deployed end-to-end proof from App composition mutation to a
served public response. A published assignment or the focused in-process contract alone remains
insufficient; internal `ACTIVE` is strong node-local activation evidence, not proof of
DNS/TLS/global reachability.

### P0-5 Production Certificate Custody And Orchestration Are Incomplete

Web Server standalone has an executable bounded ACME renewal worker. Its native data plane can also
consume a node-scoped immutable TLS snapshot, validate certificate/key/SAN/validity/fingerprint,
perform exact/wildcard SNI selection, atomically replace Rustls state, and restore the last known
good snapshot. These are local execution primitives, not a cloud certificate control plane.
Deploy App API now performs bounded IDNA normalization and exact DNS TXT token proof before a
domain can become active; a resolver error or missing/mismatched token fails closed. Commercial
cloud still requires durable proof expiry/revalidation and takeover holds, durable ACME accounts,
KMS/Secret Manager custody, DNS-01/wildcard providers, immutable versions, fleet distribution,
authenticated loaded/served observations, public SNI fingerprint convergence, revocation, and
renewal drills.

Required closure: approve ADR-20260723, select CA/DNS/KMS/material-delivery/probe providers,
implement PLAN-2026-0002 through generated owner SDKs, and attach real staging plus
expiry/failure/rotation evidence.
Deploy `certificates.renew` currently records scheduling state only; it is not connected to the
standalone Web worker and therefore is not cloud issuance/distribution evidence.

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
| Domain verification/takeover lifecycle is not certified | DNS TXT proof and IDNA normalization are implemented; add proof expiry/revalidation, global wildcard conflict transaction, hold/reclaim, external DNS and takeover drills |
| Future cache consistency is not proven | event gap/replay and route scoping are tested; add public-to-private priority eviction, negative TTL, stale policy, and stampede tests before enabling content cache |
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
| Ownership/bounded contexts | active | Deploy cloud authority and cloud/standalone process isolation implemented | artifact/topology regression gate passes; production audit pending | implementation closed/evidence gated |
| Database target | active | 21-table PostgreSQL/SQLite contract materialized | backup/restore/RLS production evidence missing | blocked |
| Drive Website Space/root/sync | active | WebsiteRoot, generated SDK delivery, event processor, range/path/version checks implemented | deployed atomic-sync E2E missing | blocked |
| Live Wiki state/provider | active | canonical publication, provider API/SDK, events, Web adapter, durable event consumer, and real Deploy-compiler-to-Web execution contract implemented | rendition/full-text/deployed E2E missing | blocked |
| Runtime descriptor/routing | active | descriptor/runtime set, desired assignment, authenticated observation evidence, node-local activation probe, strict quorum, current-revision advancement, and focused compiler-to-Wiki device routing implemented | external multi-vantage probe and production multi-node rollout/rollback evidence missing | blocked |
| Domain/path/Variant | active | normalized composition, conflict checks, exact/wildcard/path routing, and desktop/mobile/tablet/TV/bot selection implemented | takeover and production routing evidence missing | blocked |
| TLS/certificate lifecycle | complete target; useful runtime primitives | partial | real fleet evidence missing | blocked |
| User/admin UX | complete view/workflow inventory | not implemented | E2E missing | blocked |
| Security/privacy/abuse | complete requirements | partial foundations | external/drill evidence missing | blocked |
| Observability/SLO | complete signals/targets | partial | dashboards/load/error budget missing | blocked |
| Entitlement/metering | complete ownership/dimensions | absent/partial | reconciliation missing | blocked |
| Release/profile isolation/rollback | complete plans | prelaunch baseline and cloud/standalone isolation implemented | staged artifact/rollout evidence missing | blocked |

## 7. Remaining Definition Of Ready For Pilot

- Product and architecture owners reconfirm REQ-2026-0001 and linked Drive, Knowledgebase, and Web
  requirements against the implemented machine contracts.
- Database owners approve the current prelaunch baseline, tenant/RLS posture, and strict
  cloud/standalone authority isolation without compatibility tables or dual writes.
- API/SDK owners approve the active App composition operation and any future privileged Backend
  composition credential-delegation contract.
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

- accepted requirements/ADRs and prelaunch convergence/profile-boundary approvals;
- database/API/SDK/descriptor/provider contracts and compatibility matrix;
- unit/contract/integration/E2E/security/fuzz/load/soak/fault-injection reports;
- tenant isolation, domain takeover, certificate, provider outage, cache revocation, and abuse drills;
- backup/restore/RPO/RTO, rollout/canary/rollback, Web Node drift/upgrade evidence;
- usage dedupe/replay/reconciliation and plan/quota behavior;
- UI accessibility/permission/error-state acceptance for tenant and admin views;
- release artifacts, checksums, signing, SBOM, provenance, changelog, and support/runbook handoff.

## 10. Review Outcome

The target system is architecturally coherent, commercially scoped, and materially implemented. Its
provider validation, composition transaction, generated SDKs, immutable descriptor, desired runtime
assignment, authenticated observation convergence, node-local activation probes, strict quorum,
Drive/Wiki delivery, provider-event processing, and device routing may proceed to integrated
production-like testing. It must not be represented as production-ready or commercially available
until the remaining P0 findings and release gates are closed with evidence.

Knowledgebase-specific implementation evidence and realtime claim gates are maintained in
`../sdkwork-knowledgebase/docs/engineering/reviews/REVIEW-20260721-live-wiki-deployment-integration-readiness.md`.
