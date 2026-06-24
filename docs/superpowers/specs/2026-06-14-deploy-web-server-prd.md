# Deploy Web Server PRD

Status: draft
Date: 2026-06-14
Owner: SDKWork Deploy Server
Repository: sdkwork-deploy-server
Primary requirement: REQ-DEPLOY-2026-0001
Related specs: REQUIREMENTS_SPEC.md, ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, DATABASE_SPEC.md, SDK_SPEC.md, SECURITY_SPEC.md, DEPLOYMENT_SPEC.md, NGINX_SPEC.md, RUNTIME_DIRECTORY_SPEC.md, OBSERVABILITY_SPEC.md, DRIVE_SPEC.md

## 1. Product Positioning

SDKWork Deploy Server is a SaaS-capable Deploy Web Server control plane. It manages web applications, domains, TLS certificates, Nginx-compatible configuration, build and deploy pipelines, release history, rollback, health checks, and operational audit.

The product must be compatible with standard Nginx configuration as the data-plane contract. Nginx remains the primary public traffic serving layer for V1. Rust services provide the control plane, configuration rendering, validation, deployment orchestration, certificate lifecycle automation, observability, and APIs.

This PRD is the source requirement document for later architecture design, database design, OpenAPI design, SDK design, and runtime/deployment design. It does not authorize frontend implementation in this repository.

## 2. Problem

Teams need a professional web server deployment platform that can deploy applications from Git or uploaded packages, bind domains and certificates, manage Nginx-compatible configuration, support multi-tenant SaaS isolation, and provide clear operational visibility.

Current design artifacts already describe site, domain, deployment, Nginx, certificate, health check, and audit concepts, but the product model is not yet strict enough for professional implementation. The missing gaps are first-class application/repository/artifact/release/build objects, certificate lifecycle modeling, upload lifecycle modeling, explicit SaaS authorization semantics, and a stronger Nginx compatibility contract.

## 3. Goals

- Provide a professional Deploy Web Server control plane compatible with standard Nginx configuration.
- Support SaaS multi-tenancy where super administrators can see all tenant data and tenant users can only see their authorized tenant resources.
- Support deployment from Git repositories and uploaded packages.
- Support package upload types including static archives, SPA/static bundles, generic zip/tar.gz packages, Docker image references, executable binaries, Java JAR, and Java WAR.
- Support domain management, domain ownership verification, TLS certificate application, custom certificate import, certificate renewal, certificate expiry alerting, and certificate binding to domains.
- Support Nginx config import, generation, validation, versioning, diff, deploy, reload, and rollback.
- Provide app-api surfaces for tenant users and backend-api surfaces for platform administrators and operators.
- Generate SDKs from authoritative OpenAPI contracts instead of hand-written raw HTTP wrappers.
- Define database objects that can support audit, rollback, traceability, tenant isolation, and future private/local deployment.

## 4. Non-Goals

- No apps/frontend implementation in this repository for this phase.
- No custom replacement for the full Nginx runtime in V1.
- No Kubernetes-first product in V1, although the model should not block future container or Kubernetes workers.
- No direct storage lifecycle implementation outside SDKWork Drive for SDKWork-owned upload storage.
- No local authentication/session implementation that bypasses appbase IAM and SDKWORK security standards.
- No generated SDK hand editing.

## 5. Users And Roles

| Role | Scope | Must Be Able To Do |
| --- | --- | --- |
| Super administrator | Platform-wide | View all tenants, applications, deployments, certificates, servers, Nginx configs, audit logs, and platform health. |
| Platform operator | Platform-wide operational scope | Manage servers, workers, global Nginx runtime, global templates, failed jobs, reload operations, and operational incidents. |
| Tenant administrator | Own tenant | Manage tenant applications, repositories, deployments, domains, certificates, config, environment variables, members, and audit logs. |
| Tenant developer | Own tenant or authorized organization/project | Import Git repos, upload packages, create deployments, view build/deploy logs, roll back own applications when permitted. |
| Tenant viewer | Own tenant or authorized organization/project | View applications, deployments, domains, health, and logs without mutating resources. |
| System worker | Internal | Execute build, upload processing, Nginx rendering, Nginx validation, deploy, reload, renewal, health check, and alert jobs. |

