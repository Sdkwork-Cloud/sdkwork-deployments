# SDKWork Deploy Technical Standards Alignment

Status: active  
Updated: 2026-06-29

## Framework Integration

| Framework | Status | Notes |
| --- | --- | --- |
| `sdkwork-web-framework` | Complete | Bootstrap, IAM auth, route manifests, v3 response mapping |
| `sdkwork-database` | Complete | Lifecycle host, SQLx repository, `pnpm db:*` |
| `sdkwork-utils-rust` | Complete | HTTP envelope types, crypto, env parsing, pagination |
| `sdkwork-discovery` | N/A (V1) | No RPC services yet |
| `sdkwork-drive` | Complete | `sdkwork-deploy-drive-port` + app-api upload session routes; production uses `SDKWORK_DRIVE_FACADE_URL` |

## API Contract

- Authority: `apis/app-api/deploy/openapi.yaml`, `apis/backend-api/deploy/openapi.yaml`
- Materialization: `pnpm api:materialize` → JSON authorities + SDK OpenAPI + route manifests
- Runtime handlers use `SdkWorkApiResponse` / `ProblemDetail` (numeric codes)

## Verification Commands

```powershell
pnpm verify
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
```
