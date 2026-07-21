# SDKWork Cloud Site Publishing Platform PRD

Status: draft
Owner: SDKWork Deploy maintainers
Application: sdkwork-deploy
Updated: 2026-07-21
Requirement: REQ-2026-0001
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, DRIVE_SPEC.md, SECURITY_SPEC.md,
PRIVACY_SPEC.md, PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, DEPLOYMENT_SPEC.md,
RELEASE_SPEC.md

## 1. Product Summary

SDKWork Cloud Site Publishing is a multi-tenant control plane for publishing live directory trees
and live knowledgebase Wikis through SDKWork Web Server. It provides one governed model for sites,
resources, URL mounts, domains, client variants, TLS certificates, configuration revisions,
observability, quotas, and commercial operations.

The product deliberately separates content synchronization from deployment:

- A Drive directory or Knowledgebase `sources/raw` tree is the live content authority.
- Uploading or editing a file does not create a Deploy Release.
- A configuration change creates an immutable `SiteRevision` and rolls that revision to Web Nodes.
- `deploy_release` remains the artifact authority for Git, package, image, and frozen-bundle workflows.
- An optional Drive `ATOMIC_SYNC` changes a complete directory root as one operation without creating
  a Deploy Release.

## 2. Problem

The platform currently has useful but fragmented primitives: Drive owns files, Knowledgebase owns
documents, Deploy owns site/domain/deployment records, and Web Server owns delivery and ACME
execution. Without a unified publishing contract, teams must either republish every file change,
duplicate domain and certificate state, expose entire Spaces too broadly, or build product-specific
public routers that cannot share routing, observability, quotas, or commercial controls.

Customers need the behavior they already understand from professional static hosting and Wiki
products:

- upload a built React directory and see the whole directory behave as one website;
- upload governed multi-format sources and assets to a Wiki source tree and make eligible pages available quickly;
- bind one or more custom domains;
- route desktop, mobile, tablet, TV, or bot traffic to different site resources when required;
- obtain and renew free certificates automatically, or upload managed custom certificates;
- preview, pause, recover, audit, and measure a site without rebuilding its content;
- buy capacity and features through clear, enforceable entitlements.

## 3. Target Users

| Persona | Primary outcome |
| --- | --- |
| Individual developer | Publish a static build from Drive with minimal operations work. |
| Knowledge author | Publish a navigable Wiki from governed pages, documents, media, and static assets. |
| Tenant site administrator | Govern sites, domains, variants, permissions, and quotas. |
| Application operator | Inspect activation, cache, origin, and certificate health. |
| Security administrator | Control public exposure, certificate keys, headers, and abuse response. |
| Billing administrator | Understand plan limits, usage, overage, and cost attribution. |
| SDK/API integrator | Automate site lifecycle through generated SDKs. |
| Platform administrator/SRE | Operate the global domain, TLS, Web Node, and incident fleet. |
| Anonymous reader | Receive fast, secure, device-appropriate public content. |

## 4. Product Principles

1. **Explicit eligibility.** Ordinary Drive Spaces are not website providers. Every Knowledgebase is
   Wiki-capable but remains private by default; neither source becomes publicly exposed by inference.
2. **Directory fidelity.** A mounted directory is served with its hierarchy intact, subject to
   explicit path, MIME, visibility, and security policies.
3. **Live content, revisioned configuration.** Content mutation and runtime configuration activation
   are independent lifecycles.
4. **One control-plane authority.** `sdkwork-deployments` is the writable source of truth for site,
   binding, routing, certificate metadata, revision, and rollout state.
5. **Source ownership remains local.** Drive owns files and versions; Knowledgebase owns Wiki page
   state and rendering semantics; Web Server owns HTTP/TLS execution.
6. **No storage topology leakage.** Public contracts use stable resource identities and never expose
   buckets, object keys, private upstreams, secret values, or presigned URLs as business identity.