Authorization must be enforced in backend service logic and repository access. UI-only visibility is not sufficient.

## 6. Product Surfaces

| Surface | Audience | Purpose |
| --- | --- | --- |
| app-api | Tenant users and tenant administrators | Application deployment, domain, certificate, configuration, environment, release, health, and tenant-scoped audit workflows. |
| backend-api | Super administrators and platform operators | Platform-wide tenant visibility, server/worker management, Nginx runtime operations, platform audit, and incident operations. |
| open-api | External automation and webhooks | API-key based deployment triggers, Git webhooks, CI/CD callbacks, and controlled external integrations. |
| worker runtime | Internal | Async build, deploy, certificate renewal, health check, cleanup, and alert execution. |

All API contracts must follow SDKWORK API rules: `/app/v3/api`, `/backend/v3/api`, and a domain-approved open-api prefix such as `/deploy/v3/api`.

## 7. Core Domain Model

The product must treat the following as first-class concepts:

| Object | Meaning |
| --- | --- |
| Application | Tenant-owned deployable product or service. It groups sites, environments, repositories, releases, deployments, config, and audit history. |
| Site | Public or internal web endpoint managed by Nginx. A site maps traffic to a static root, reverse proxy upstream, process, container, or packaged workload. |
| Environment | Deployment target such as production, staging, test, or custom tenant environment. |
| Repository | Git source binding with provider, URL, auth reference, default branch, webhook state, and sync status. |
| Upload session | Drive-backed upload flow for package artifacts, with size, type, checksum, scan, retention, and completion state. |
| Artifact | Immutable build or upload output used by a release. Stores type, version, checksum, size, Drive/media reference, source metadata, and retention policy. |
| Build job | Async build execution from repository or uploaded package. Tracks commands, environment, logs, status, duration, and output artifact. |
| Release | Immutable deployable version assembled from one artifact and configuration snapshot. Supports promote and rollback. |
| Deployment | Attempt to apply a release to an environment/site/server. Tracks status, logs, operator, duration, rollback source, and health result. |
| Domain | Hostname bound to a site, with verification state, primary flag, redirect behavior, TLS policy, and Nginx server name mapping. |
| Certificate | TLS certificate lifecycle object with type, issuer, SANs, validity, renewal policy, secret references, and binding state. |
| Certificate binding | Association between certificate, domain, site, environment, and generated Nginx config. |
| Config set | Versioned runtime config collection for environment variables, config files, secrets, and Nginx overrides. |
| Nginx config version | Rendered or imported Nginx-compatible config with source, hash, validation result, active state, deploy path, and rollback metadata. |
| Server/worker node | Execution target for build/deploy/Nginx operations. Can be local, SSH-managed, container host, or future agent-based node. |
| Audit event | Append-oriented record for sensitive business, security, and platform operations. |

## 8. Functional Requirements

### 8.1 SaaS Tenancy And Permissions

Requirement id: REQ-DEPLOY-2026-0001

Acceptance criteria:

- Super administrators can list and inspect resources across all tenants from backend-api.
- Tenant users can only list and mutate applications, deployments, domains, certificates, config, logs, and audit events allowed by their tenant, organization, owner, data scope, and permission.
- Every protected operation declares auth mode, permission, tenant scope, data scope, audit event, owner, and API authority in OpenAPI extensions.
- Tenant context comes from validated appbase IAM/session context, not request payload or arbitrary headers.
- Platform-wide queries are explicit backend-api operations and are audited.

### 8.2 Application And Site Management

Requirement id: REQ-DEPLOY-2026-0002

Acceptance criteria:

- Tenant administrators can create applications and sites.
- A site can be configured as static, SPA, reverse proxy, process workload, Docker image workload, Java JAR/WAR workload, or custom packaged workload.
- Each site has lifecycle states including draft, active, paused, error, and archived.
- Site activation requires at least one valid deployment target or upstream strategy.
- Site pause does not delete deployment history, domains, certificates, or audit logs.
- Application and site list APIs support pagination, stable sorting, tenant filtering, and `q` as generic search.

### 8.3 Git Repository Import And Deployment

Requirement id: REQ-DEPLOY-2026-0003

