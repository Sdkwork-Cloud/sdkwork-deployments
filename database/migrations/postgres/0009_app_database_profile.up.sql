-- sdkwork:migration
-- id: 0009_app_database_profile
-- engine: postgres
-- module: deploy
-- purpose: Add the application database structure contract (REQ-2026-0002
--   extension): an App may declare a database profile (engine, catalog, and
--   schema contract version) plus versioned migration definitions whose
--   checksums bind the app release to its data structure. Deploy stores the
--   definitions; the runtime executes them against the declared engine.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- Database structure contract of an App (at most one active profile per app).
CREATE TABLE IF NOT EXISTS deploy_app_database_profile (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    profile_key     VARCHAR(120) NOT NULL,
    db_engine       VARCHAR(16)  NOT NULL,
    catalog_name    VARCHAR(128) NOT NULL,
    schema_version  VARCHAR(64),
    baseline_version VARCHAR(64),
    migration_strategy VARCHAR(24) NOT NULL DEFAULT 'VERSIONED',
    profile_status  VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_app_database_profile PRIMARY KEY (id),
    CONSTRAINT fk_deploy_app_database_profile_app
        FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT ck_deploy_app_database_profile_engine
        CHECK (length(db_engine) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_database_profile_uuid
    ON deploy_app_database_profile (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_database_profile_key
    ON deploy_app_database_profile (app_id, profile_key)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_app_database_profile_app
    ON deploy_app_database_profile (app_id, profile_status);

-- Versioned migration definitions bound to a profile. Checksums are the
-- release-to-schema binding evidence: a release ships with the exact
-- migration set and checksums recorded here.
CREATE TABLE IF NOT EXISTS deploy_app_database_migration (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    profile_id      BIGINT       NOT NULL,
    migration_version VARCHAR(64) NOT NULL,
    migration_name  VARCHAR(200) NOT NULL,
    checksum_sha256 VARCHAR(64)  NOT NULL,
    script_ref      VARCHAR(512),
    migration_status VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    applied_at      TIMESTAMPTZ,
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_app_database_migration PRIMARY KEY (id),
    CONSTRAINT fk_deploy_app_database_migration_profile
        FOREIGN KEY (profile_id) REFERENCES deploy_app_database_profile(id),
    CONSTRAINT ck_deploy_app_database_migration_checksum
        CHECK (length(checksum_sha256) = 64)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_database_migration_uuid
    ON deploy_app_database_migration (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_database_migration_version
    ON deploy_app_database_migration (profile_id, migration_version)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_app_database_migration_profile
    ON deploy_app_database_migration (profile_id, migration_version);