7. **Fail closed.** Ambiguous hosts, paths, variants, visibility, certificates, or stale descriptors
   do not become public by inference.
8. **Commercial controls are first class.** Entitlements, quotas, usage, retention, audit, support,
   and SLO evidence are designed with the publishing workflow rather than added after launch.

## 5. Eligibility And Exposure Rules

### 5.1 Drive Website

A Drive resource is eligible only when all of the following are true:

- the Space has `spaceType=website`;
- a Drive-owned WebsiteRoot selects either `SPACE_ROOT` or one active same-Space descendant
  `FOLDER`, excludes reserved/internal namespaces, and is owned by the same tenant as the Site;
- an active `DRIVE_DIRECTORY` resource references that stable WebsiteRoot; its `LIVE_TREE` or
  `ATOMIC_GENERATION` content mode is provider-owned;
- an active Site Variant mounts the resource;
- an active and verified Site Binding points to the Site;
- the Site and its current configuration revision are active.

Creating a `website` Space provisions a default whole-Space WebsiteRoot but is not publication. The
Space establishes project/ownership/quota/security; the WebsiteRoot selector establishes either the
complete eligible Space tree or a chosen folder as document root. Additional folder roots are
entitlement-controlled. The same root can be reused by multiple Sites/Variants/Mounts.

### 5.2 Knowledgebase Wiki

A Knowledgebase resource is eligible only when all of the following are true:

- the Knowledgebase owns a Drive Space with `spaceType=knowledge_base` and its one canonical
  WikiPublication has been provisioned;
- its publication has `publicationType=wiki` and `wikiStatus=ACTIVE`;
- the fixed public source root resolves to `sources/raw`;
- an active `KNOWLEDGEBASE_WIKI` resource references that Wiki publication;
- an active WIKI mount and Site Binding exist;
- each requested document is in a public publication state.

`okf/`, `output/`, `.sdkwork/`, governance files, draft content, private content, failed ingest
records, and deleted nodes are never public resource roots.

## 6. Functional Scope

### 6.1 Site Lifecycle

The tenant console shall support create, retrieve, list, update, validate, preview, activate, pause,
archive, and restore operations. A Site contains one or more resources, variants, mounts, and
bindings. Activation is rejected until ownership, eligibility, route uniqueness, TLS readiness,
descriptor validity, and entitlement checks pass.

Site states are `DRAFT`, `VALIDATING`, `READY`, `ACTIVE`, `DEGRADED`, `PAUSED`, `ARCHIVED`, and
`FAILED`. A paused or archived Site returns a non-disclosing not-found response on public routes.

### 6.2 Resources And Mounts

The resource provider types are:

- `DRIVE_DIRECTORY`
- `KNOWLEDGEBASE_WIKI`

The resource creation UI/API uses a discriminated source selector:

```text
DRIVE_DIRECTORY    { websiteSpaceUuid, root: SPACE_ROOT | FOLDER(folderNodeUuid) }
KNOWLEDGEBASE_WIKI { knowledgebaseUuid }
```

For Drive, Deploy asks Drive to create or reuse the stable WebsiteRoot and persists only its
`providerResourceUuid`. For Knowledgebase, Deploy resolves the one canonical WikiPublication. A
draft/paused Wiki may be connected for authenticated configuration/preview, but public activation
requires `ACTIVE`. Source selectors, node UUIDs, and publication UUIDs are never accepted from a
different tenant, and Deploy does not duplicate their business state.

The same provider resource may be attached to multiple Sites and mounted by multiple Variants or URL
prefixes. A Site-local Resource remains the configuration identity, while
`providerResourceUuid` remains the source identity.

The handler types are:

- `STATIC` for directory-faithful assets and HTML;
- `SPA` for static assets plus a controlled application fallback;
- `WIKI` for provider-owned page/document/media resolution, navigation, search, and Wiki metadata.

