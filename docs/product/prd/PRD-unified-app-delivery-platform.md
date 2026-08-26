# SDKWork Unified App Delivery Platform PRD

Status: implementation in progress
Owner: SDKWork Deploy maintainers
Application: sdkwork-deploy
Updated: 2026-08-04
Requirement: REQ-2026-0002
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md, DRIVE_SPEC.md, SECURITY_SPEC.md,
PRIVACY_SPEC.md, PERFORMANCE_SPEC.md, OBSERVABILITY_SPEC.md, DEPLOYMENT_SPEC.md,
RELEASE_SPEC.md

## 1. Product Summary

SDKWork Deploy gains a unified application delivery model on top of the cloud site publishing
platform. One tenant App can represent a static website (PC/H5), an SPA, an API service, a WeChat
or Douyin mini-program, an iOS or Android application (Flutter or native), or a HarmonyOS
application. Each App binds source repositories, compiles source through governed build templates
into standardized immutable deployment packages, manages semantic versions through channels, and
deploys through typed targets (Web runtime sets, containers, mini-program review submission, app
store/TestFlight/OTA).

The model deliberately separates:

- **source** (Git repositories; commit snapshots captured at build time);
- **build** (executions through bounded templates; logs and artifacts owned by Drive);
- **package** (immutable standardized deployment package with manifest and provenance);
- **release** (immutable semantic version referencing one package);
- **channel assignment** (current version pointer per channel with immutable promotion history);
- **deployment** (execution record against a typed target with review/rollout state).

## 2. Problem

Teams that build apps and mini-programs today have no governed path from source to release:
versions are free text, packages have no standard, builds are ungoverned, and every platform
(WeChat, Douyin, App Store, TestFlight, OTA, HarmonyOS AppGallery) has its own delivery ritual.
The existing Deploy control plane serves web Sites well but cannot express a Flutter app that
produces iOS and Android from one repository, or an H5 product that also ships a WeChat
mini-program.

## 3. Target Users

| Persona | Primary outcome |
| --- | --- |
| Application developer | Push a commit, trigger a governed build, get a validated package. |
| Release manager | Promote semantic versions through channels with auditable history and rollback. |
| Mini-program publisher | Submit the validated package to WeChat/Douyin review and track review state. |
| Mobile release manager | Distribute iOS/Android/HarmonyOS packages through TestFlight/store/OTA channels. |
| Tenant application administrator | Govern apps, platform targets, repositories, signing identities, and quotas. |
| Platform build operator | Operate build runners, toolchains, queues, and build capacity. |
| Platform administrator/SRE | Operate the fleet, retention, metering, and incident surfaces. |

## 4. Product Principles

1. **One App, many platforms.** Source and product identity live on the App; each platform target
   defines its delivery unit.
2. **Governed builds.** Build commands come from validated, versioned templates, never from
   arbitrary customer input.
3. **Immutable packages and releases.** A package or release is never edited; promotion and
   rollback are new references to immutable objects.
4. **Semantic versions everywhere.** Every release carries a comparable SemVer 2.0.0 version and
   a monotonic build number for ordering.
5. **Secret custody extends to builds.** Repository credentials, signing keys, keystores, and
   upload secrets are secret references only; key material lives in the executor environment.
6. **Platform review is platform-owned.** Deploy submits and tracks; WeChat/App Store review
   decisions remain with the platform.
7. **One control-plane authority.** Deploy is the single writer for app, build, package, release,
   channel, and deployment state; runners and executors report through typed contracts.
8. **Fail closed.** Ambiguous versions, unvalidated packages, unverified signing identities, and
   unknown build states never become releases or deployments.

## 5. Functional Scope

### 5.1 Apps And Platform Targets

App creation requires a name, slug, `app_kind`, and at least one platform target. Supported kinds:

| app_kind | Example tech stack | Delivery targets |
| --- | --- | --- |
| `STATIC_WEB` | Vite/React static build | Web runtime set via Site |
| `SPA_WEB` | React/Vue SPA | Web runtime set via Site |
| `API_SERVICE` | Rust/Node/Go service | Container image / process bundle |
| `WECHAT_MINIPROGRAM` | miniprogram framework | WeChat review submission |
| `DOUYIN_MINIPROGRAM` | miniprogram framework | Douyin review submission |
| `IOS_APP` | Flutter / native Swift | App Store Connect, TestFlight, enterprise OTA |
| `ANDROID_APP` | Flutter / native Kotlin | Play/other stores, enterprise OTA |
| `HARMONYOS_APP` | ArkTS / Flutter | AppGallery / enterprise distribution |
| `DESKTOP_APP` | Electron / Tauri / Flutter / native | Windows (MSI/NSIS/MSIX/EXE), macOS (DMG/PKG), Linux (DEB/RPM/AppImage), Microsoft Store, Mac App Store, OTA auto-update |

