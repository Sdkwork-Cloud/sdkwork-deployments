# SDKWork Deploy Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-21
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, DOMAIN_SPEC.md, DATABASE_SPEC.md,
DEPLOYMENT_SPEC.md, SECURITY_SPEC.md

## Document Map

- [TECH-design-report.md](TECH-design-report.md)
- [TECH-standards-alignment.md](TECH-standards-alignment.md)
- [TECH-cloud-site-publishing-control-plane.md](TECH-cloud-site-publishing-control-plane.md) - target
  cloud publishing ownership, database contract, runtime descriptor, request flow, TLS lifecycle,
  commercial controls, and verification matrix.

## 1. Architecture Overview

Architecture detail lives in the linked TECH shards above. The cloud publishing architecture is
proposed and requires the linked cross-repository ADR and migration review before implementation.


## 2. Technology Choices

## 3. System Boundaries And Modules

## 4. Directory And Package Layout

## 5. API, SDK, And Data Ownership

## 6. Security, Privacy, And Observability

## 7. Deployment And Runtime Topology

## 8. Architecture Decision Index

- [ADR-20260721 Unified Cloud Site Publishing Control Plane](../decisions/ADR-20260721-unified-cloud-site-publishing-control-plane.md)

## 9. Verification

Implementation status and framework integration: [TECH-standards-alignment.md](TECH-standards-alignment.md).

```powershell
pnpm verify
```
