# SDKWork Deploy Technical Architecture

Status: active
Owner: SDKWork maintainers
Updated: 2026-07-22
Specs: ARCHITECTURE_DECISION_SPEC.md, DOCUMENTATION_SPEC.md, DOMAIN_SPEC.md, DATABASE_SPEC.md,
DEPLOYMENT_SPEC.md, SECURITY_SPEC.md

## Document Map

- [TECH-design-report.md](TECH-design-report.md)
- [TECH-standards-alignment.md](TECH-standards-alignment.md)
- [TECH-cloud-site-publishing-control-plane.md](TECH-cloud-site-publishing-control-plane.md) - active
  cloud publishing ownership, implemented composition/SDK/runtime-assignment foundation, target TLS
  lifecycle, remaining production gates, and verification matrix.

## 1. Architecture Overview

Architecture detail lives in the linked TECH shards above. The cloud publishing architecture is the
  active implementation boundary. Its App composition/provider/descriptor/desired-assignment path,
  cloud single-writer process isolation, Drive/Wiki delivery, provider-event processing,
  authenticated runtime observations, immutable convergence evidence, strict all-target quorum, and
  current-revision advancement are implemented. External public probes, cloud TLS automation, UI,
  metering, and production evidence remain gated by the linked ADR, prelaunch convergence record,
  and readiness review.


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