A platform target carries its platform identity (bundle id / package name / app id / bundle
name), tech stack (`FLUTTER`, `NATIVE`, `UNI_APP`, `ELECTRON`, `TAURI`, or web/API stack), build
template reference, and allowed channels. Desktop targets are per-operating-system
(`WINDOWS`/`MACOS`/`LINUX`); CPU architecture (x86_64/arm64) is recorded on the package
`architectures` field. One source repository can feed multiple targets; each target keeps its own
monotonic `build_number` and semantic version sequence.

Web-kind Apps link a `deploy_app`; existing Sites continue to work and are treated as implicit
`STATIC_WEB`/`SPA_WEB` Apps.

### 5.1.1 Application Database Structure Contract

An App (typically `API_SERVICE` or `DESKTOP_APP` with a server side) may declare a database
structure contract so releases ship with their data definition:

- a database profile (`database_profiles` resource) declares the engine
  (`POSTGRES`/`MYSQL`/`SQLITE`), catalog/schema name, schema and baseline versions, and the
  migration strategy (`VERSIONED`/`REPEATABLE`);
- versioned migration definitions are added under the profile with a SHA-256 checksum and an
  opaque script reference; the checksum is the release-to-schema binding evidence — a release
  carries the exact migration set recorded on the profile;
- deploy stores the definitions; the runtime executes them against the declared engine
  (Flyway/Liquibase-style versioned execution is owned by the application runtime).

### 5.2 Source Repositories

An App binds one or more Git repositories: URL, provider (GitHub, Gitee, GitLab, self-hosted),
default branch, and clone policy. Credentials are attached by secret reference only. Every build
captures an immutable source snapshot (commit SHA, branch/tag, message, author, tree state
summary) so any release can be traced to exactly the source state that produced it.

### 5.3 Build Templates And Builds

A build template defines the governed recipe for a platform target: toolchain contract (language
and versions, e.g. Node 22, Flutter 3.x, Xcode 16, JDK 17, hvigor), bounded command list with
allowlisted prefixes, artifact output paths, and quality gates (test/lint/scan summary).
Templates are versioned and validated; arbitrary shell is prohibited.

A build execution records:

- monotonic `build_number` per (App, platform target);
- source snapshot at claim time;
- state machine `QUEUED -> PREPARING -> COMPILING -> TESTING -> PACKAGING -> SUCCEEDED` (or
  `FAILED`/`CANCELLED`/`TIMED_OUT`);
- Drive-backed log reference with streamed capture;
- produced package references and quality-gate summary.

Retries reuse the claimed build row; `build_number` never decreases. The build runner claims
work through a typed executor boundary; platform-specific executors (flutter, xcodebuild, gradle,
hvigor, mini-program CI) are command constructors with environment checks.

### 5.4 Deployment Package Standard

Every successful build produces one `deploy_package`:

- immutable record with package format, manifest digest, size, checksum, opaque Drive references,
  producing build, signing identity, and platform requirements;
- in-package manifest standard `sdkwork.deploy-package.v1` with canonical hashing;
- per-format validation rules (see REQ-2026-0002 functional requirement 8): web bundles, API
  container image references / process bundles / JVM artifacts, mini-program archives with
  platform size ceilings, iOS bundle identity and signing requirements, Android
  package/signature requirements, HarmonyOS bundle/API requirements, and desktop installers;
- desktop installers validate at the container boundary (OLE for MSI, xar for PKG, ar for DEB,
  RPM magic, ELF for AppImage, PE signature for EXE/NSIS, `koly` trailer for DMG, ZIP container
  for MSIX/JAR/WAR) with a 2 GiB ceiling; the package manifest travels as registration metadata,
  never embedded (embedding would break Authenticode/notarization and vendor installers).

### 5.5 Versions, Channels, And Releases

A release references exactly one immutable package with a SemVer 2.0.0 version
(`X.Y.Z[-prerelease][+build]`, bounded). `(tenant, app, platform target, version)` is unique.
Lifecycle: `DRAFT -> ACTIVE -> SUPERSEDED/DEPRECATED -> RETIRED -> ARCHIVED`.

Channels (`stable`, `beta`, `alpha`, `qa`) keep a current release pointer per (App, platform
target). Promotion creates an immutable `deploy_channel_rollout` row with strategy
(immediate, percentage gray rollout, manual approval) and, for gray rollout, a percentage.
Every transition is auditable and reversible: rollback assigns a prior release to the channel.

### 5.6 Deployments

Deployments execute a channel release against a typed target:

| deployment_kind | Target semantics |
| --- | --- |
| `ARTIFACT_RELEASE` | frozen artifact release (legacy-compatible) |
| `SITE_CONFIG` / `TLS_CONFIG` | existing Site revision/TLS rollout |
| `MINIPROGRAM_REVIEW` | WeChat/Douyin review submission with platform review reference |
| `STORE_SUBMISSION` | App Store Connect / Microsoft Store / Mac App Store submission |
| `OTA_DISTRIBUTION` | self-hosted OTA install channel for iOS/Android/HarmonyOS and desktop auto-update (Electron `latest.yml`, Tauri `latest.json`, Sparkle `appcast.xml`) |
| `ENTERPRISE_DISTRIBUTION` | enterprise signed distribution |
| `CONTAINER_ROLLOUT` | container image rollout through an approved orchestrator |

Deployment records include start/completion times, status, strategy, percentage, platform review
reference, rollback linkage, and audit. Platform review states (`PENDING_REVIEW`, `IN_REVIEW`,
`REJECTED`, `APPROVED`, `LIVE`) are tracked as observations, never inferred.

### 5.7 Signing Identities

`deploy_signing_identity` models iOS signing, Android keystore, HarmonyOS certificate profile,
mini-program upload key, Windows Authenticode (PFX/EV), and macOS Developer ID (with notarization)
identities with bounded metadata (name, kind, fingerprint, expiry, secret reference). Key
material is never stored in Deploy; signing executes in the build runner host using injected
secret files.

### 5.8 Metering, Quotas, And Retention

Usage dimensions: build minutes, package storage bytes, release/channel counts, deployment
counts. Entitlements control active apps, platform targets, repositories, build concurrency,
package retention, and channel counts. Retention policies cover build logs, packages, releases,
and rollout history; retention never deletes audit rows.

## 6. User Console Information Architecture

### 6.1 Global Navigation Addition

| View | Primary content | Primary actions |
| --- | --- | --- |
| Apps | App list (kind, platforms, latest release, status) | Create app, open, archive |
| App workspace | Overview, Platform targets, Repositories, Builds, Packages, Releases, Channels, Deployments, Signing, Settings | Create target/repo, trigger build, promote release, deploy |
| Builds | Queue, running, history, logs, quality gates | Trigger, cancel, retry, inspect log |
| Releases | Version list, channels, lifecycle, traceability | Create release, promote, rollback, retire |

### 6.2 App Workspace Tabs

| Tab | Required information and controls |
| --- | --- |
| Overview | app kind, platform targets, source repos, latest release per channel, quota |
| Platform targets | platform, tech stack, identity, build template, allowed channels |
| Repositories | bound repos, branch, credential status, last build commit |
| Builds | build list, source snapshot, log, quality gates, produced package |
| Packages | immutable package list, format, manifest digest, size, signing |
| Releases | semver list, lifecycle, source trace, channel assignment |
| Channels | channel keys, current release, rollout history, gray percentage |
| Deployments | deployment records per target with platform review state |
| Signing | signing identities (metadata + secret reference status) |
| Audit | mutation history for the App scope |

Every asynchronous view exposes loading, empty, permission denied, partial/degraded, retry, and
terminal error states. Destructive commands (archive, retire, delete package) require resource
identity and impact confirmation.

## 7. Platform Admin Information Architecture

| View | Scope |
| --- | --- |
| Build fleet | runners, toolchains, queue depth, concurrency, failure rates, capacity |
| Package registry | formats, sizes, validation failures, retention backlog |
| Version registry | releases, channels, promotion history, deprecated/retired inventory |
| Signing identities | fingerprints, expiry, secret-reference health |
| Metering | build minutes, storage, counts, reconciliation |

Admin actions are separately permissioned, reason-coded, audited, and where appropriate require
four-eyes approval.

## 8. Roles And Permissions

| Capability | Owner | Maintainer | Publisher | Build operator | Platform admin |
| --- | --- | --- | --- | --- | --- |
| Manage App settings | yes | yes | no | read | break-glass |
| Manage platform targets/repositories | yes | yes | no | read | governed |
| Trigger/cancel builds | yes | yes | yes | yes | governed |
| Create release / promote channel | yes | yes | yes | no | governed |
| Deploy to review/store/OTA | yes | yes | yes | no | governed |
| Manage signing identities | yes | no | no | read | governed |
| Rollback channel | yes | yes | yes | no | governed |

The exact permission tokens are defined by the owning IAM manifests during implementation; the
UI matrix does not replace server-side authorization.

## 9. Non-Functional Requirements