Acceptance criteria:

- Tenant users with permission can connect a Git repository using provider metadata and a secret/auth reference.
- Supported Git inputs include provider, repository URL, branch, tag, commit hash, subdirectory, build command, output path, install command, and runtime start command when relevant.
- Repository import creates or updates a repository object and does not directly deploy without an explicit trigger unless auto-deploy is enabled.
- Webhook payloads create traceable build/deploy jobs through open-api with API-key or provider-signature verification.
- Build jobs capture status, logs, duration, commit metadata, artifact output, and failure reason.
- Re-running a Git deployment is idempotent when the same tenant, repository, ref, environment, and idempotency key are used.

### 8.4 Package Upload Deployment

Requirement id: REQ-DEPLOY-2026-0004

Acceptance criteria:

- Package upload uses SDKWork Drive-backed upload lifecycle for SDKWork-owned storage.
- Upload sessions define allowed package types, max size, checksum, retention, access policy, and scan requirements.
- Supported package types are static archive, SPA/static bundle, generic zip, generic tar.gz, Docker image reference, executable binary, Java JAR, and Java WAR.
- Upload completion creates an immutable artifact or fails with a standard problem response.
- Deploying an uploaded package creates a build job when transformation is needed and a release before deployment.
- Rollback can target a prior release generated from an uploaded artifact.

### 8.5 Build And Release Pipeline

Requirement id: REQ-DEPLOY-2026-0005

Acceptance criteria:

- Build jobs and deployments are separate resources.
- A release is immutable and references one artifact plus a config snapshot.
- Deployments record the target environment, target site, target server/node, release id, status, operator, logs, duration, and rollback source.
- Long-running build/deploy operations return `202` with a job or operation resource.
- Failed deployments preserve logs and failure reason.
- Rollback creates a new deployment that references the previous release rather than mutating historical deployment facts.

### 8.6 Domain Management

Requirement id: REQ-DEPLOY-2026-0006

Acceptance criteria:

- Tenant administrators can bind domains to sites.
- Domain uniqueness is global for active hostnames.
- Domain ownership verification supports DNS TXT and HTTP challenge modes where applicable.
- A domain can be primary or alias for a site.
- Domain redirects, HTTPS enforcement, HSTS policy, and canonical host behavior are explicit configuration.
- Domain binding changes trigger Nginx config render and validation before activation.

### 8.7 Certificate Management

Requirement id: REQ-DEPLOY-2026-0007

Acceptance criteria:

- Tenant administrators can request managed ACME certificates for verified domains.
- Tenant administrators can import custom certificates and private keys through secure, write-only input.
- Certificate private key material is stored through approved secret/file/KMS references and is never returned by API responses.
- Certificates support SANs, wildcard metadata, issuer, validity period, auto-renewal policy, renewal status, and expiry alert thresholds.
- Certificate binding to domains is explicit and auditable.
- Renewal attempts are tracked with status, logs, provider response metadata, and next retry time.
- Expiring, failed, revoked, or mismatched certificates produce alerts and audit events.

### 8.8 Nginx-Compatible Config Management

Requirement id: REQ-DEPLOY-2026-0008

Acceptance criteria:

- The system can generate standard Nginx server config for static sites, SPA sites, reverse proxy sites, WebSocket proxying, streaming proxying, TLS, redirects, gzip, cache headers, security headers, and custom safe snippets.
- Generated site files use canonical SDKWORK path conventions: `/etc/nginx/sites-enabled/sdkwork/<domain>.conf` for Linux deployment.
- Certificate paths use `/opt/certs/letsencrypt/live/<cert-name>/fullchain.pem` and `/opt/certs/letsencrypt/live/<cert-name>/privkey.pem` unless explicitly overridden by operator configuration.
- Nginx changes follow plan, render, validate, deploy, reload, health-check sequence.
- `nginx -t` must pass before reload.
- Previous active config can be restored when deploy, reload, or post-reload health checks fail.
- Imported Nginx config is preserved as raw source and parsed into a structured model where supported.
- Unsupported Nginx directives must be preserved or rejected according to documented compatibility policy; they must not be silently dropped.

### 8.9 Config, Environment, And Secrets

Requirement id: REQ-DEPLOY-2026-0009

