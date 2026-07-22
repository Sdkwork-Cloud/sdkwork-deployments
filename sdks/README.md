# SDK Families

Generated deploy SDK artifacts for app-api and backend-api consumers.

| Family | OpenAPI | Assembly |
| --- | --- | --- |
| `sdkwork-deploy-app-sdk` | `openapi/deploy-app-api.openapi.json` | `sdk-manifest.json`, `specs/component.spec.json` |
| `sdkwork-deploy-backend-sdk` | `openapi/deploy-backend-api.openapi.json` | `sdk-manifest.json`, `specs/component.spec.json` |

Route manifests for Rust handlers live under `_route-manifests/`.

Regenerate from authoritative YAML under `apis/`:

```powershell
pnpm api:materialize
```

Authority and envelope rules: `../sdkwork-specs/SDK_SPEC.md`, `../sdkwork-specs/API_SPEC.md` section 15.
