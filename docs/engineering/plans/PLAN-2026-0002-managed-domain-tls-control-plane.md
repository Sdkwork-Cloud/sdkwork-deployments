# PLAN-2026-0002 Managed Domain And TLS Control Plane

Status: active
Owner: SDKWork Deploy maintainers
Updated: 2026-07-30
Requirement: REQ-2026-0001
Decision: ADR-20260723-managed-domain-tls-control-plane
Specs: ENGINEERING_WORKFLOW_SPEC.md, MIGRATION_SPEC.md, API_SPEC.md, SDK_SPEC.md,
DATABASE_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, TEST_SPEC.md

## Objective

Replace the prelaunch metadata-only domain/certificate implementation with a durable, secure, and
observable managed TLS control plane that activates independently from Website content revisions.

## Entry Gate

The ADR and prelaunch destructive schema/API replacement are accepted. Production CA, DNS
automation, KMS/Secret Manager, CSI/material delivery, public-probe, and Nginx/runtime operations
remain human-review gates. No implementation phase may report production readiness while those
provider and operational decisions are unresolved.

## Phase 1: Contract And Schema - Completed

1. Author Deploy App/Backend request, response, error, permission, idempotency, and concurrency
   contracts for domain verification and certificate lifecycle.
2. Author Web Internal TLS assignment and observation contracts.
3. Replace the prelaunch baseline with the authoritative PostgreSQL tables and constraints.
4. Add repository ports and state-machine tests before provider implementations.
5. Preserve the implemented fail-closed exact DNS TXT token check; remove `drive://` private-key
   references and planned-only certificate success semantics. Do not add compatibility tables or
   dual writes.
6. Materialize OpenAPI and regenerate SDK families from their authorities. Generated transports are
   never hand-edited.

Exit evidence: API materialization and SDK generation, PostgreSQL baseline execution, domain and
certificate tenant/status/idempotency tests, composition/outbox PostgreSQL tests, and removal of
SQLite/Any production and test paths.

## Phase 2: Domain Proof And Anti-Takeover

1. Extend the implemented canonical IDNA normalization with public-suffix validation and global
   active-claim uniqueness.
2. Promote exact DNS TXT verification into a durable worker with leases, retry, expiry, and cleanup,
   and add the bounded HTTP proof worker.
3. Implement periodic revalidation, suspension, hold, reclaim, and audited administrator recovery.
4. Add domain proof UI and admin conflict/hold UI through generated SDKs.

Exit evidence: controlled DNS/HTTP integration tests, SSRF/rebinding tests, cross-tenant race tests,
revalidation/hold drills, UI loading/empty/denied/degraded/retry/success tests.

## Phase 3: ACME And Secret Custody

1. Implement ACME account/order/challenge provider ports and the Let's Encrypt staging profile.
2. Implement KMS/Secret Manager certificate bundle and account-key custody.
3. Implement DNS provider adapters and isolated HTTP-01 challenge presentation.
4. Implement a dedicated one-time custom certificate secret ingest, in-memory validation, and
   zeroization without Drive or ordinary JSON private-key fields.
5. Implement bounded orchestration workers with idempotency, fencing, backoff, cleanup, and audit.

Exit evidence: controlled CA issuance/failure tests, secret leak scans, provider quota/rate-limit
tests, custom import mismatch tests, KMS access-policy review, account recovery drill.

## Phase 4: Distribution And Web Activation

1. Compile complete per-node/listener TLS snapshots independently from Website runtime sets.
2. Publish through the generated Web Internal SDK and deliver immutable material through the
   approved CSI/secret adapter.
3. Extend Web cloud runtime ingestion while preserving node scope, generation fencing, atomic
   activation, and last-known-good recovery.
4. Emit authenticated `RECEIVED`, `MATERIAL_READY`, `LOADED`, `SERVED`, and failure observations.
5. Add independent public SNI fingerprint probes and strict rollout quorum.

Exit evidence: multi-node activation, corrupt/missing material, stale/conflicting generation, node
loss, SNI mismatch, hot switch, rollback, and public fingerprint tests.

## Phase 5: Renewal, Revocation, And Operations

1. Schedule renewal with 30-day start and 14-day SLO thresholds.
2. Keep the current version on failure and alert before unsafe expiry windows.
3. Implement version rollback, CA revocation, emergency SNI removal, and domain suspension.
4. Add dashboards, bounded-cardinality metrics, alert policies, audit search, and support diagnostics.
5. Run backup/restore, expiry, CA outage, DNS outage, KMS outage, node divergence, rollback, and
   revocation exercises.

Exit evidence: recorded staging drills, SLO dashboards, alert delivery, support runbook, recovery
time evidence, and Security/Operations sign-off.

## Release Gate

Commercial TLS can be enabled only after all phase exit evidence is recorded, external staging and
public probes pass, release artifacts are signed and provenance-verified, and the commercial
readiness review changes the TLS gate from blocked to passed. Until then, cloud production keeps
external TLS termination as the declared default and does not claim managed certificate completion.
