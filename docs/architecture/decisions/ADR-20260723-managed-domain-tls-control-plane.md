# ADR-20260723 Managed Domain And TLS Control Plane

Status: proposed
Requirement: REQ-2026-0001
Owner: SDKWork Deploy maintainers
Date: 2026-07-23
Specs: ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, INTERNAL_API_SPEC.md, DATABASE_SPEC.md,
MIGRATION_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md, CONFIG_SPEC.md, DEPLOYMENT_SPEC.md,
NGINX_SPEC.md, OBSERVABILITY_SPEC.md, TEST_SPEC.md

## Context

The accepted cloud publishing architecture assigns domain and certificate control-plane ownership to
SDKWork Deploy and HTTP/TLS execution to SDKWork Web Server. The current Deploy implementation does
not satisfy that boundary:

- `domains.verify` marks a domain verified without observing a DNS or HTTP ownership proof;
- managed certificate creation persists only a pending metadata row;
- renewal changes only `renewal_status` and performs no ACME operation;
- custom certificate upload represents the private key as a Drive node reference;
- there is no durable ACME account, order, challenge, immutable certificate version, distribution,
  activation observation, served fingerprint, rollback, or revocation model.

Web Server now has a bounded native TLS consumer that can validate an immutable snapshot and mounted
certificate material, build an exact/wildcard SNI index, enforce TLS policy, atomically replace the
Rustls configuration, and recover the last known good snapshot. It deliberately does not own domain
claims, certificate orchestration, or secret custody. A reviewed contract is required to connect the
control plane and data plane without creating a second writable authority or placing private keys in
business storage.

## Decision

### 1. Ownership And Dependency Direction

1. Deploy is the only cloud writer for domain claims, verification attempts, TLS policies, ACME
   accounts, orders, challenges, certificate versions, rollout state, and activation observations.
2. Web Server consumes immutable node-scoped TLS snapshots and authorized mounted material. It
   reports loaded and served evidence but cannot create or mutate a cloud certificate intent.
3. Certificate material is never embedded in a Website runtime descriptor, TLS snapshot, event,
   database column, generated SDK model, log, metric label, or support bundle.
4. Website revisions and certificate versions remain independent. Content changes, Wiki page
   changes, certificate renewal, and certificate rollback do not create each other's revisions.
5. Cross-repository calls use generated SDKs. Deploy publishes TLS assignments through the Web
   Internal SDK; Web nodes report observations through the same Web-owned internal surface.

### 2. Domain Identity And Anti-Takeover

Deploy canonicalizes every requested host to lower-case IDNA ASCII without a trailing dot before a
claim is created. Exact and wildcard identities are distinct. Public-suffix apex claims and wildcard
label boundaries are validated before persistence.

A live exact or wildcard identity has one globally exclusive claim. A database transaction and
unique active-claim constraint prevent cross-tenant ownership races. Deletion enters a configurable
hold state before another tenant can claim the identity. A verified claim is periodically
revalidated and is suspended when proof is lost beyond the configured grace period.

Domain lifecycle:

```text
PENDING_PROOF -> VERIFYING -> VERIFIED -> ACTIVE
       |              |          |          |
       +-> FAILED <---+          +-> REVERIFYING
                                  |          |
                                  +-> SUSPENDED -> HOLD -> RELEASED
```

Only `ACTIVE` domains are eligible for public host bindings or managed certificate issuance.
`VERIFIED` proves control but does not by itself activate traffic.

### 3. Domain Verification

Each verification is a durable, expiring attempt. Supported methods are:

- `DNS_TXT`, the default for exact names and the required method for wildcard claims;
- `HTTP_FILE`, allowed only for exact names when the Web edge can serve the isolated proof path;
- `DNS_CNAME`, an optional delegated verification method when the configured provider supports it.

