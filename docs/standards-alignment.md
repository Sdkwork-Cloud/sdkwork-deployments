# Standards Alignment

SDKWork Deploy standards alignment for `sdkwork-deployments`.

## Integrated Frameworks

| Framework | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | Auth layers, route manifests, `WebRequestContext`, `finish_api_json` / `problem_response` response mapping |
| `sdkwork-database` | Integrated | `database/` assets, `sdkwork-deploy-database-host`, `pnpm db:*` |
| `sdkwork-utils-rust` | Integrated | `SdkWorkApiResponse`, `PageInfo`, `parse_bool`, `slugify`, `sha256_hash`, shared pagination |
| `sdkwork-discovery` | Deferred | HTTP-only unified-process V1; add when split-services RPC is required |
| `sdkwork-drive-app-sdk` | Integrated | `sdkwork-deploy-drive-port` delegates package uploads to Drive via generated Rust SDK; memory adapter for local dev |

## HTTP API Envelope

All app-api and backend-api handlers return:

- **Success:** `{ "code": 0, "data": { "item" \| "items" + "pageInfo" }, "traceId" }` via `sdkwork-utils-rust` + `sdkwork-web-framework`
- **Error:** HTTP 4xx/5xx `application/problem+json` with numeric `code` and `traceId`

OpenAPI authorities under `apis/` and materialized SDK contracts under `sdks/` are kept in sync by `pnpm api:materialize` (includes v3 envelope migration).

## Upload Sessions (Drive)

App-api upload session routes (`POST/GET /app/v3/api/upload_sessions`, complete, cancel) orchestrate Drive-backed package uploads. Deploy stores metadata in `deploy_upload_session_ref`; binary storage stays in Drive.

| Env | Purpose |
| --- | --- |
| `SDKWORK_DEPLOY_USE_MEMORY_DRIVE` | Default memory adapter when unset or true; set `false` for production Drive |
| `SDKWORK_DRIVE_FACADE_URL` | Drive app-api base URL when memory drive is disabled |

## Verification

```powershell
pnpm install
pnpm verify
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
pnpm api:materialize
```

## Remaining Product Scope (not standards debt)

- Publish generated SDK client packages from `sdks/sdkwork-deploy-*-sdk`
- Certificate file upload route (`certificates.upload`) when TLS custom cert upload ships
- Public open-api surface `/deploy/v3/api` when scoped
