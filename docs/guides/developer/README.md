# Developer Guide

## Prerequisites

- Rust toolchain (workspace `Cargo.toml`)
- Node.js + pnpm (root `package.json`)

## Local Run

```powershell
pnpm dev
```

Default topology profile: `standalone.unified-process.development` (`configs/topology/`). Upload sessions use in-memory Drive unless `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false` and `SDKWORK_DRIVE_FACADE_URL` are set.

## Verification

```powershell
pnpm check
pnpm verify
pnpm db:validate
pnpm api:materialize
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
```

## Key Paths

| Area | Location |
| --- | --- |
| OpenAPI authority | `apis/app-api/deploy/openapi.yaml`, `apis/backend-api/deploy/openapi.yaml` |
| HTTP handlers | `crates/sdkwork-routes-deploy-app-api`, `crates/sdkwork-routes-deploy-backend-api` |
| Service layer | `crates/sdkwork-intelligence-deploy-service` |
| Drive port | `crates/sdkwork-deploy-drive-port` |
| Standards status | [docs/standards-alignment.md](../../standards-alignment.md) |

See `DOCUMENTATION_SPEC.md` section 2.
