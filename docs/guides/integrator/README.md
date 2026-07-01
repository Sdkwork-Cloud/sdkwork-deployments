# Integrator Guide

## API Surfaces

| Surface | Prefix | OpenAPI |
| --- | --- | --- |
| App API | `/app/v3/api` | `apis/app-api/deploy/openapi.yaml` |
| Backend API | `/backend/v3/api` | `apis/backend-api/deploy/openapi.yaml` |

Materialized JSON and generated SDK inputs: `sdks/sdkwork-deploy-*-sdk/openapi/`.

## Response Envelope (v3)

- **Success:** `{ "code": 0, "data": { "item" | "items" + "pageInfo" }, "traceId" }`
- **Error:** HTTP 4xx/5xx `application/problem+json` with numeric `code` and `traceId`

Generated SDKs (`--standard-profile sdkwork-v3`) unwrap `data` by default.

## Package Upload (Drive)

Use app-api upload session operations (`uploadSessions.create`, `retrieve`, `complete`, `cancel`). Binary storage is delegated to SDKWork Drive; Deploy stores session metadata only. Completing a package upload (`packageType` 1–5) automatically creates an immutable artifact record.

Production requires `SDKWORK_DRIVE_FACADE_URL` and `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false` in the active topology profile.

## Artifact And Release Pipeline

1. Upload package via upload sessions; `uploadSessions.complete` creates `deploy_artifact`.
2. `sites.releases.create` with `artifactId` and `idempotencyKey` — immutable release per site.
3. `sites.deployments.create` with `releaseId` — deployment references artifact Drive path and checksum.

| Operation | Notes |
| --- | --- |
| `artifacts.list` / `retrieve` | Tenant-scoped immutable upload outputs |
| `artifacts.retain` | Marks artifact retained (status=2); does not delete Drive nodes |
| `sites.releases.list` / `retrieve` / `create` | Site-scoped immutable releases |

## Custom TLS Certificate Import

1. `uploadSessions.create` with `packageType` `6` (certificate PEM) and `7` (private key PEM); upload bytes to Drive; `uploadSessions.complete` each session.
2. `certificates.upload` with completed session ids and `idempotencyKey`. Response metadata only — private keys are never returned.

## Certificate Lifecycle

| Operation | Notes |
| --- | --- |
| `certificates.list` / `retrieve` | Metadata only (`certType`, `notAfter`, `autoRenew`, etc.) |
| `certificates.create` | Registers managed (Let's Encrypt) certificate request (`status=0` pending) |
| `certificates.renew` | Schedules renewal for `certType=1`; ACME worker automation is Phase 2+ |
| `certificates.delete` | Revokes certificate record (`status=3`); Drive nodes follow Drive retention |
| `certificates.upload` | Custom cert import via completed upload sessions |

## Regenerate Contracts

```powershell
pnpm api:materialize
```

See `DOCUMENTATION_SPEC.md` section 2 and [standards-alignment.md](../../standards-alignment.md).
