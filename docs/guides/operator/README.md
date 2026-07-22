# Operator Guide

## Deployment Manifest

- Canonical: [deployments/deploy.yaml](../../deployments/deploy.yaml)
- Validate: `pnpm deploy:validate`
- Plan / Nginx render: `pnpm deploy:plan`, `pnpm deploy:nginx:render`

## Topology Profiles

Production profiles under `etc/topology/` use the Drive facade and publish immutable runtime sets
through the Web Internal SDK:

- `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=0`
- `SDKWORK_DRIVE_FACADE_URL`
- `SDKWORK_DEPLOY_WEB_INTERNAL_API_URL`
- `SDKWORK_DEPLOY_WEB_INTERNAL_API_INGRESS_TOKEN_FILE`

The Web ingress token is mounted under `/run/secrets/sdkwork/` and is never committed. Rotate the
file atomically; Deploy reloads it for each publication attempt. Production browser access is
restricted to the exact `https://deploy.sdkwork.com` origin in the gateway template.

## Database

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:migrate
```

## Verification

```powershell
pnpm verify
```

See [standards-alignment.md](../../standards-alignment.md) and `DOCUMENTATION_SPEC.md` section 2.
