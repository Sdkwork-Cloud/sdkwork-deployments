# SDKWork Deploy

SDKWork Deploy is the SaaS-capable Deploy Web Server control plane. It manages web
applications, domains, TLS certificates, Nginx-compatible configuration, build and deploy
pipelines, release history, rollback, health checks, and operational audit.

This repository follows `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md` and bootstraps from the
historical `sdkwork-deploy-server` contract scaffold.

## Standards Alignment

| Framework | Status |
| --- | --- |
| `sdkwork-web-framework` | Integrated on app-api and backend-api routers |
| `sdkwork-database` | Integrated through `database/` assets and `sdkwork-deploy-database-host` |
| `sdkwork-utils-rust` | Used for env parsing and shared validation helpers |
| `sdkwork-discovery` | Deferred until RPC services are introduced |

## Root Layout

| Directory | Status | Purpose |
| --- | --- | --- |
| `apis/` | active | Authoritative OpenAPI contracts for deploy app/backend surfaces |
| `crates/` | active | Rust service, repository, route, and API server crates |
| `database/` | active | Database contract, baseline DDL, migrations, seeds, drift policy |
| `sdks/` | placeholder | SDK family workspace for generated deploy SDKs |
| `specs/` | active | Component and topology contracts |
| `configs/` | active | Topology profile env templates |
| `deployments/` | placeholder | Docker, Kubernetes, and release handoff descriptors |
| `scripts/` | active | Dev orchestration and verification entrypoints |
| `docs/` | active | PRD, design notes, ADRs |
| `tests/` | active | Cross-package contract tests |
| `apps/` | placeholder | Reserved for future PC/admin client roots |
| `jobs/`, `tools/`, `plugins/`, `examples/` | placeholder | Reserved capability directories |

## Development

```powershell
pnpm dev
pnpm check
pnpm verify
```

Database lifecycle:

```powershell
pnpm db:validate
pnpm db:plan
```

## Documentation

- PRD: `docs/superpowers/specs/2026-06-14-deploy-web-server-prd.md`
- Design report: `docs/DESIGN_REPORT.md`
- Standards entry: `../sdkwork-specs/README.md`

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Application Roots

- [apps directory index](apps/README.md)
