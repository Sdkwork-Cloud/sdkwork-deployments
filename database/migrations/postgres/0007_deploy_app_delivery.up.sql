-- sdkwork:migration
-- id: 0007_deploy_app_delivery
-- engine: postgres
-- module: deploy
-- purpose: Create the unified application delivery tables (app aggregate,
--   platform targets, source repositories, build templates, builds, packages,
--   release channels, channel rollouts, signing identities) and add the
--   new-model linkage columns to deploy_site, deploy_release, and
--   deploy_deployment. All new columns are additive; legacy rows remain
--   readable without backfill.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- Tenant-owned application aggregate (REQ-2026-0002)
CREATE TABLE IF NOT EXISTS deploy_app (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    name            VARCHAR(200) NOT NULL,
    slug            VARCHAR(120) NOT NULL,
    app_kind        VARCHAR(32)  NOT NULL,
    description     VARCHAR(2000),
    app_status      VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    site_id         BIGINT       NULL,
    default_environment VARCHAR(16) NOT NULL DEFAULT 'production',
    activated_at    TIMESTAMPTZ,
    paused_at       TIMESTAMPTZ,
    archived_at     TIMESTAMPTZ,
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_app PRIMARY KEY (id),
    CONSTRAINT fk_deploy_app_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_uuid
    ON deploy_app (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_tenant_slug
    ON deploy_app (tenant_id, slug)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_app_tenant_status_updated
    ON deploy_app (tenant_id, app_status, updated_at DESC);

-- Platform delivery unit inside an App
CREATE TABLE IF NOT EXISTS deploy_app_platform_target (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    target_key      VARCHAR(120) NOT NULL,
    platform        VARCHAR(16)  NOT NULL,
    tech_stack      VARCHAR(16)  NOT NULL DEFAULT 'OTHER',
    bundle_id       VARCHAR(255),
    package_name    VARCHAR(255),
    app_id_value    VARCHAR(255),
    bundle_name     VARCHAR(255),
    build_template_id BIGINT     NULL,
    allowed_channels_json JSONB  NOT NULL DEFAULT '[]',
    target_status   VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_app_platform_target PRIMARY KEY (id),
    CONSTRAINT fk_deploy_app_platform_target_app
        FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT fk_deploy_app_platform_target_template
        FOREIGN KEY (build_template_id) REFERENCES deploy_build_template(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_platform_target_uuid
    ON deploy_app_platform_target (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_platform_target_key
    ON deploy_app_platform_target (app_id, target_key)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_app_platform_target_app
    ON deploy_app_platform_target (app_id, target_status);

-- Git source repository binding
CREATE TABLE IF NOT EXISTS deploy_source_repository (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    repo_key        VARCHAR(120) NOT NULL,
    repo_provider   VARCHAR(32)  NOT NULL,
    repo_url        VARCHAR(1000) NOT NULL,
    default_branch  VARCHAR(255) NOT NULL DEFAULT 'main',
    clone_mode      VARCHAR(16)  NOT NULL DEFAULT 'SHALLOW',
    credential_secret_ref VARCHAR(512),
    repo_status     VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    last_validated_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_source_repository PRIMARY KEY (id),
    CONSTRAINT fk_deploy_source_repository_app
        FOREIGN KEY (app_id) REFERENCES deploy_app(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_source_repository_uuid
    ON deploy_source_repository (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_source_repository_key
    ON deploy_source_repository (app_id, repo_key)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_source_repository_tenant_url
    ON deploy_source_repository (tenant_id, repo_url)
    WHERE repo_status = 'VALIDATED';

-- Governed build recipe
CREATE TABLE IF NOT EXISTS deploy_build_template (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    template_name   VARCHAR(200) NOT NULL,
    template_version VARCHAR(64) NOT NULL,
    platform        VARCHAR(16)  NOT NULL,
    tech_stack      VARCHAR(16)  NOT NULL DEFAULT 'OTHER',
    toolchain_json  JSONB        NOT NULL DEFAULT '{}',
    commands_json   JSONB        NOT NULL DEFAULT '[]',
    artifact_outputs_json JSONB  NOT NULL DEFAULT '[]',
    quality_gates_json JSONB    NOT NULL DEFAULT '{}',
    template_status VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_build_template PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_build_template_uuid
    ON deploy_build_template (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_build_template_name_version
    ON deploy_build_template (tenant_id, template_name, template_version)
    WHERE deleted_at IS NULL;

-- Build execution record with monotonic build_number per (App, platform target)
CREATE TABLE IF NOT EXISTS deploy_build (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    platform_target_id BIGINT    NOT NULL,
    template_id     BIGINT       NULL,
    build_number    BIGINT       NOT NULL,
    source_repository_id BIGINT  NULL,
    source_ref      VARCHAR(255),
    source_snapshot_json JSONB   NOT NULL DEFAULT '{}',
    build_status    VARCHAR(16)  NOT NULL DEFAULT 'QUEUED',
    log_ref         VARCHAR(512),
    produced_package_id BIGINT   NULL,
    quality_gate_json JSONB      NOT NULL DEFAULT '{}',
    runner_node_uuid VARCHAR(64),
    runner_version  VARCHAR(64),
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ,
    duration_ms     BIGINT,
    error_code      VARCHAR(128),
    idempotency_key VARCHAR(128),
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_build PRIMARY KEY (id),
    CONSTRAINT fk_deploy_build_app FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT fk_deploy_build_target
        FOREIGN KEY (platform_target_id) REFERENCES deploy_app_platform_target(id),
    CONSTRAINT fk_deploy_build_template
        FOREIGN KEY (template_id) REFERENCES deploy_build_template(id),
    CONSTRAINT fk_deploy_build_repository
        FOREIGN KEY (source_repository_id) REFERENCES deploy_source_repository(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_build_uuid
    ON deploy_build (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_build_app_target_number
    ON deploy_build (app_id, platform_target_id, build_number);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_build_idempotency
    ON deploy_build (tenant_id, app_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_build_app_status_updated
    ON deploy_build (app_id, build_status, updated_at);

CREATE INDEX IF NOT EXISTS idx_deploy_build_app_created
    ON deploy_build (app_id, created_at DESC);

-- Signing identity: metadata and opaque secret reference only
CREATE TABLE IF NOT EXISTS deploy_signing_identity (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    identity_name   VARCHAR(200) NOT NULL,
    signing_kind    VARCHAR(32)  NOT NULL,
    platform_target_id BIGINT    NULL,
    fingerprint_sha256 VARCHAR(128),
    expires_at      TIMESTAMPTZ,
    secret_ref      VARCHAR(512),
    identity_status VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_signing_identity PRIMARY KEY (id),
    CONSTRAINT fk_deploy_signing_identity_target
        FOREIGN KEY (platform_target_id) REFERENCES deploy_app_platform_target(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_signing_identity_uuid
    ON deploy_signing_identity (uuid);

CREATE INDEX IF NOT EXISTS idx_deploy_signing_identity_tenant_status
    ON deploy_signing_identity (tenant_id, identity_status, expires_at);

-- Immutable standardized deployment package
CREATE TABLE IF NOT EXISTS deploy_package (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    platform_target_id BIGINT    NOT NULL,
    build_id        BIGINT       NOT NULL,
    package_format  VARCHAR(32)  NOT NULL,
    semantic_version VARCHAR(64) NOT NULL,
    package_size_bytes BIGINT    NOT NULL,
    checksum_sha256 VARCHAR(128) NOT NULL,
    manifest_sha256 VARCHAR(128) NOT NULL,
    drive_ref_json  JSONB        NOT NULL DEFAULT '{}',
    signing_identity_id BIGINT   NULL,
    min_platform_version VARCHAR(64),
    arch_json       JSONB        NOT NULL DEFAULT '[]',
    bundle_identity_json JSONB   NOT NULL DEFAULT '{}',
    package_status  VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    validation_report_json JSONB NOT NULL DEFAULT '{}',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_package PRIMARY KEY (id),
    CONSTRAINT fk_deploy_package_app FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT fk_deploy_package_target
        FOREIGN KEY (platform_target_id) REFERENCES deploy_app_platform_target(id),
    CONSTRAINT fk_deploy_package_build FOREIGN KEY (build_id) REFERENCES deploy_build(id),
    CONSTRAINT fk_deploy_package_signing
        FOREIGN KEY (signing_identity_id) REFERENCES deploy_signing_identity(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_package_uuid
    ON deploy_package (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_package_identity
    ON deploy_package (app_id, platform_target_id, semantic_version, build_id);

CREATE INDEX IF NOT EXISTS idx_deploy_package_app_target_created
    ON deploy_package (app_id, platform_target_id, created_at DESC);

-- Release channel current pointer
CREATE TABLE IF NOT EXISTS deploy_release_channel (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    platform_target_id BIGINT    NOT NULL,
    channel_key     VARCHAR(32)  NOT NULL,
    current_release_id BIGINT    NULL,
    channel_status  VARCHAR(16)  NOT NULL DEFAULT 'ACTIVE',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_release_channel PRIMARY KEY (id),
    CONSTRAINT fk_deploy_release_channel_app
        FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT fk_deploy_release_channel_target
        FOREIGN KEY (platform_target_id) REFERENCES deploy_app_platform_target(id),
    CONSTRAINT fk_deploy_release_channel_release
        FOREIGN KEY (current_release_id) REFERENCES deploy_release(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_release_channel_uuid
    ON deploy_release_channel (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_release_channel_scope
    ON deploy_release_channel (app_id, platform_target_id, channel_key)
    WHERE deleted_at IS NULL;

-- Immutable channel assignment/promotion history
CREATE TABLE IF NOT EXISTS deploy_channel_rollout (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    channel_id      BIGINT       NOT NULL,
    release_id      BIGINT       NOT NULL,
    strategy        VARCHAR(24)  NOT NULL DEFAULT 'IMMEDIATE',
    percentage      INTEGER      NULL,
    rollout_status  VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    supersedes_rollout_id BIGINT NULL,
    requested_by    BIGINT,
    requested_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_deploy_channel_rollout PRIMARY KEY (id),
    CONSTRAINT fk_deploy_channel_rollout_channel
        FOREIGN KEY (channel_id) REFERENCES deploy_release_channel(id),
    CONSTRAINT fk_deploy_channel_rollout_release
        FOREIGN KEY (release_id) REFERENCES deploy_release(id),
    CONSTRAINT ck_deploy_channel_rollout_percentage
        CHECK (percentage IS NULL OR (percentage >= 1 AND percentage <= 100))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_channel_rollout_uuid
    ON deploy_channel_rollout (uuid);

CREATE INDEX IF NOT EXISTS idx_deploy_channel_rollout_channel_created
    ON deploy_channel_rollout (channel_id, created_at DESC);

-- New-model linkage columns on existing tables (all nullable; legacy rows unchanged)
ALTER TABLE deploy_site
    ADD COLUMN IF NOT EXISTS app_id BIGINT NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_site_app
    ON deploy_site (app_id)
    WHERE app_id IS NOT NULL;

ALTER TABLE deploy_release
    ADD COLUMN IF NOT EXISTS app_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS platform_target_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS semantic_version VARCHAR(64) NULL,
    ADD COLUMN IF NOT EXISTS build_number BIGINT NULL,
    ADD COLUMN IF NOT EXISTS release_status VARCHAR(16) NULL,
    ADD COLUMN IF NOT EXISTS release_notes_json JSONB NOT NULL DEFAULT '{}';

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_release_app_target_version
    ON deploy_release (app_id, platform_target_id, semantic_version)
    WHERE app_id IS NOT NULL AND platform_target_id IS NOT NULL AND semantic_version IS NOT NULL;

ALTER TABLE deploy_deployment
    ADD COLUMN IF NOT EXISTS app_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS platform_target_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS deployment_kind VARCHAR(32) NULL,
    ADD COLUMN IF NOT EXISTS deployment_target VARCHAR(32) NULL,
    ADD COLUMN IF NOT EXISTS strategy VARCHAR(24) NULL,
    ADD COLUMN IF NOT EXISTS percentage INTEGER NULL,
    ADD COLUMN IF NOT EXISTS platform_review_ref VARCHAR(255) NULL,
    ADD COLUMN IF NOT EXISTS deployment_status VARCHAR(24) NULL,
    ADD COLUMN IF NOT EXISTS rollback_from_deployment_id BIGINT NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_deployment_app_created
    ON deploy_deployment (app_id, created_at DESC)
    WHERE app_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_deployment_status_active
    ON deploy_deployment (deployment_status)
    WHERE deployment_status IN ('PENDING', 'SUBMITTING', 'PENDING_REVIEW', 'IN_REVIEW', 'ROLLING');