The API returns the proof value only when an attempt is created. Persistence stores its SHA-256
digest, bounded public record/path metadata, expiry, retry schedule, and observation evidence. A
worker performs bounded DNS or HTTP checks. A user command may request an immediate check, but it
cannot assert success. HTTP checks do not follow arbitrary redirects, do not use private or reserved
addresses, and use fixed ports and response-size/time budgets. DNS observations are normalized,
bounded, and compared in constant time where the proof value is secret before publication.

Verification success records the resolver/checker identity, observed value digest, checked time,
and proof method. It never treats possession of the Deploy API credential as domain ownership.

### 4. Certificate Intent, Versions, And Secret Custody

A logical Certificate describes source type, identifiers, renewal policy, and desired/current
version. Certificate Versions are immutable. Each version records only public certificate evidence:
serial digest, leaf SHA-256 fingerprint, SPKI digest, issuer, validity, identifiers, key algorithm,
chain digest, and opaque secret-store references.

Cloud private keys and ACME account keys live in an approved KMS/Secret Manager. Web nodes receive
short-lived, target-scoped authorization and mount immutable material through an approved secret
delivery mechanism such as Secrets Store CSI. The Web snapshot uses only
`file:<opaque-certificate-version-id>`; the configured material root maps that id to read-only
`fullchain.pem` and `privkey.pem` files.

Standalone deployments may use the approved encrypted standalone secret store. Its master key is
loaded from a protected secret file, never an environment value or database row. Standalone
self-signed certificates are never eligible for the cloud production profile.

Custom certificate import uses a one-time secret-ingest session. The private key is streamed over a
protected backend to the secret store, validated in memory, and zeroized. It is not uploaded to Drive.
The public chain may be retained in the certificate secret bundle, but Drive node ids are not a
certificate or private-key custody contract.

### 5. Managed ACME Lifecycle

Let's Encrypt production and staging directory profiles are the initial managed CA integration.
Provider ports keep CA, DNS automation, and secret storage replaceable. Exact-name certificates use
HTTP-01 by default when the edge proof path is available; DNS-01 is used for wildcards and may be
selected for exact names. TLS-ALPN-01 remains disabled until the edge challenge listener has its own
bounded activation and conflict tests.

Managed lifecycle:

```text
REQUESTED
  -> ACCOUNT_READY
  -> ORDER_PENDING
  -> CHALLENGE_PRESENTING
  -> CHALLENGE_VALIDATING
  -> FINALIZING
  -> VERSION_STORED
  -> DISTRIBUTING
  -> ACTIVATING
  -> SERVED_VERIFIED
```

Every external operation has an idempotency key, lease/fence, attempt count, next-attempt time,
deadline, bounded provider error code, and terminal/non-terminal classification. Workers use bounded
concurrency and exponential backoff with jitter. Provider responses, challenge values, account keys,
private keys, and secret-store credentials are never logged.

Renewal creates a new order and version. The current valid version remains active until the new
version reaches the configured activation quorum and public served-SNI verification passes. A failed
renewal never replaces the last known good version. Renewal begins before the 30-day threshold and
must complete before the 14-day product SLO threshold.

### 6. Distribution, Activation, And Observation

Deploy compiles a complete, hash-addressed `sdkwork.tls-runtime.v1` snapshot per Web node and
listener. A candidate includes certificate/version identity, authorized material reference,
expected fingerprint, exact/wildcard server names, validity, TLS version range, ALPN, generation,
node identity, and digest. It contains no PEM or secret-provider path.

The Web Internal API adds generated-SDK operations for:

```text
PUT  /internal/v3/api/nodes/{nodeUuid}/tls-runtime-assignments/current
POST /internal/v3/api/nodes/{nodeUuid}/tls-runtime-observations
GET  /internal/v3/api/nodes/{nodeUuid}/tls-runtime-observations/latest
```

The exact OpenAPI operation ids and request/response envelopes must follow `API_SPEC.md` during the
approved implementation. The Edge Runtime validates node scope, generation, digest, material root,
certificate/key match, SAN coverage, validity, fingerprint, listener policy, and ambiguous SNI
ownership before atomic activation. It keeps the last known good configuration on any failure.

