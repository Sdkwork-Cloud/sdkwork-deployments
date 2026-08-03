-- sdkwork:migration
-- id: 0002_deploy_artifact_release
-- engine: postgres
-- module: deploy
-- purpose: Create immutable artifact and site release tables, plus the
--   deploy_deployment.release_id link. Folded into the consolidated baseline
--   on 2026-07-01; restored as a forward migration so databases initialized
--   before that date converge through migrate instead of baseline replay.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- Immutable artifacts from completed package upload sessions
CREATE TABLE IF NOT EXISTS deploy_artifact (
    id BIGINT PRIMARY KEY NOT NULL,
    uuid VARCHAR(36) NOT NULL,
    tenant_id BIGINT NOT NULL,
    site_id BIGINT NULL,
    upload_session_ref_id BIGINT NOT NULL,
    package_type INTEGER NOT NULL,
    file_name VARCHAR(500) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    content_length BIGINT NOT NULL,
    checksum_sha256 VARCHAR(128) NULL,
    drive_node_id VARCHAR(128) NOT NULL,
    drive_space_id VARCHAR(128) NULL,
    drive_path VARCHAR(500) NOT NULL,
    status INTEGER NOT NULL DEFAULT 1,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT fk_deploy_artifact_upload_session
        FOREIGN KEY (upload_session_ref_id) REFERENCES deploy_upload_session_ref(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_artifact_uuid
    ON deploy_artifact (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_artifact_upload_session
    ON deploy_artifact (upload_session_ref_id);

CREATE INDEX IF NOT EXISTS idx_deploy_artifact_tenant_created
    ON deploy_artifact (tenant_id, created_at DESC);

-- Immutable site releases referencing one artifact
CREATE TABLE IF NOT EXISTS deploy_release (
    id BIGINT PRIMARY KEY NOT NULL,
    uuid VARCHAR(36) NOT NULL,
    tenant_id BIGINT NOT NULL,
    site_id BIGINT NOT NULL,
    artifact_id BIGINT NOT NULL,
    version_tag VARCHAR(100) NULL,
    status INTEGER NOT NULL DEFAULT 1,
    idempotency_key VARCHAR(128) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT fk_deploy_release_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_release_artifact FOREIGN KEY (artifact_id) REFERENCES deploy_artifact(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_release_uuid
    ON deploy_release (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_release_idempotency
    ON deploy_release (tenant_id, site_id, idempotency_key);

CREATE INDEX IF NOT EXISTS idx_deploy_release_site_created
    ON deploy_release (site_id, created_at DESC);

ALTER TABLE deploy_deployment
    ADD COLUMN IF NOT EXISTS release_id BIGINT NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_deployment_release
    ON deploy_deployment (release_id)
    WHERE release_id IS NOT NULL;
