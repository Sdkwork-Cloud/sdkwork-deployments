# REQ-2026-0001 Cloud Site Publishing Platform

```yaml
id: REQ-2026-0001
title: Govern live Drive directory and Knowledgebase Wiki publication as commercial cloud sites
owner: SDKWork Deploy maintainers
status: ready
source: product
problem: SDKWork has source, deployment, TLS, and delivery primitives but no single commercial control-plane contract for live directory and Wiki websites.
users:
  - tenant site administrators
  - developers
  - knowledge authors
  - platform administrators
  - public readers
goals:
  - publish an eligible Drive Website Space root or selected folder without a release for each file change
  - connect every Knowledgebase's canonical Wiki publication while keeping non-active Wikis private
  - support multi-domain and client-variant routing
  - operate managed and custom certificate lifecycles
  - provide tenant and platform administration, metering, audit, recovery, and SLO evidence
non_goals:
  - move Drive file ownership into Deploy
  - move Wiki content ownership into Deploy
  - move price books, invoices, payments, or taxes into Deploy
  - execute arbitrary customer server code
affected_surfaces:
  - api
  - sdk
  - backend
  - database
  - pc
  - deployment
  - security
  - observability
```

Specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, DOMAIN_SPEC.md, DATABASE_SPEC.md,
DRIVE_SPEC.md, API_SPEC.md, SDK_SPEC.md, APP_SDK_INTEGRATION_SPEC.md, CONFIG_SPEC.md,
DEPLOYMENT_SPEC.md, NGINX_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md, PERFORMANCE_SPEC.md,
OBSERVABILITY_SPEC.md, TEST_SPEC.md, RELEASE_SPEC.md, MIGRATION_SPEC.md

## Functional Requirements

1. `sdkwork-deployments` shall be the only writable authority for Sites, host/path Bindings,
   Variants, routing rules, Mounts, certificate metadata, configuration revisions, and rollout
   observations.
2. Drive shall expose only a stable WebsiteRoot in an active `website` Space as `DRIVE_DIRECTORY`.
   The root selector is `SPACE_ROOT` or `FOLDER(folderNodeUuid)`; both exclude reserved/internal
   namespaces and remain independent from `LIVE_TREE`/`ATOMIC_GENERATION` content mode.
3. Every Knowledgebase shall have one canonical DRAFT/PRIVATE WikiPublication. Deploy may connect it
   for configuration/preview, but only an ACTIVE publication rooted at `sources/raw` is publicly
   eligible as `KNOWLEDGEBASE_WIKI`, with per-file publication and visibility enforcement.
4. Ordinary source file changes shall not create `deploy_release`, `deploy_deployment`, or
   `deploy_site_revision` records. Provider change events and read-through resolution shall make
   changes visible within the freshness target.
5. `ATOMIC_SYNC` shall switch a complete Drive directory tree without exposing partial bundles and
   without creating a Deploy Release.
6. A Site shall support one or more verified domains and disjoint path Bindings. Active host/path
   identity shall be globally conflict-free.
7. A Site shall support DEFAULT, DESKTOP, MOBILE, TABLET, TV, and BOT Variants with deterministic,
   explainable precedence. Variant selection shall never influence authorization.
8. Certificate lifecycle shall support ACME managed, custom, self-signed standalone, and disabled
   modes; one-name, SAN, and wildcard certificates; HTTP-01, DNS-01, and TLS-ALPN-01 where the
   provider/runtime supports them.
9. Certificate private keys and ACME account keys shall never be stored in ordinary database
   columns or runtime descriptors.
10. A compiled, immutable `WebsiteRuntimeDescriptor` shall be validated, hash-addressed, rolled out,
    observed, and atomically activated independently from certificate runtime snapshots.
11. Tenant user views and platform admin views shall cover the complete lifecycle documented in the
    PRD, with permission checks and audit records for every mutation.
12. Deploy shall enforce versioned entitlement projections, produce deduplicated usage facts, and
    expose reconcilable aggregates without becoming the Commerce invoice authority.
13. Resource creation shall use a discriminated Drive-root or Knowledgebase selector, resolve it
    through the owner SDK/service, and persist only stable provider resource identity plus bounded
    observations. The same provider resource may back multiple authorized Site Resources/Mounts.