Mounts use normalized URL prefixes and `ROOT` or `ALIAS` semantics comparable to Nginx
`location`/`root`/`alias`. Longest normalized path prefix wins. Directory listing is disabled by
default and cannot be enabled for a Wiki source root.

### 6.3 Live Content And Atomic Sync

Ordinary file create, update, move, rename, and delete operations become visible without a Deploy
Release. Provider events invalidate affected cache keys, and read-through resolution covers event
delay or loss. The product freshness objective is stated in section 12.
Deploy does not synchronously process, relay, or acknowledge each content event. Drive and
Knowledgebase commit provider state/events and Web Server consumes them directly. Deploy remains
the configuration authority and enters the path only for attachment, activation, explicit
reconciliation, provider-wide health policy, or a Site configuration change.

For Wiki sources, realtime is policy-aware: review-required changes update author state and private
preview but wait for a version-fenced publish/republish command; auto-public changes may become
public after all gates. Provider generation, route page public version, navigation/search
generation, and SiteRevision policy generation remain independent.

For hashed application bundles, the Drive console and SDK shall offer `ATOMIC_SYNC`: upload into an
isolated tree, validate completeness and quotas, then atomically switch the active root pointer.
Readers see either the old tree or the new tree, never a partially uploaded build. This operation is
a Drive content transaction, not a Deploy Release or a SiteRevision.

### 6.4 Domains And URL Bindings

A Site may have system domains and one or more custom domains. One verified domain may bind to
multiple applications through non-overlapping path prefixes, or to one Site whose Variants point to
different application directories. The same active `(hostname, pathPrefix, environment)` cannot be
claimed by multiple tenants or Sites.

Host matching order is exact host, then an explicitly approved wildcard. Path matching is longest
prefix. IDNA is normalized to ASCII for comparison while the display form is retained for UI.
Default ports, trailing dots, duplicate slashes, encoded separators, dot segments, and invalid Host
forms are normalized or rejected before lookup.

Custom domain ownership requires challenge verification before activation and periodic revalidation.
Removing DNS does not immediately transfer the domain to another tenant; an anti-takeover hold and
proof workflow applies.

### 6.5 Client Variants

Supported variant types are `DEFAULT`, `DESKTOP`, `MOBILE`, `TABLET`, `TV`, and `BOT`. A Variant can
mount a different Drive directory or Wiki resource, allowing one domain to serve independently
built PC and mobile applications.

Routing precedence is:

1. forced Variant on the Binding;
2. explicit, valid user preference;
3. exact path rule;
4. trusted Client Hints;
5. bounded User-Agent classification;
6. bot rule;
7. Binding default Variant;
8. Site default Variant.

Variant classification only selects presentation. It never grants access or weakens content
visibility. Dedicated domains such as `www.example.com` and `m.example.com` remain the preferred
deterministic option; same-domain automatic selection is optional.

### 6.6 Wiki Experience

The Wiki handler shall provide:

- Markdown and supported rich-text rendering with HTML sanitization;
- stable canonical routes, breadcrumbs, previous/next navigation, and configurable navigation tree;
- homepage selection and predictable directory index behavior;
- public, unlisted, private, draft, review, scheduled, published, and archived page states;
- bulk visibility and publication actions;
- title, description, locale, canonical URL, Open Graph metadata, sitemap, and robots policy;
- full-text search over published pages with bounded result pagination;
- asset resolution relative to the source document and root;
- redirect records for approved page moves;
- `ETag`, `Last-Modified`, conditional requests, cache policy, and content version diagnostics;
- theme tokens and templates that cannot execute untrusted server-side code.

The default upload policy is `REVIEW_REQUIRED`. Authorized tenants may select
`AUTO_PUBLIC_AFTER_CHECKS`; eligible source content becomes public only after upload completion, malware
and format checks, projection, sanitization, and index readiness. Generic uploads never silently
inherit public visibility.

### 6.7 TLS And Certificates

