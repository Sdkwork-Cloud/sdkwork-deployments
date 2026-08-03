# PostgreSQL migrations

Add versioned SQL files using `{version}_{name}.up.sql` and optional `{version}_{name}.down.sql`.

## Evolution policy (SDKWORK database framework spec §6.1 / §7.4)

The consolidated baseline (`ddl/baseline/postgres/0001_deploy_baseline.sql`) is a
full-schema snapshot used **only** to bootstrap empty databases (`db:init`).
Existing databases converge through `db:migrate`; replaying a changed baseline
over a non-empty schema is forbidden.

Every table or column added to the baseline after the first deploy snapshot
must also exist as an idempotent forward migration here, so databases
initialized from an earlier baseline converge without manual SQL.

Current inventory:

| version | name | content |
| --- | --- | --- |
| 0001 | `deploy_upload_session_ref` | Drive-backed artifact upload session refs (historical, restored from git 5926afb; registered as historical-immutable in `metadata.json`) |
| 0002 | `deploy_artifact_release` | `deploy_artifact`, `deploy_release`, `deploy_deployment.release_id` |
| 0003 | `deploy_site_runtime` | site resource / variant / variant_rule / mount / binding / revision / web_node_target / runtime_assignment / site_target_observation |
| 0004 | `deploy_domain_certificate_tls` | `deploy_dns_zone`, domain_verification, ACME account/order/challenge/distribution, certificate_identifier/version, TLS policy/binding/runtime tables |
| 0005 | `deploy_node_cluster` | node cluster grouping table |
| 0006 | `deploy_legacy_schema_convergence` | converge pre-2026-07-31 `deploy_domain` / `deploy_certificate` shapes and `deploy_site` / `deploy_server` columns; fails loudly on unmappable legacy rows |

Migrations are idempotent (`IF NOT EXISTS` / guarded `DO` blocks) so they apply
cleanly to both fresh (baseline-applied) and older databases. They are
irreversible (`reversible: false`, `rollback: forward-fix`) — dropping tables is
not data-preserving — and therefore ship without `.down.sql` files except the
historical `0001`.