14. Changing a Drive Space/folder selector shall select another WebsiteRoot and create a
    SiteRevision; ordinary files or atomic generation changes behind the same WebsiteRoot shall not.
15. Deploy shall declare generated Drive and Knowledgebase internal SDK dependencies for provider
    resolution, while Web Server consumes owner AsyncAPI provider events directly. Deploy shall not
    become the relay or acknowledgement authority for ordinary content changes.
16. The control-plane convergence shall retire overlapping Web Server Site/Domain/Deployment/
    Certificate write routes and `web_*` business authority after shadow comparison and
    reconciliation. A rollback shall not permit both writers to become active.
17. Managed certificate renewal success shall require a completed ACME order/challenge, immutable
    certificate version, secure distribution, Web Node activation, and served-SNI verification.
    Setting `renewal_status=planned` is scheduling evidence only.

## Non-Functional Requirements

- Security: fail-closed host/path/resource/visibility resolution, domain anti-takeover, bounded
  inputs, safe MIME/headers, secret references, tenant isolation, and abuse response.
- Privacy: minimize public metadata and telemetry; classify IP, user agent, source content, domain
  ownership proof, and certificate material; provide retention/export/deletion controls.
- Performance: cached and origin latency, content freshness, activation, and capacity targets follow
  the linked PRD and `PERFORMANCE_SPEC.md`.
- Reliability: descriptor and TLS activation are atomic; last-known-good state survives temporary
  control-plane/provider failure; renewal failure does not replace a valid certificate.
- Observability: request, resource, revision, certificate, provider, and target identities are
  correlated without unbounded or sensitive metric labels.
- Portability: supported business behavior is defined for cloud PostgreSQL and approved standalone
  profiles, with capability differences made explicit.

## Acceptance Criteria

- Contract tests prove eligibility rules for non-Website Drive Spaces, Website Space roots/folders,
  Wiki-capable but inactive Knowledgebases, and active Wiki publications.
- Contract tests cover Drive `SPACE_ROOT`/`FOLDER`, Knowledgebase canonical publication,
  multi-Site provider reuse, source-selector tenant isolation, and selector-change revision behavior.
- End-to-end tests publish a React build, update it through `ATOMIC_SYNC`, and observe no mixed
  asset tree and no Deploy Release.
- End-to-end tests upload Markdown/assets, exercise review-required and auto-public policies, and
  prove private/draft/failed/deleted pages are not public.
- Drive/Knowledgebase input/output AsyncAPI compatibility, generated internal SDK dependency,
  provider generation, route page public version, event replay/gap, and route-scoped cache tests
  pass without a Deploy content event relay.
- A single-writer test proves Deploy is the only Site/domain/TLS configuration authority and Web
  write routes/tables cannot be reactivated by normal rollback.
- Host/path conflict, IDNA, wildcard, redirect, and multi-Variant routing tests pass.
- ACME issuance/renewal/failure/hot-switch and custom certificate validation/distribution tests pass.
- Descriptor schema, deterministic compilation, hash, rollout quorum, last-known-good recovery, and
  drift detection tests pass.
- Cross-tenant tests cover database, cache key, domain, descriptor, provider resolver, preview, and
  certificate boundaries.
- User-console and admin-console acceptance covers loading, empty, denied, degraded, retry, success,
  and destructive-confirmation states.
- Meter events reconcile to daily aggregates and duplicate delivery does not duplicate billable use.
- Backup/restore, provider outage, certificate-expiry, domain-takeover, config rollback, source
  rollback, and node-fleet recovery drills have recorded evidence.

## Trace

- Product: `docs/product/prd/PRD-cloud-site-publishing-platform.md`
- Decision: `docs/architecture/decisions/ADR-20260721-unified-cloud-site-publishing-control-plane.md`
- Architecture: `docs/architecture/tech/TECH-cloud-site-publishing-control-plane.md`
- Migration: `docs/migrations/MIG-2026-0001-cloud-site-control-plane-convergence.md`
- Module requirements: Drive `REQ-2026-0004`, Knowledgebase `REQ-2026-0721`, Web Server
  `REQ-2026-0060`

## Review Gate

Human approval is required before public API, generated SDK ownership, table migrations, permission
manifests, production TLS policy, or commercial SLO commitments are implemented.
