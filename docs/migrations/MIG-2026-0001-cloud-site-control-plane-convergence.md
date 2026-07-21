# MIG-2026-0001 Cloud Site Control-Plane Convergence

Status: proposed
Requirement: REQ-2026-0001
Owner: SDKWork Deploy maintainers
Updated: 2026-07-21
Specs: MIGRATION_SPEC.md, DATABASE_SPEC.md, API_SPEC.md, SDK_SPEC.md, DEPLOYMENT_SPEC.md,
RELEASE_SPEC.md, SECURITY_SPEC.md, TEST_SPEC.md

## 1. Scope

Converge cloud Site, domain, route, deployment, and certificate authority on normalized `deploy_*`
tables. Convert overlapping Web Server `web_*` records into a one-way runtime projection or retire
them. Introduce live Drive directory and Knowledgebase Wiki resources without translating ordinary
content changes into Releases.

This document is a migration plan only. It does not authorize schema execution or production
cutover.

## 2. Compatibility Contract

- Existing Git/package/artifact Releases remain valid and continue through `deploy_release`.
- Existing active Web Server bindings and certificates must remain serviceable throughout cutover.
- No domain may become claimable by another tenant during backfill or rollback.
- Certificate private keys are not copied through business tables; secret references are re-bound
  and certificate fingerprints are compared.
- The old and new systems must not accept independent writes to the same business record.
- Public APIs require a reviewed compatibility window and generated SDK regeneration from owner
  OpenAPI sources; generated transports are not hand-edited.

## 3. Migration Stages

### Stage 0 - Inventory And Freeze

Inventory `deploy_*` and `web_*` tables, routes, SDK methods, background jobs, certificate stores,
domain uniqueness behavior, and current production data. Freeze new Web Server management features
that would expand the overlapping model. Establish record mapping and reconciliation queries.

Exit evidence: approved ownership matrix, row counts, conflict report, secret-reference inventory,
and rollback owner.

### Stage 1 - Expand Deploy Schema

Add the normalized resource, Variant, rule, Mount, Binding, policy, revision, observation, TLS, and
metering tables described by the target architecture. Add source columns to existing Site/domain/
deployment/certificate tables through additive migrations. Do not remove old columns.

Exit evidence: PostgreSQL and supported standalone migration validation, schema contract, indexes,
RLS/tenant checks, backup, and restore rehearsal.

### Stage 2 - Backfill And Reconcile

Backfill stable UUID mappings from Web Server records into Deploy. Invalid or ambiguous host/path,
tenant, certificate, and Site mappings enter a quarantine table/report and are never auto-activated.
Compile descriptors in shadow mode and compare route/TLS behavior without serving them.

Exit evidence: deterministic rerun, zero unexplained active-record differences, certificate
fingerprint match, and sampled public route parity.

### Stage 3 - Single-Writer Cutover

Change management APIs and workers to write Deploy only. Web Server consumes signed/versioned
runtime snapshots. If a temporary projection is required, it is produced only from an accepted
Deploy revision and includes source revision/hash. Web Server write endpoints become read-only or
return an explicit migration response.

Exit evidence: dependency SDK integration checks, audit continuity, mutation rejection tests on the
old authority, and staged tenant canary.

### Stage 4 - Live Provider Enablement

Enable Drive Website Space resources, then Wiki resources, behind tenant-scoped feature flags.
Provider events invalidate caches; read-through resolution verifies freshness. Content mutation
must not create Release/Deployment/SiteRevision rows.

Exit evidence: React atomic-sync, Wiki visibility, event loss/replay, provider outage, and rollback
tests.

### Stage 5 - Retire Compatibility State

After the compatibility window and rollback freeze expire, stop projecting unused business state,
remove old write code, archive reconciliation evidence, and schedule destructive schema cleanup as
a separately approved contract migration.

Exit evidence: no consumers, no writes, no rollback dependency, approved deletion plan, and current
backup.

## 4. Rollout

Roll out by internal tenant, pilot tenant, low-risk production cohort, then general availability.
Each cohort has descriptor, TLS, origin, latency, error, cache, and reconciliation guardrails.
Automatic rollback returns serving to the last-known-good descriptor/TLS snapshot; it does not
reverse completed source writes.

## 5. Rollback

Before single-writer cutover, stop new Deploy mutation traffic and resume the previous management
path only from the last reconciled checkpoint. After cutover, prefer forward-fix. Runtime rollback
selects the last-known-good descriptor and certificate snapshots. Schema contraction is never used
as an emergency rollback after new-shape writes begin.

## 6. Human Review Gates

- approval of table prefix, table/column names, enum vocabulary, and RLS strategy;
- approval of public API compatibility and generated SDK ownership;
- approval of domain conflict and certificate secret-reference mapping;
- approval before disabling Web Server writes;
- approval before any destructive cleanup or production migration.

## 7. Verification

Required evidence includes database contract validation, migration plan/status/drift checks,
backfill idempotency, source/target row reconciliation, domain uniqueness, tenant isolation,
descriptor golden tests, TLS fingerprint and SNI probes, API/SDK contract checks, staged traffic
comparison, backup/restore, and rollback drills.

