> Migrated from `docs/standards-alignment.md` on 2026-06-24.
> Owner: SDKWork maintainers

SDKWork Deploy standards alignment status for `sdkwork-deployments`.

## Integrated Frameworks

| Framework | Status | Evidence |
| --- | --- | --- |
| `sdkwork-web-framework` | Integrated | `sdkwork-routes-deploy-*` web bootstrap, dual-token route manifests, auth context injection |
| `sdkwork-database` | Integrated | `database/` assets, `sdkwork-deploy-database-host`, `pnpm db:*` |
| `sdkwork-utils-rust` | Integrated | `sdkwork-deploy-core` env parsing, repository slugify |
| `sdkwork-discovery` | Deferred | V1 is HTTP-only unified-process; add when split-services RPC is required |

## Implementation Status

| Layer | Status | Notes |
| --- | --- | --- |
| OpenAPI authorities | Complete | App + backend YAML materialized to JSON, route manifests, SDK assembly |
| Service layer | Complete | `DeployAppApi` + `DeployBackendApi` on `DeployService` |
| Repository SQLx | Complete | All `deploy_*` tables wired via `DeployRepositoryPort` |
| HTTP routes | Complete | 20 app + 11 backend operations aligned with OpenAPI paths |
| Runtime bootstrap | Complete | `bootstrap_deploy_runtime_from_env()` with DB lifecycle + service |
| Deployments | Complete | Docker + Kubernetes manifests under `deployments/` |

## Verification

```powershell
pnpm verify
cargo test --workspace
pnpm db:validate
pnpm topology:validate
pnpm api:materialize
```

## Remaining Work

- Generate and publish SDK client packages from `sdks/sdkwork-deploy-*-sdk`
- Add open-api surface per PRD (`/deploy/v3/api`) when public domain API is scoped
- Deprecate `sdkwork-deploy-server` after migration confirmation
- Deploy API Server `nginx/configs/*/deploy` orchestrates `deployctl nginx apply` via site `runtimeConfig.sdkworkDeploy`

