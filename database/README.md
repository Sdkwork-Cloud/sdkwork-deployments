# DEPLOY Database Module

Canonical lifecycle assets for `sdkwork-deploy-server` per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `deploy`
- serviceCode: `DEPLOY`
- tablePrefix: `deploy_`

## Commands

```bash
pnpm run db:materialize:contract
pnpm run db:validate
pnpm run db:bootstrap
```

Legacy SQL: `migrations/*.sql` → `database/ddl/baseline/postgres/0001_deploy_legacy_baseline.sql`

Runtime bootstrap: `pnpm run db:bootstrap` via `sdkwork-database-cli` (Node-only repo; no Rust host crate).