Certificate source types are `ACME_MANAGED`, `CUSTOM`, `SELF_SIGNED`, and `DISABLED`. Supported
deployment modes are one certificate per domain, shared SAN certificates, and wildcard
certificates. ACME challenges include HTTP-01, DNS-01, and TLS-ALPN-01; wildcard issuance requires
DNS-01.

Managed certificates shall be issued, distributed, verified, renewed, and hot-switched without a
content deployment. Existing valid certificates remain active when renewal fails. Private keys and
ACME account keys are stored only in an approved KMS/Secret Manager or encrypted standalone secret
store; the Deploy database stores secret references, fingerprints, state, and audit metadata.

### 6.8 Preview, Activation, Rollback, And Recovery

Preview uses a short-lived, tenant-authorized preview hostname or token and the same descriptor
validation path as production. Preview must not make a private provider resource publicly
enumerable.

Every accepted configuration change creates an immutable `SiteRevision`. Activation rolls a
compiled descriptor to selected Web Nodes, verifies observations and probes, and atomically changes
the active revision. Rollback selects a prior valid configuration revision. File rollback remains a
Drive node/version or `ATOMIC_SYNC` root operation; Wiki content rollback remains a Knowledgebase
document/version operation.

### 6.9 API And SDK Automation

All authenticated user workflows use generated Deploy app SDK clients. Internal administrative
workflows use the generated Deploy backend SDK with explicit backend-admin credentials. Drive,
Knowledgebase, and Web Server integrations use their owning generated SDK family or approved
in-process service port. Business modules shall not add raw HTTP wrappers or manual auth headers.

Mutations use idempotency keys where replay is possible and optimistic versions where concurrent
operator changes could conflict. List and search operations are store-paginated and bounded.

## 7. User Console Information Architecture

### 7.1 Global Navigation

| View | Primary content | Primary actions |
| --- | --- | --- |
| Overview | Site health, traffic, certificate warnings, quota use, recent changes | Create site, open incident, review warnings |
| Sites | Searchable site list, status, primary domain, source, freshness, owner | Create, filter, pause, archive |
| Domains | Domain inventory, verification, bindings, TLS, expiry | Add, verify, bind, redirect, remove |
| Usage & Plan | Entitlements, current usage, forecast, overage state | Change plan, export usage, set alerts |
| Audit | Actor, action, resource, result, trace, time | Filter, export, investigate |

### 7.2 Create Site Wizard

1. Choose Static/SPA or Wiki.
2. Select an eligible Drive Website Space/folder or active Wiki publication.
3. Choose handler behavior, index file, fallback, and default cache profile.
4. Create the default Variant and optional device Variants.
5. Select a system domain or add and verify a custom domain.
6. Select TLS policy.
7. Validate ownership, conflicts, limits, files, Wiki states, and security policy.
8. Preview.
9. Activate with a visible revision summary and rollback point.

### 7.3 Site Workspace

| Tab | Required information and controls |
| --- | --- |
| Overview | public URLs, active revision, source freshness, health, traffic, certificate, quota |
| Resources | provider type, Space/Knowledgebase identity, root, validation, reconnect |
| Routes & Mounts | URL prefix, handler, root/alias, index/fallback, precedence simulator |
| Variants | variant list, target resource/mounts, rule priority, test client classifier |
| Domains | verification records, canonical/alias role, redirects, HSTS readiness |
| TLS | source, covered names, current version, expiry, renewal, distribution observations |
| Delivery | cache, compression, headers, MIME, SPA, error pages, robots, directory policy |
| Analytics | requests, bandwidth, cache ratio, status, latency, top paths, referrers |
| Revisions | diff, author, validation, rollout, observations, rollback |
| Access | owner, maintainer, publisher, domain manager, certificate manager, viewer |
| Settings | status, environment, transfer, archive, deletion safeguards |

Every asynchronous view exposes loading, empty, permission denied, partial/degraded, retry, and
terminal error states. Destructive commands require resource identity and impact confirmation.

