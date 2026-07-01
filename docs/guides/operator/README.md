# Operator Guide

## Deployment Manifest

- Canonical: [deployments/deploy.yaml](../../deployments/deploy.yaml)
- Validate: `pnpm deploy:validate`
- Plan / Nginx render: `pnpm deploy:plan`, `pnpm deploy:nginx:render`

## Topology Profiles

Production profiles under `configs/topology/` include Drive integration:

- `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=0`
- `SDKWORK_DRIVE_FACADE_URL`

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
