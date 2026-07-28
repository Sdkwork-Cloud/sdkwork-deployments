# MIG-2026-0002 Deployments SDK Family Rename

Status: active prelaunch cutover
Requirement: direct human-approved naming migration on 2026-07-28
Owner: SDKWork Deploy maintainers
Updated: 2026-07-28
Type: SDK and package naming
Strategy: no-compatibility-approved
Specs: MIGRATION_SPEC.md, NAMING_SPEC.md, SDK_SPEC.md, SDK_PACKAGE_NAMING_SPEC.md,
SDK_MANIFEST_SPEC.md, SDK_WORKSPACE_GENERATION_SPEC.md, APP_SDK_INTEGRATION_SPEC.md,
DEPENDENCY_MANAGEMENT_SPEC.md, TEST_SPEC.md

## 1. Scope

Rename the unpublished Deploy consumer SDK family stem from `deploy` to `deployments` so the
family identity matches the established `sdkwork-deployments` repository and PC application line.
The migration covers both HTTP consumer families:

- `sdkwork-deploy-app-sdk` -> `sdkwork-deployments-app-sdk`
- `@sdkwork/deploy-app-sdk` -> `@sdkwork/deployments-app-sdk`
- `sdkwork-deploy-backend-sdk` -> `sdkwork-deployments-backend-sdk`
- `@sdkwork/deploy-backend-sdk` -> `@sdkwork/deployments-backend-sdk`

The API authorities remain `sdkwork-deploy-app-api` and `sdkwork-deploy-backend-api`; the HTTP
paths, operation ids, DTOs, permissions, runtime application code, and `SDKWORK_DEPLOY_*`
configuration contract do not change.

## 2. Producers And Consumers

Producer:

- `sdkwork-deployments`: family manifests, composed TypeScript consumer packages, generated
  transports, generation scripts, OpenAPI-derived SDK inputs, route manifests, PC composition,
  documentation, and workspace lock state.

Consumers:

- `sdkwork-deployments/apps/sdkwork-deployments-pc`: app and backend-admin SDK composition.
- `sdkwork-birdcoder/apps/sdkwork-birdcoder-pc`: application publishing adapter and package
  workspace dependency.
- `sdkwork-api-cloud-gateway`: embedded API surface metadata that identifies the SDK families.

## 3. Compatibility Window

Start: 2026-07-28
End: 2026-07-28
Release boundary: prelaunch `0.1.0`

Both SDK manifests declare `releaseState: not_published`, the Deployments PC application is
`DRAFT`, and BirdCoder is prelaunch. Human review approved a direct cutover, so no deprecated npm
alias, compatibility facade, dual package publication, or generated transport fork is retained.

## 4. Cutover

1. Rename both family roots and TypeScript consumer roots.
2. Update materialization and `sdkgen` inputs so consumer and transport names derive from the new
   family stems.
3. Regenerate OpenAPI-derived inputs, route manifests, family manifests, and TypeScript transports.
4. Update Deployments PC, BirdCoder, and cloud-gateway component contracts and imports.
5. Regenerate pnpm lockfiles after all workspace paths and package names resolve.
6. Verify no legacy family or npm package name remains in source, generated evidence, or lockfiles.

## 5. Rollback

Rollback is supported before publication:

1. Revert the rename changes in `sdkwork-deployments`, `sdkwork-birdcoder`, and
   `sdkwork-api-cloud-gateway` as one coordinated change.
2. Run `pnpm api:materialize` and `pnpm sdk:generate` in `sdkwork-deployments`.
3. Run lockfile-only installation in `sdkwork-deployments` and `sdkwork-birdcoder`.
4. Re-run SDK naming, component composition, consumer import, TypeScript, and contract checks.

After either new npm package is published, rollback requires a new governed migration rather than
republishing the retired names implicitly.

## 6. Verification

- `node ../sdkwork-specs/tools/check-sdk-standard.mjs --workspace .`
- `node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .`
- `node --test tests/contract/openapi-materialization.contract.test.mjs`
- `pnpm --filter @sdkwork/deployments-app-sdk build`
- `pnpm --filter @sdkwork/deployments-backend-sdk build`
- `pnpm --dir apps/sdkwork-deployments-pc check`
- BirdCoder infrastructure typecheck and application publishing tests
- Cloud-gateway component contract verification
- Workspace-wide scan for the two retired family stems and npm package names
