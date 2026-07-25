# SDKWork Deploy PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-deploy
Updated: 2026-07-24
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- [PRD-cloud-site-publishing-platform.md](PRD-cloud-site-publishing-platform.md) - commercial live
  Drive Space-root/folder directory and every-Knowledgebase Wiki publishing product, user console,
  admin console, frozen artifact/release workflows, domains, client Variants, TLS, metering, and
  launch gates.
- [Standards alignment](../../standards-alignment.md)
- [Technical standards alignment](../../architecture/tech/TECH-standards-alignment.md)

Deprecated redirect:

- [PRD-2026-06-14-deploy-web-server-prd.md](PRD-2026-06-14-deploy-web-server-prd.md) - retained only
  as a redirect to the current product authority; it contains no active requirements.

## 1. Background And Problem

Product detail lives in the active cloud publishing shard above. It is the product authority for
live directory/Wiki Sites and frozen artifact/release deployment. Live WebsiteRoot and
WikiPublication content changes do not enter the artifact/release pipeline.

## 2. Deployments PC Product Surface

`apps/sdkwork-deployments-pc` is the runnable PC management application. Its tenant Console covers
sites, environment configuration, domains, managed-certificate metadata, package artifacts,
releases, deployments, and monitoring. Its separately loaded backend-admin surface exposes only
the Nginx, server, and audit operations currently defined by the Deploy Backend API.

Application package bytes are uploaded through `@sdkwork/drive-app-sdk`. Deploy receives stable
Drive upload, space, and node references through `artifacts.create`, then owns immutable artifact,
release, deployment, and runtime-assignment business state. Disabling an application is the
recoverable `sites.pause` command; re-enabling it is `sites.activate`.

Console packages must consume `@sdkwork/deploy-app-sdk` and `@sdkwork/drive-app-sdk` through
console-core. Backend-admin packages must consume `@sdkwork/deploy-backend-sdk` through the lazy
admin-core boundary. UI packages must not construct clients, issue raw HTTP, create authentication
headers, or treat presigned URLs and provider object keys as business identity.

Custom certificate private-key ingestion is not commercially enabled. Production deployments use
the externally terminated TLS profile until the managed-domain/TLS ADR is accepted and the
KMS/Secret Manager custody, distribution, activation, and served-certificate evidence chain is
implemented. The PC application must not present Drive-backed private-key upload as successful
production certificate activation.


## 9. Open Questions

- Which approved KMS/Secret Manager provider owns one-time custom private-key ingestion after the
  managed-domain/TLS ADR is accepted?
- Which backend-admin rollout and forced-disable operations will be added after their audit,
  recovery, expiry, and authorization contracts are approved?