Acceptance criteria:

- Environment variables, config files, and secret references are managed as versioned config sets.
- Secret values are write-only through APIs and are never returned, logged, or exposed in audit payloads.
- Config sets can be compared, promoted, and rolled back.
- Deployments reference the exact config snapshot used.
- ABAC/RBAC fields needed for authorization are stored as columns, not hidden only in JSON.

### 8.10 Server, Worker, And Runtime Target Management

Requirement id: REQ-DEPLOY-2026-0010

Acceptance criteria:

- Backend operators can register and inspect server/worker nodes.
- Nodes expose capability metadata such as Nginx available, build available, Docker available, SSH managed, local managed, and agent managed.
- Node health, last heartbeat, version, and supported runtime features are visible to backend-api.
- Deployments record the node/server that executed the deployment.
- V1 may support local and SSH-managed nodes; the data model must allow future agent-based nodes.

### 8.11 Observability, Logs, Alerts, And Audit

Requirement id: REQ-DEPLOY-2026-0011

Acceptance criteria:

- Build logs, deploy logs, Nginx validation output, reload output, certificate renewal logs, and health check results are queryable through authorized APIs.
- API errors use RFC 9457 problem responses.
- Logs include requestId/traceId, operationId, route template, deployment mode, safe tenant context, and redacted sensitive data.
- Metrics cover build count/duration, deploy count/duration, deploy failures, Nginx reload attempts/failures, certificate renewal attempts/failures, certificate expiry, health check success/failure, and worker queue depth.
- Audit records capture actor, tenant, operation, resource, result, requestId/traceId, and safe metadata for sensitive operations.

## 9. Required API Capability Groups

The final OpenAPI contracts must cover these app-api groups:

- Applications: create, list, retrieve, update, archive.
- Sites: create, list, retrieve, update, activate, pause, archive.
- Repositories: connect, list, retrieve, update, disconnect, webhook status.
- Upload sessions: create, retrieve, complete, cancel.
- Artifacts: list, retrieve, retain, delete when policy allows.
- Build jobs: create, retrieve, list, logs, cancel, retry.
- Releases: create, list, retrieve, promote, compare.
- Deployments: create, retrieve, list, logs, rollback, cancel, retry.
- Domains: create, list, retrieve, verify, update, delete, set primary.
- Certificates: request, import, list, retrieve, renew, delete, bind, unbind.
- Config sets: create, list, retrieve, update, diff, promote, rollback.
- Health checks: create, list, retrieve, update, delete, results.
- Tenant audit: list, retrieve.
- Tenant dashboard: stats, recent deployments, health overview, alerts.

The final OpenAPI contracts must cover these backend-api groups:

- Platform tenant/resource overview.
- Server/worker node management.
- Nginx config plan/render/validate/deploy/reload/rollback/status.
- Platform certificate operations and incident inspection.
- Cross-tenant audit search.
- Worker queue and job inspection.
- Runtime health, readiness, diagnostics, and metrics metadata.

The final OpenAPI contracts must cover these open-api groups:

- Git webhook deployment trigger.
- CI/CD deployment trigger.
- Artifact or release deployment trigger by API key.

## 10. Database Design Implications

The later database design should add or revise tables around these aggregates:

- `deploy_application`
- `deploy_site`
- `deploy_environment`
- `deploy_repository`
- `deploy_upload_session_ref`
- `deploy_artifact`
- `deploy_build_job`
- `deploy_release`
- `deploy_deployment`
- `deploy_domain`
- `deploy_domain_verification`
- `deploy_certificate`
- `deploy_certificate_binding`
- `deploy_certificate_renewal_attempt`
- `deploy_acme_account`
- `deploy_acme_order`
- `deploy_acme_challenge`
- `deploy_config_set`
- `deploy_config_variable`
- `deploy_config_file`
- `deploy_secret_reference`
- `deploy_nginx_config`
- `deploy_nginx_render_plan`
- `deploy_nginx_reload_event`
- `deploy_server_node`
- `deploy_worker_job`
- `deploy_health_check`
- `deploy_health_result`
- `deploy_alert`
- `deploy_audit_log`