## 8. Source Product Views

### 8.1 Drive Views

Drive adds Website Space creation, a site-aware file explorer, default whole-Space root, additional
folder-root selector, content mode, current mounted root indicator,
`ATOMIC_SYNC`, validation results, preview/open-site commands, file version rollback, and a link to
the Deploy Site workspace. A normal Space does not show public-site controls.

### 8.2 Knowledgebase Views

Every Knowledgebase adds Wiki capability/settings and a `sources/raw` publication explorer with columns for path,
content type, ingest state, publication state, visibility, public route, source version, index state,
last error, and updated time. It supports bulk review/publish/unpublish, navigation editing,
homepage selection, route preview, broken-link review, redirects, theme, SEO, domains, and analytics.
Knowledgebases not activated as Wiki show setup/preview readiness but no public Wiki URLs.

## 9. Platform Admin Information Architecture

| View | Scope |
| --- | --- |
| Fleet overview | active Sites, global traffic, error budget, descriptor rollout, provider health |
| Tenant publishing | entitlement, quotas, suspensions, abnormal growth, support access |
| Domain registry | ownership, conflicts, verification attempts, takeover holds, wildcard use |
| Certificate fleet | expiry buckets, renewal SLO, CA rate limits, failed orders, key algorithms |
| ACME operations | accounts, orders, challenges, retry schedule, DNS provider status |
| Runtime revisions | descriptor validation, target rollout, drift, node observations, rollback |
| Web Nodes | region, version, capacity, readiness, loaded revisions, loaded certificates |
| Resource providers | Drive/Knowledgebase latency, errors, event lag, cache invalidations |
| Abuse and trust | reports, takedown, malware, phishing, legal hold, appeal state |
| Usage and metering | request, transfer, certificate, domain, build/sync, retention aggregates |
| Audit and investigations | privileged actions, secret-reference access, impersonation/support sessions |
| Incident center | active incidents, affected bindings, mitigations, customer communications |

Admin actions are separately permissioned, reason-coded, audited, and where appropriate require
four-eyes approval. Support access is time-bound and visible to the tenant.

## 10. Roles And Permissions

| Capability | Owner | Maintainer | Publisher | Domain manager | TLS manager | Analyst | Platform admin |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Manage Site settings | yes | yes | no | no | no | read | break-glass |
| Manage resources/mounts/variants | yes | yes | yes | no | no | read | break-glass |
| Activate/pause/rollback revision | yes | yes | yes | no | no | read | break-glass |
| Verify/bind domain | yes | optional | no | yes | read | read | governed |
| Manage certificate policy | yes | optional | no | read | yes | read | governed |
| View analytics and usage | yes | yes | yes | yes | yes | yes | governed |
| Transfer/delete Site | yes | no | no | no | no | no | governed |

The exact permission tokens are defined by the owning IAM/application permission manifests during
implementation. The UI matrix does not replace server-side authorization.

## 11. Commercial Model

### 11.1 Entitlement Dimensions

Plans may control:

- active Sites, resources, variants, mounts, system domains, and custom domains;
- wildcard domains, managed certificates, custom certificate upload, and certificate mode;
- monthly requests, outbound transfer, cache purge volume, preview duration, and log retention;
- Drive Website storage and version retention through Drive-owned entitlements;
- Knowledgebase Wiki page count, search index size, and publish automation through
  Knowledgebase-owned entitlements;
- analytics depth, regional placement, SSO/RBAC, audit export, support tier, and SLO tier.

Deploy checks a versioned entitlement projection but does not own price books, invoices, payments,
tax, or credit balances. It emits signed/traceable usage facts to the Commerce capability and keeps
reconcilable daily aggregates.

### 11.2 Metering Dimensions

