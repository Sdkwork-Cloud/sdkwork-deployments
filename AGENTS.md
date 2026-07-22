# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing tasks in this root. Follow specs before memory,
dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

<!-- SDKWORK-PROGRESSIVE-LOADING: v1 -->
Resolve this standards root once and use it as the global authority for the current task:

- `../sdkwork-specs/README.md`
- `../sdkwork-specs/SOUL.md`
- `../sdkwork-specs/AGENTS_SPEC.md`

Read only the relevant README task-matrix row or navigation heading, then load the selected authority
sections. Do not copy root standard text into this repository. If these relative paths do not
resolve, stop and report the broken workspace layout.
<!-- /SDKWORK-PROGRESSIVE-LOADING: v1 -->

## Application Identity

Read `sdkwork.app.config.json` for Deploy identity, API/SDK inventory, release metadata, packaging,
or app-owned capabilities. Read `etc/` for concrete environment, bind, upstream, runtime, and
deployment values. The app manifest is not runtime configuration authority.

## Local Dictionary Structure

- `AGENTS.md`: repository agent entrypoint and relative SDKWork spec index.
- `CLAUDE.md`, `GEMINI.md`, `CODEX.md`: compatibility shims that point to `AGENTS.md`.
- `sdkwork.app.config.json`: Deploy application identity, runtime, release, and capability metadata.
- `etc/`: source-controlled topology profiles, gateway templates, and safe secret-file references.
- `sdkwork.workflow.json`: GitHub packaging/release workflow manifest.
- `.github/workflows/package.yml`: thin reusable workflow call only.
- `.sdkwork/`: repository/application AI workspace metadata.
- `specs/`: local application/component contracts and topology authority.
- `apis/`: Deploy-owned API contract sources.
- `apps/`: reserved client application roots.
- `crates/`: Rust service, repository, route, provider, compiler, and API host crates.
- `sdks/`: SDK families and generated SDK artifacts.
- `database/`: database contract, baseline DDL, migrations, seeds, and drift policy.
- `deployments/`, `scripts/`, `tools/`, `docs/`, `tests/`: infrastructure descriptors, command
  entrypoints, validators, documentation, and verification assets.
- `package.json`, `Cargo.toml`: language/build manifests.

## Documentation Canon

- [docs/README.md](docs/README.md)
- [docs/product/prd/PRD.md](docs/product/prd/PRD.md)
- [docs/architecture/tech/TECH_ARCHITECTURE.md](docs/architecture/tech/TECH_ARCHITECTURE.md)

## Spec Resolution Order

<!-- SDKWORK-PROGRESSIVE-LOADING: v1 -->
Use dynamic progressive loading for the current task:

1. Read this `AGENTS.md` and any nearer component-level `AGENTS.md`.
2. Read `sdkwork.app.config.json` only when app behavior, runtime config, SDK wiring, release,
   packaging, or app-owned capabilities are touched.
3. Read local `specs/README.md` and `specs/component.spec.json` only when the task touches that
   contract.
4. Read `../sdkwork-specs/README.md`, then only the task-specific global specs.
5. Inspect implementation files after the dictionary and relevant specs are clear.

Language-specific specs are on-demand. Load only the touched language or framework specification.
<!-- /SDKWORK-PROGRESSIVE-LOADING: v1 -->

## Required Specs By Task Type

- Agent/workflow changes: `../sdkwork-specs/SOUL.md`, `../sdkwork-specs/AGENTS_SPEC.md`,
  `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`, and
  `../sdkwork-specs/TEST_SPEC.md`.
- Package script changes: `../sdkwork-specs/PNPM_SCRIPT_SPEC.md`,
  `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`, `../sdkwork-specs/CONFIG_SPEC.md`, and
  `../sdkwork-specs/TEST_SPEC.md`.
- Any code change: `../sdkwork-specs/CODE_STYLE_SPEC.md`, `../sdkwork-specs/NAMING_SPEC.md`, plus only
  the touched language/framework spec.
- Rust code: `../sdkwork-specs/RUST_CODE_SPEC.md`; add `../sdkwork-specs/RUST_RPC_SPEC.md` only when
  RPC is touched.
- API/SDK changes: `../sdkwork-specs/API_SPEC.md`, `../sdkwork-specs/WEB_FRAMEWORK_SPEC.md`,
  `../sdkwork-specs/WEB_BACKEND_SPEC.md`, `../sdkwork-specs/SDK_SPEC.md`,
  `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Database changes: `../sdkwork-specs/DATABASE_SPEC.md`,
  `../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md`, `../sdkwork-specs/PRIVACY_SPEC.md`, and
  `../sdkwork-specs/TEST_SPEC.md`.
- Source configuration changes: `../sdkwork-specs/SOURCE_CONFIG_SPEC.md`,
  `../sdkwork-specs/CONFIG_SPEC.md`, `../sdkwork-specs/ENVIRONMENT_SPEC.md`,
  `../sdkwork-specs/DEPLOYMENT_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Runtime/deployment/release changes: `../sdkwork-specs/CONFIG_SPEC.md`,
  `../sdkwork-specs/ENVIRONMENT_SPEC.md`, `../sdkwork-specs/DEPLOYMENT_SPEC.md`,
  `../sdkwork-specs/RELEASE_SPEC.md`, `../sdkwork-specs/NGINX_SPEC.md`, and
  `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.
- Security/auth changes: `../sdkwork-specs/IAM_SPEC.md`,
  `../sdkwork-specs/IAM_LOGIN_INTEGRATION_SPEC.md`, `../sdkwork-specs/SECURITY_SPEC.md`, and
  `../sdkwork-specs/PRIVACY_SPEC.md`.

## Code Style Rules

Read `../sdkwork-specs/CODE_STYLE_SPEC.md` and `../sdkwork-specs/NAMING_SPEC.md` before code changes.
Use `sdkwork-utils-rust` and `sdkwork-id-core` for shared helpers instead of duplicating utility
logic locally. Generated SDK output must not be hand-edited.

Build scripts, dev runners, and `pnpm clean` must follow `CODE_STYLE_SPEC.md` section 7. Tracked
build-critical source files must be verified before builds and self-healed from git when missing;
`clean` must not delete them.

## Build, Test, and Verification

Choose the narrowest verification selected by the changed surface. Workspace-wide checks are
required only when the change crosses that boundary. Mutating alignment or materialization commands
are not verification defaults.

```powershell
pnpm check
pnpm verify
pnpm db:validate
pnpm topology:validate
```

## Agent Execution Rules

Do not rely on memory when a relevant SDKWork spec exists. Do not replace generated SDK calls with
raw HTTP. Stop when the relative specs path, app identity, component spec, API authority, SDK family,
table prefix, or provider ownership is ambiguous. `sdkwork-discovery` is not required until RPC
services are introduced.

## Task-Specific Standards

API work loads `../sdkwork-specs/API_SPEC.md` and its response-envelope and operation-pattern
validators. App/backend SDK consumer work loads `../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`,
`../sdkwork-specs/SDK_SPEC.md`, and `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`. List/search
work loads `../sdkwork-specs/PAGINATION_SPEC.md` and `check-pagination.mjs`. Source configuration
work loads `../sdkwork-specs/SOURCE_CONFIG_SPEC.md` and `check-source-config-standard.mjs`. Link
these authorities instead of copying their normative bodies into `AGENTS.md`.

## Human Review Rules

Human review is required for breaking public API changes, schema migrations, privacy/security
exceptions, generated SDK ownership changes, Nginx runtime operations, and destructive filesystem
or data operations.
