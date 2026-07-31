# REVIEW-20260731 Domain, Deployment, And Certificate Model

Status: implementation-active
Owner: SDKWork Deploy, IAM, and Web Server maintainers
Date: 2026-07-31
Specs: DATABASE_SPEC.md, DATABASE_FRAMEWORK_SPEC.md, SUBJECT_ID_SPEC.md, API_SPEC.md,
PAGINATION_SPEC.md, SDK_SPEC.md, SECURITY_SPEC.md, NGINX_SPEC.md, TEST_SPEC.md

## 1. Scope And Verdict

This review covers the domain, application deployment, certificate, tenant identity, and runtime
activation relationships in `sdkwork-deployments`, `sdkwork-iam`, and `sdkwork-web-server`.

The target cloud model is now implemented in Deployments: root-domain Zones own hostnames,
applications bind hostnames through a relation table, certificates identify one or more hostnames,
hostnames may be covered by multiple certificates, material versions are immutable, and private
material is referenced through Secret Manager/KMS. IAM correctly owns authorization rather than
domain or certificate business tables.

Commercial readiness is still blocked by two cross-repository gaps:

1. Web Server standalone retains a legacy direct `domain -> site` and `certificate -> domain/site`
   model with certificate paths in ordinary SQL columns.
2. IAM persists subject identifiers as `TEXT`, while Deployments and Web Server use the SDKWork
   `BIGINT` subject contract. Cross-service claims therefore need a single normalized subject-id
   contract before production, not database foreign keys or per-service parsing conventions.

## 2. Authority Boundaries

| Deployment profile | Domain/TLS writer | Runtime consumer | Rule |
| --- | --- | --- | --- |
| cloud | Deployments | Web Server edge runtime | Web Server must not expose a second cloud management authority or dual-write metadata. |
| standalone | Web Server standalone control plane | Web Server local runtime | The local schema must implement the same invariants with `web_` ownership; it is not a compatibility copy of the cloud database. |
| identity | IAM | Deployments and Web Server | IAM owns subjects, grants, sessions, and authorization only; it owns no domain, certificate, or deployment tables. |

Cross-repository relationships are API/event/SDK references. Foreign keys never cross service
databases.

## 3. Canonical Relationship Model

```text
tenant
  +-- dns_zone 1 ---- * domain_hostname
  +-- application 1 -- * application_domain_binding * -- 1 domain_hostname
  +-- certificate 1 -- * certificate_identifier * ------ 1 domain_hostname
  +-- certificate 1 -- * certificate_version
  +-- application_domain_binding 1 -- * listener_certificate_binding * -- 1 certificate_version
  +-- application 1 -- * deployment
```

Required cardinalities and invariants:

| Relationship | Cardinality | Integrity rule |
| --- | --- | --- |
| Zone to hostname | one-to-many | Hostname belongs to one explicit Zone; no browser public-suffix inference. |
| Application to hostname | many-to-many historically, one active route owner per environment | One application supports many hostnames; a hostname cannot have conflicting active route ownership. |
| Certificate to hostname | many-to-many | One SAN certificate covers multiple hostnames; one hostname may retain parallel/replacement certificates. |
| Certificate to version | one-to-many | Versions are immutable; aggregate points to the current version. |
| Listener to certificate | one active certificate per key algorithm | RSA and ECDSA may coexist; duplicate active algorithm bindings fail at the database and compiler boundaries. |
| Hostname to deployment | derived | Deployment remains application-owned. Domain views derive latest deployment through the active application binding. |

Private keys, certificate PEM, provider credentials, ACME account keys, and one-time proof values
must not be stored in ordinary business columns, Drive, API responses after their one-time boundary,
logs, metadata JSON, or generated SDK models. Only opaque approved `secret_bundle_ref` values may
cross the business database boundary.

## 4. Repository Findings

### 4.1 Deployments

Implemented and aligned:

- `deploy_dns_zone` is the root-domain authority.
- `deploy_domain` is a hostname asset independent of an application.
- `deploy_site_binding` is the application/hostname relation and carries routing state.
- `deploy_certificate_identifier` provides certificate/hostname many-to-many coverage.
- `deploy_certificate_version` stores immutable metadata plus `secret_bundle_ref`.
- `deploy_listener_certificate_binding` selects explicit certificate versions and enforces one
  active certificate per key algorithm for a listener.
- Certificate creation accepts `domainIds`, validates tenant and verification state, rejects
  duplicates, and applies idempotency hashing.
- PostgreSQL integration evidence proves multi-hostname certificates, multiple certificates per
  hostname, cross-tenant rejection, pending-domain rejection, replay, and replay conflict.

Remaining production gates are provider evidence, real Secret Manager/KMS custody, ACME issuance,
fleet distribution, loaded/served fingerprint observations, expiry/revocation drills, and public SNI
convergence. Database/API completeness is not evidence that those external gates are closed.

### 4.2 IAM

IAM contains no domain, certificate, deployment, or Nginx ownership tables, which is correct.

The unresolved issue is subject identity: the PostgreSQL IAM baseline uses `TEXT` for
`tenant_id`/`organization_id`, while Deployments and Web Server use `BIGINT`. The global database
standard requires SDKWork subject references to use logical `int64`. Until IAM migrates or an
approved subject-id mapping contract is accepted, consumers must treat IAM claim values as external
identifiers, validate their canonical decimal representation, convert once at a typed ingress
boundary, reject overflow/non-canonical forms, and never create cross-service database foreign keys.

Required closure:

- select one canonical SDKWork subject-id representation;
- publish it in the subject identity contract and token claims;
- migrate IAM schema and repositories if `int64` remains authoritative;
- add cross-service maximum/minimum/invalid/leading-zero/tenant-isolation tests;
- remove local fallback parsing and implicit `0` substitutions.

