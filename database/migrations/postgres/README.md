# PostgreSQL Migrations

Pre-launch the Deploy schema is consolidated on the single greenfield baseline:
`database/ddl/baseline/postgres/0001_deploy_baseline.sql`. It contains the
complete initial schema (apps, DNS/domain, certificate/TLS lifecycle, node
cluster, upload session refs, artifacts and releases, unified application
delivery — apps, platform targets, source repositories, build templates,
builds, packages, release channels and rollouts, signing identities, usage
metering and entitlement projections, app database profiles, CI source event
ingestion, and the application environment promotion chain).

No ordered post-baseline migrations exist while the app is pre-launch; the
lifecycle orchestrator applies the baseline once on an empty schema
(`baseline-plus-migrations`, `lifecycle.autoMigrate=false`). The drift gate
then verifies the live schema against `database/contract/`. Shared development
schemas converge by resetting the module state to the baseline instead of
replaying forward-only migrations.

After the first production release, add ordered expand/contract migrations here
without rewriting the released baseline; the previous greenfield migration
inventory was folded into the baseline before launch.