| Area | Standard target |
| --- | --- |
| Build claim-to-start | p95 <= 30 seconds when capacity available |
| Build log visibility | streamed with p95 <= 2 seconds latency |
| Release creation | p95 <= 500 ms after package validation |
| Channel promotion | p95 <= 500 ms, strictly serialized per channel |
| Version uniqueness | enforced transactionally |
| Secret custody | zero credentials/keys in database, logs, or API responses |
| Audit | every app/build/package/release/channel/deployment mutation recorded |
| Tenant isolation | zero cross-tenant reads at API, store, and cache layers |

Performance budgets use `PERFORMANCE_SPEC.md`; telemetry and label cardinality use
`OBSERVABILITY_SPEC.md`.

## 10. Security, Privacy, And Metering Requirements

- Validate all template commands against allowlists; reject path escape and ungoverned execution.
- Never return secret references' material, presigned URLs, object keys, or private keys from any
  API.
- Bound package size, manifest size, log size, source snapshot size, and rollout history.
- Build logs and packages are tenant-scoped; cross-tenant access fails closed.
- Usage facts carry tenant, app, platform target, dimension, unit, and deduplication identity;
  aggregates reconcile with raw facts.

## 11. Success Metrics

- Median time from commit to validated package under 15 minutes for supported toolchains.
- 100% of releases carry parseable, unique semantic versions.
- 100% of packages pass format validation before release creation.
- Zero credentials or keys in database, logs, and API responses.
- Channel promotions and rollbacks complete with immutable audit history.
- Usage aggregates reconcile with raw build/package facts within documented tolerance.

## 12. Phases And Release Gates

### Phase 0 - Contract Approval

Approve REQ-2026-0002, this PRD shard, the unified App ADR, the deployment package standard,
the database contract, and naming. No production claim is permitted without implementation
evidence.

### Phase 1 - Control Plane

App/platform target/repository/build/package/release/channel/deployment schema, contract types,
semver and manifest validation, repository/service/routes, app/backend OpenAPI, generated SDKs,
build runner executor boundary with command executors, and cross-repository verification.

### Phase 2 - Web And API Delivery

Web-kind Apps link Sites; `ARTIFACT_RELEASE`/`SITE_CONFIG` deployments reuse the runtime
assignment path; `CONTAINER_ROLLOUT` target contract with an approved orchestrator boundary.

### Phase 3 - Mini-Program Delivery

WeChat and Douyin platform targets, package validation with platform size ceilings, review
submission executor (mini-program CI) after credential integration, review observation tracking.

### Phase 4 - Mobile And HarmonyOS Delivery

iOS/Android (Flutter and native) and HarmonyOS targets, signing identity enforcement, TestFlight/
store submission and OTA distribution executors after credential integration.

### Phase 4.5 - Desktop Delivery

Desktop (`DESKTOP_APP`) targets per operating system (WINDOWS/MACOS/LINUX), installer package
formats (MSI/NSIS/MSIX/EXE, DMG/PKG, DEB/RPM/AppImage) with container-boundary validation and
2 GiB ceiling, Windows Authenticode and macOS Developer ID signing identities, Microsoft Store /
Mac App Store targets, and desktop auto-update manifests (Electron `latest.yml`, Tauri
`latest.json`, Sparkle `appcast.xml`) with SHA-512 checksum binding. The application database
structure contract (profiles + versioned migration definitions) is available to all App kinds.

### Phase 5 - Commercial GA

Entitlement/usage reconciliation for builds and packages, retention enforcement, SLO dashboards,
backup/restore drills, staged rollout, external security/load review.

## 13. Acceptance Criteria

- An App of every kind can be created with platform targets and a bound repository.
- A build runs through the bounded state machine, streams a Drive-backed log, and produces a
  validated package or a stable failure code.
- Each package format fails validation when its platform rules are violated.
- `(app, target, semantic_version)` uniqueness and monotonic `build_number` are enforced.
- Channel promotion records immutable rollout history; rollback re-assigns a prior release.
- Every deployment kind records start, platform review reference, completion, and rollback
  linkage with audit.
- Source -> build -> package -> release -> channel -> deployment resolves in one bounded query.
- Legacy `deploy_app`/`deploy_artifact`/`deploy_release`/`deploy_deployment` rows remain
  readable; no destructive migration exists.

## 14. Linked Documents

- [REQ-2026-0002 Unified App Delivery Platform](../../product/requirements/REQ-2026-0002-unified-app-delivery-platform.md)
- [ADR-20260804 Unified App Delivery Platform](../../architecture/decisions/ADR-20260804-unified-app-delivery-platform.md)
- [Unified app delivery architecture](../../architecture/tech/TECH-unified-app-delivery-platform.md)
- [Cloud site publishing platform PRD](PRD-cloud-site-publishing-platform.md)