### 4.3 Web Server

Current standalone gaps are P0:

| Current shape | Risk | Target |
| --- | --- | --- |
| `web_domain.site_id` | Domain asset and application binding share one row; no relation lifecycle or multi-application history. | `web_site_binding` relation with active-route uniqueness. |
| `web_domain.is_primary`, `ssl_enabled`, `ssl_provider`, `redirect_target` | Application routing/TLS policy leaks into hostname inventory. | Move routing fields to site binding and TLS policy tables. |
| `web_certificate.domain_id` and `site_id` | One certificate cannot cover multiple hostnames; application scope is duplicated. | `web_certificate_identifier`; derive application visibility through active bindings. |
| `san_list TEXT` | Unqueryable, unvalidated duplicate association. | Normalized certificate identifiers with ordered positions. |
| `cert_path`, `key_path`, `chain_path` | Private material paths are mutable ordinary columns and may expose local topology. | Immutable `web_certificate_version.secret_bundle_ref`. |
| SQLx `sqlite` + `any` authoritative repository | Conflicts with the PostgreSQL-only database manifest and server storage standard. | PostgreSQL repository and real PostgreSQL integration suite. |
| Compatibility-first REQ/ADR/API text | Prelaunch debt preserves known-wrong paths and singular `domainId`. | Replace with canonical Zone/hostname/binding/certificate contracts and regenerate SDKs. |

The current root-domain list also derives bound, HTTPS, and deployment counts from
`web_domain.site_id` and `ssl_enabled`; those projections must join the new relation and listener
tables. Deployment state itself must remain in `web_deployment`.

## 5. API And Product Contract

The professional user workflow is:

1. List root domains without requiring an application id.
2. Define a root domain and open a stable detail route.
3. Page through apex/subdomain hostnames under that root domain.
4. Verify hostname ownership before production certificate issuance or activation.
5. Bind one or more hostnames to an application through an explicit routing command.
6. Request one certificate for multiple verified hostnames and select RSA or ECDSA.
7. Retain multiple certificates per hostname for rotation and parallel algorithms.
8. Select exact immutable certificate versions for the listener and deploy through the application
   deployment workflow.
9. Show derived binding/deployment/certificate coverage in domain inventory without copying
   deployment state onto domain rows.

Every mutation requires standard dual-token security, permission scopes, tenant filtering,
idempotency where retryable, optimistic concurrency where replacing state, an audit record, and a
state-aware destructive-operation guard. Generated SDKs are the only browser/backend integration
path.

## 6. Database Migration Order

Because all three applications are prelaunch, use contract/baseline convergence rather than a
permanent compatibility layer:

1. Freeze the new logical contract and subject-id decision.
2. Add relation, identifier, version, and listener-binding tables with tenant-leading indexes and
   foreign keys inside each owning database.
3. Backfill Web Server standalone rows transactionally into the new relations and versions only if
   preserving local test data is required; otherwise rebuild the prelaunch database.
4. Switch repositories, APIs, route manifests, SDKs, UI, compiler, and runtime distribution reads.
5. Prove tenant isolation, uniqueness, idempotency, version selection, and fail-closed secret reads
   against PostgreSQL.
6. Remove direct relation columns, singular DTO fields, SQLite server paths, compatibility routes,
   obsolete migrations/fixtures, and historical documentation in the same release line.
7. Run drift validation and repeat generation/materialization to prove zero net change.

No migration may copy private-key bytes or paths into the new aggregate. Missing Secret Manager/KMS
configuration must fail closed.

## 7. Verification Gates

| Gate | Required evidence |
| --- | --- |
| Database | contract/baseline/migration agreement; PostgreSQL migration from supported prelaunch state; tenant and organization isolation; constraints and query plans |
| Repository | one application/many hostnames; hostname rebind fencing; one certificate/many hostnames; many certificates/hostname; RSA+ECDSA listener selection; deletion conflicts |
| API | standard envelopes, pagination, auth, permissions, idempotency, optimistic concurrency, no secret fields, generated route manifests |
| SDK | owner-only generation, App/Backend builds, consumer typechecks, no raw HTTP or local DTO forks, regeneration idempotency |
| UI | root list, nested hostname page, operation column, multi-select certificate request, binding/deployment visibility, error/empty/loading/permission/mobile states |
| Runtime | compiler coverage validation, atomic TLS activation, last-known-good retention, no detached asset distribution, loaded/served fingerprint observations |
| Operations | expiry, renewal, revocation, provider outage, KMS outage, rollback, backup/restore, audit, SLO dashboards and alert drills |

## 8. Current Completion State

| Item | State |
| --- | --- |
| Deployments PostgreSQL relationship model | complete |
| Deployments certificate repository integration tests | complete |
| Deployments App OpenAPI and generated SDKs | complete |
| Deployments PC root-domain/hostname/multi-domain certificate UI | implemented; browser visual evidence pending |
| IAM domain/TLS ownership separation | complete |
| IAM canonical subject-id alignment | blocked pending shared contract/migration decision |
| Web Server standalone relationship model | not aligned |
| Web Server PostgreSQL-only authoritative repository | not aligned |
| Web Server API/SDK/UI migration | not aligned |
| Real CA/DNS/KMS/fleet production evidence | not complete |

## 9. Review Outcome

Deployments is the correct cloud authority and now has the correct core relational model. IAM must
remain an identity boundary. Web Server cloud packaging must continue to consume immutable runtime
assignments only. Web Server standalone must be converged to the same invariants before the product
can claim a professional Nginx-class domain/TLS control plane.

No production or commercial-launch claim is approved until the P0 Web Server schema/security gaps,
the shared subject-id mismatch, and the external certificate/runtime evidence gates are closed.