An observation distinguishes:

- `RECEIVED`: complete snapshot persisted;
- `MATERIAL_READY`: authorized material resolved and validated;
- `LOADED`: the process atomically selected the version;
- `SERVED`: an authenticated local handshake presented the expected fingerprint for each SNI class;
- `PUBLIC_VERIFIED`: an independent external probe observed the expected fingerprint;
- `FAILED`: bounded stage and reason code without secret or filesystem disclosure.

Deploy advances the certificate's current version only after the policy's node quorum reaches
`SERVED`. Production rollout completion additionally requires `PUBLIC_VERIFIED` from the configured
vantage policy. Observations are monotonic, node-authenticated, generation-fenced, replay-safe, and
retained for audit.

### 7. Rollback, Revocation, And Expiry

Rollback selects a previous non-revoked immutable version and creates a new desired rollout; it does
not mutate history. Revocation marks a version ineligible, creates replacement assignments, and
keeps evidence of CA revocation status and operator reason. Expired, revoked, mismatched, or
unobserved versions cannot become current. Emergency fail-closed policy can remove the SNI mapping
when no safe version exists.

Domain suspension removes public bindings and certificate assignments through independent desired
generations. Reclaiming or deleting a domain never silently transfers an existing certificate or
secret reference to another tenant.

## Data View

The approved implementation replaces the prelaunch simplified domain/certificate baseline. There
is no compatibility table, dual write, backfill, or legacy Drive private-key path.

| Table | Responsibility | Critical constraints |
| --- | --- | --- |
| `deploy_domain` | Canonical claim and lifecycle | globally exclusive active normalized host; tenant/site scoped; optimistic version |
| `deploy_domain_verification` | Expiring proof attempt and observation | token digest only; lease/fence; bounded retry; immutable success evidence |
| `deploy_tls_policy` | Domain/Site TLS source, challenge, renewal, rollout policy | one active policy per binding scope; no secret values |
| `deploy_acme_account` | Tenant/platform CA account metadata | account key secret reference only; directory/profile uniqueness |
| `deploy_certificate` | Logical certificate intent | source type, desired/current version, state, optimistic version |
| `deploy_certificate_identifier` | Exact SAN and wildcard identifiers | normalized unique position; identifier must be owned by active claim |
| `deploy_certificate_order` | Durable ACME order workflow | idempotency, lease/fence, retry, deadline, external reference digest |
| `deploy_certificate_challenge` | Authorization/challenge workflow | proof digest/secret reference only; presentation and cleanup state |
| `deploy_certificate_version` | Immutable issued/imported public evidence | unique fingerprint/serial scope; KMS/secret refs only; no PEM/key columns |
| `deploy_certificate_distribution` | Version-to-node material authorization | target-scoped authorization reference, expiry, desired state |
| `deploy_tls_runtime_snapshot` | Complete node/listener generation | unique node/listener/generation; canonical digest; bounded payload metadata |
| `deploy_tls_runtime_assignment` | Snapshot SNI-to-version mapping | unambiguous exact/wildcard owner; expected fingerprint and material id |
| `deploy_tls_target_observation` | Loaded/served/public evidence | node auth, monotonic generation, dedup key, bounded reason code |

Every table follows the standard SDKWork `id`, `uuid`, tenant/ownership, audit, lifecycle, and
optimistic concurrency fields that apply to its profile. High-cardinality workflow tables have
status/next-attempt/lease indexes; tenant reads lead with `tenant_id`; observations are time
partition/retention candidates in PostgreSQL and explicitly bounded in standalone SQLite.

## API And UI Consequences

The current `domains.verify` command cannot remain an unconditional success operation. It becomes a
request to check the latest active verification attempt, while a separate creation operation returns
the one-time proof. Existing certificate create/renew responses must report accepted workflow state,
not issued/renewed success.

