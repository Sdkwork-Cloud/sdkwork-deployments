# DEPLOY Database Module

Canonical lifecycle assets for SDKWork Deploy per `DATABASE_FRAMEWORK_SPEC.md`.

- moduleId: `deploy`
- serviceCode: `DEPLOY`
- tablePrefix: `deploy_`

## Initialization state

This module is in **initialization state** for greenfield deployments:

1. **Baseline** — `database/ddl/baseline/{engine}/0001_deploy_baseline.sql` contains the full initial-state DDL snapshot, including the pre-launch greenfield migration inventory folded into the baseline before launch.
2. **Migrations** — `database/migrations/{engine}/` is reserved for post-GA incremental schema changes only. It is intentionally empty at initialization; new installations bootstrap from the complete baseline, and shared development schemas converge by resetting the module state to the baseline instead of replaying forward-only migrations.
3. **Drift** — run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