Database design must avoid storing core permission, tenant, state, idempotency, ownership, status, and high-frequency query fields only in JSON. JSONB is acceptable for bounded metadata when its schema is documented.

## 11. Runtime And Deployment Requirements

- Runtime app code is `deploy`.
- Linux system config should live under `/etc/sdkwork/deploy`.
- Durable mutable state should live under `/var/lib/sdkwork/deploy`.
- Logs should live under `/var/log/sdkwork/deploy`.
- Cache should live under `/var/cache/sdkwork/deploy`.
- Runtime process state should live under `/run/sdkwork/deploy`.
- Production-like SaaS/private/server/container modes use PostgreSQL and Redis unless a documented local/private exception exists.
- Desktop/local mode is not a V1 priority for this product, but shared API semantics must not block future local/private parity.

## 12. Security Requirements

- Protected app-api and backend-api operations use SDKWORK dual token mode.
- Protected open-api operations use API key or verified provider signature according to API contract.
- Private keys, API keys, tokens, passwords, and secret values must never appear in logs, audit changes, error responses, generated SDKs, or frontend bundles.
- File upload must follow SDKWork Drive rules and define size, type, scan, checksum, retention, grant expiry, and access rules.
- Object-level authorization is required before returning or mutating tenant resources.
- Sensitive operations must emit audit events.
- Backend-api must not expose login/session creation endpoints.

## 13. Success Metrics

- A tenant administrator can deploy a static site from Git and bind HTTPS with a managed certificate.
- A tenant administrator can upload a packaged artifact and deploy it to a domain.
- A failed Nginx config change is blocked before reload.
- A failed reload or health check can revert to the previous active config.
- A super administrator can inspect cross-tenant deployment and certificate health.
- A tenant viewer cannot access another tenant resource by id.
- API contracts can generate SDKs without manual client wrappers.

## 14. V1 Scope

V1 must include:

- SaaS tenant-scoped application/site/domain/deployment/certificate management.
- Super administrator backend visibility across tenants.
- Git repository import and deployment.
- Package upload deployment through Drive-backed upload lifecycle.
- Static, SPA, reverse proxy, Docker image reference, binary, JAR, and WAR workload modeling.
- ACME managed certificate request and custom certificate import.
- Certificate-domain binding and expiry alerting.
- Nginx config plan/render/validate/deploy/reload/rollback.
- Build job, release, deployment, and rollback records.
- Audit, health checks, logs, and core metrics.
- OpenAPI and SDK design aligned with SDKWORK standards.

## 15. V2 Candidates

- Agent-based remote worker installation and lifecycle management.
- Kubernetes deployment target.
- Blue/green and canary rollout automation beyond basic rollback.
- Multi-region SaaS deployment.
- Advanced WAF/rate-limit rule authoring.
- Full Nginx config visual editor and deep import remediation.
- Policy-as-code for deployment approvals.
- Billing, quotas, and tenant plan entitlements.

## 16. Decisions To Record Before Implementation

The following architecture decisions must be recorded or embedded in follow-up architecture docs before implementation:

- API authority and SDK family ownership for deploy app-api, backend-api, and open-api.
- Domain model split between application, site, release, deployment, artifact, build job, and config set.
- Drive-backed upload integration contract for package artifacts.
- Certificate private key storage policy and ACME provider model.
- Nginx compatibility policy for unsupported or custom directives.
- Runtime node model: local, SSH-managed, and future agent-managed.
- Database table prefix and bounded context compliance for `deploy_` tables.
- Event/job model for long-running build, deploy, renewal, and reload operations.

## 17. Verification Checklist

- Requirement records have stable ids and acceptance criteria.
- OpenAPI uses `q` for generic search and int64 strings for browser-facing boundaries.
- Every SDK-generated operation declares SDKWORK ownership and authority extensions.
- Upload requirements reference Drive lifecycle instead of creating independent storage lifecycle APIs.
- Nginx deploy flow includes plan, render, validate, deploy, reload, health check, and rollback.
- Certificate APIs do not return private keys or secret values.
- Tenant isolation rules are defined for app-api and backend-api.
- Database design separates first-class entities instead of overusing `runtime_config` JSONB.
- Observability includes requestId/traceId, redaction, low-cardinality metrics, and audit correlation.