At minimum: `site_active_hours`, `domain_active_hours`, `managed_certificate_count`,
`request_count`, `origin_request_count`, `egress_bytes`, `cache_purge_count`,
`descriptor_activation_count`, `preview_minutes`, and `log_retained_bytes`. Metrics shall have a
tenant, service period, dimension, unit, source revision, and deduplication identity.

Quota behavior is explicit per dimension: reject creation, throttle, degrade an optional feature,
allow bounded overage, or alert only. Existing public Sites are never silently deleted because a
plan changes.

## 12. Non-Functional Requirements

These are launch targets, not a claim that the current implementation already meets them.

| Area | Standard target |
| --- | --- |
| Data-plane availability | 99.95% monthly; enterprise target 99.99% after multi-region certification |
| Control-plane availability | 99.9% monthly |
| Cached static p95 server latency | <= 100 ms in-region, excluding Internet transit |
| Uncached provider p95 server latency | <= 500 ms for eligible object sizes in-region |
| Live content freshness | p95 <= 5 seconds, p99 <= 30 seconds after provider commit |
| Descriptor activation | p95 <= 30 seconds to healthy target quorum |
| TLS hot switch | no dropped established connections; new handshakes use verified current version |
| Renewal posture | attempt by 30 days before expiry; urgent escalation at 14, 7, 3, and 1 days |
| RPO/RTO | control-plane RPO <= 5 minutes and RTO <= 60 minutes for standard cloud tier |
| Tenant isolation | zero cross-tenant data disclosure; tested at API, store, cache, and hostname layers |
| Audit retention | plan/policy controlled, immutable export available for enterprise |

Performance budgets use `PERFORMANCE_SPEC.md`; telemetry and label cardinality use
`OBSERVABILITY_SPEC.md`. Public HTML should be compatible with Core Web Vitals measurement, but the
platform does not promise application-authored frontend performance.

## 13. Security, Privacy, And Abuse Requirements

- Defend against path traversal, encoded separator ambiguity, symlink/shortcut escape, Host header
  confusion, cache poisoning, request smuggling, MIME confusion, active SVG/HTML abuse, and domain
  takeover.
- Serve only `GET`, `HEAD`, and controlled `OPTIONS` on public content routes unless a separately
  owned dynamic application API is mounted.
- Apply `nosniff`, safe referrer policy, configurable CSP, bounded headers/bodies, and sanitized
  errors. HSTS is enabled only after the full domain/TLS readiness gate.
- Classify certificate private keys, ACME account keys, DNS credentials, support session data, raw
  request IP/user agent, and unpublished content according to `PRIVACY_SPEC.md`.
- Avoid unbounded hostname/path labels in metrics. Sensitive values do not enter logs or descriptor
  payloads.
- Provide malware/phishing controls, abuse reporting, tenant notification, emergency suspension,
  appeal, legal hold, and audit-preserving deletion workflows before broad public launch.

## 14. Success Metrics

- Median time from eligible source selection to first verified HTTPS response is under five minutes,
  excluding customer DNS propagation.
- At least 99% of ordinary Drive/Wiki content changes meet the freshness objective.
- At least 99.9% of eligible managed certificate renewals complete before the 14-day threshold.
- Zero active hostname/path conflicts and zero certificate private keys stored in the database.
- At least 95% of site activation failures provide an actionable failing gate and trace ID.
- Usage aggregates reconcile with raw metering facts within the documented tolerance.
- Support can determine source, configuration revision, certificate version, and serving node for a
  public request from one trace without accessing customer content.

## 15. Phases And Release Gates

### Phase 0 - Contract Approval

Approve cross-repository ownership, database migration, descriptor schema, permissions, and naming.
No public production claim is permitted.
The current Release-oriented Deploy DTO/schema, duplicate Web Server control plane, missing
provider SDK/events/runtime, and planned-only certificate renewal are explicit Phase 0 blockers,
not partial proof of the target capability.

### Phase 1 - Static Website Pilot

