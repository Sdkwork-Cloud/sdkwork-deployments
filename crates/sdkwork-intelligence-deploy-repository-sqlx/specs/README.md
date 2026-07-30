# Deploy SQLx Repository Component

`component.spec.json` declares the Deploy persistence adapters. This crate owns parameterized SQL,
row mapping, and database transactions only; environment loading and runtime composition are not
repository responsibilities.

The ignored PostgreSQL concurrency test requires `SDKWORK_DATABASE_TEST_POSTGRES_URL` to point to an
empty disposable database. Run it explicitly with:

```powershell
cargo test -p sdkwork-intelligence-deploy-repository-sqlx postgres_serializes_mutations_and_fences_assignment_leases -- --ignored --exact
```