The custom certificate input removes `certificateNodeId` and `privateKeyNodeId` and uses a one-time
secret-ingest session. This is a breaking prelaunch API/SDK correction and requires approval and SDK
regeneration from authored OpenAPI.

Tenant UI must expose domain proof instructions, observed checks, TLS policy, certificate/version
history, renewal, rollout quorum, expiry, rollback, and bounded failure reasons. Admin UI must expose
claim conflicts/holds, CA/DNS/secret-provider health, stuck orders, fleet divergence, served
fingerprints, revocation, and audited recovery actions. No UI can display or download private keys.

## Alternatives

1. Keep metadata-only certificate rows and let operators manage TLS externally. Rejected because it
   cannot satisfy managed renewal, served evidence, rollback, or commercial support.
2. Store custom private keys in Drive. Rejected because Drive is business file storage, not approved
   private-key custody, rotation, or target authorization.
3. Put PEM in the TLS snapshot. Rejected because snapshots are replicated, inspected, and retained
   runtime metadata.
4. Let Web Server own ACME and certificate business state in cloud. Rejected because it creates a
   second domain/certificate authority and prevents deterministic cross-site governance.
5. Replace the active certificate immediately after CA issuance. Rejected because issuance does not
   prove distribution, process activation, SNI selection, or public service.
6. Use only an ingress-controller certificate abstraction. Rejected as the platform contract because
   it would not cover native Rustls, standalone, node observations, or provider portability. It may
   implement the distribution port in a specific deployment profile.

## Consequences

- Deploy gains several durable workflow tables, provider ports, workers, App/Backend operations, and
  Web Internal SDK dependencies.
- Web Server gains TLS assignment ingestion/observation contracts and cloud material delivery, while
  its standalone `web_*` certificate manager remains isolated from cloud authority.
- Domain verification and certificate APIs change before launch; generated SDKs must be regenerated.
- Production native TLS remains disabled until KMS/Secret Manager custody, material distribution,
  node/public observations, rollback, and expiry drills have evidence.
- CA, DNS, KMS/Secret Manager, and external probe providers require explicit production configuration,
  credentials, quotas, alerts, and failure budgets.

## Verification

- PostgreSQL and SQLite contract tests cover all states, constraints, tenant isolation, leases,
  idempotency, retries, rollback, revocation, retention, and failure recovery.
- Domain tests cover IDNA, public suffixes, exact/wildcard conflicts, DNS/HTTP proof, rebinding/SSRF,
  expiry, revalidation, hold/reclaim, and cross-tenant races.
- ACME tests use a controlled test CA and DNS/HTTP solvers; no production CA is used by CI.
- Secret tests prove no private key/account key appears in SQL, API/SDK models, snapshots, events,
  logs, metrics, traces, crash reports, support bundles, or Drive.
- Web tests cover material authorization, SAN/key/fingerprint/validity rejection, SNI selection,
  atomic replacement, last-known-good recovery, generation fencing, and served-fingerprint evidence.
- End-to-end staging evidence covers browser DNS -> TLS handshake -> host/path/variant routing ->
  Drive/Wiki content, renewal, renewal failure, rollback, revocation, node loss, and public probes.
- Production enablement requires Security, Database, API/SDK, Operations, and Release approval.

## Review Gate

Approval of this ADR is required before changing Deploy database baselines/migrations, public or
internal OpenAPI, generated SDKs, secret-provider custody, production TLS policy, or Nginx/runtime
operations. Approval must also select the production KMS/Secret Manager and DNS automation provider
profiles; provider credentials and live infrastructure changes are outside this source change.

## Supersedes / Superseded By

This decision refines the certificate section of
`ADR-20260721-unified-cloud-site-publishing-control-plane.md`. It does not supersede that ownership
decision. When accepted and implemented, it supersedes the simplified metadata-only verification,
Drive private-key reference, and planned-only renewal behavior.
