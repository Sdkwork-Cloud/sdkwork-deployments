# Developer Guide

## Prerequisites

- Rust toolchain (workspace `Cargo.toml`)
- Node.js + pnpm (root `package.json`)

## Local Run

```powershell
pnpm dev
```

The default topology profile is `standalone.development` under `etc/topology/`. It uses in-memory
Drive and publishes Web runtime assignments to the local Web Server at `http://127.0.0.1:3800`.
Create the ignored `.runtime/secrets/deploy-web-internal-ingress-token` file before exercising the
runtime publication path. Cloud development uses only the explicit remote endpoints in
`etc/topology/cloud.development.env`.

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