Drive Website Space, directory resource, STATIC/SPA mounts, system/custom domain, managed single-name
certificate, one region, preview, activation, rollback, basic analytics, and quota enforcement.

### Phase 2 - Wiki And Variants

Knowledgebase Wiki provider, page state workflow, search/navigation/SEO, device Variants, multiple
domains, custom certificates, SAN/wildcard support, event-driven invalidation, and production admin
views.

### Phase 3 - Commercial GA

Entitlement and Commerce integration, usage reconciliation, SLO dashboards, backup/restore drills,
abuse response, domain takeover recovery, certificate fleet certification, support tooling, staged
rollout, and external security/load review.

### Phase 4 - Enterprise

Multi-region routing, regional residency, enterprise audit export, approval workflows, private
origins, advanced traffic policy, and certified 99.99% data-plane tier.

## 16. Acceptance Criteria

- An ordinary Drive Space cannot be exposed; an eligible Website Space can serve either its complete
  eligible root or an explicit descendant folder with directory fidelity.
- Root selector union/default/idempotency, reserved namespace rejection, multiple folder roots,
  provider-resource reuse, and root-change SiteRevision tests pass.
- A React `dist/` tree can be atomically synchronized and switched without a Deploy Release and
  without mixed old/new hashed assets.
- Every Knowledgebase has one canonical DRAFT/PRIVATE WikiPublication; a Wiki serves only after it is
  ACTIVE and then only eligible content under `sources/raw`, respecting each page's state and
  visibility. One publication may back multiple authorized Sites/Mounts without cloning content.
- One Site supports multiple domains and distinct desktop/mobile resources with deterministic rule
  precedence and an explainable routing simulator.
- Domain verification prevents cross-tenant claims and custom-domain takeover.
- Managed and custom certificates can be associated, versioned, distributed, verified, renewed, and
  hot-switched without storing keys in the database.
- Browser-to-resource resolution is deterministic, bounded, observable, cache-safe, and tenant-safe.
- User and platform-admin views cover create, configure, operate, recover, meter, and audit
  workflows, including empty, error, degraded, and permission-denied states.
- Configuration rollback, source rollback, and certificate rollback are separate and tested.
- The control plane remains the only writable site/domain/TLS authority; Web Server runtime state is
  a one-way projection.
- Current overlapping Web Server app-api control-plane routes/tables are removed or made
  non-authoritative through an approved single-writer migration with shadow-compare and rollback
  evidence.
- Drive and Knowledgebase provider input/output AsyncAPI plus generated internal SDK dependencies
  are accepted, declared in component/app manifests, and verified in standalone/cloud integration.
- Native auto-public, explicit publish, and priority revocation meet their measured p95/p99 targets;
  private processing does not cause a global public cache flush.
- Managed renewal performs real ACME issue/challenge/version/distribution/SNI verification;
  `renewal_status=planned` alone fails acceptance.
- Required security, load, isolation, backup/restore, renewal, and rollout evidence exists before
  commercial GA.

## 17. Linked Documents

- [REQ-2026-0001 Cloud Site Publishing Platform](../../product/requirements/REQ-2026-0001-cloud-site-publishing-platform.md)
- [Unified cloud publishing control-plane ADR](../../architecture/decisions/ADR-20260721-unified-cloud-site-publishing-control-plane.md)
- [Cloud publishing control-plane architecture](../../architecture/tech/TECH-cloud-site-publishing-control-plane.md)
- [Control-plane authority convergence migration](../../migrations/MIG-2026-0001-cloud-site-control-plane-convergence.md)

## 18. Open Commercial Decisions

- Final plan names, prices, included quantities, overage prices, and contractual SLO credits.
- Initial managed ACME certificate authority set and per-CA fallback policy.
- Initial cloud regions, residency guarantees, and multi-region activation date.
- Legal process and response times for public-content abuse, DMCA-equivalent notices, and appeals.
- Whether customer-managed CDN/origin integrations enter Enterprise or a later product line.
