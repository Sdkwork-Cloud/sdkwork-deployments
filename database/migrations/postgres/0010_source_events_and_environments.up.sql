-- sdkwork:migration
-- id: 0010_source_events_and_environments
-- engine: postgres
-- module: deploy
-- purpose: CI event ingestion (Git webhook push events deduplicated per
--   commit, driving automatic builds) and the application environment model
--   with an immutable promotion history for the release promotion chain.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- Git webhook push events. Deduplicated per (repository, commit): one commit
-- triggers builds at most once even when the webhook redelivers.
CREATE TABLE IF NOT EXISTS deploy_source_event (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    source_repository_id BIGINT  NOT NULL,
    event_kind      VARCHAR(16)  NOT NULL,
    source_ref      VARCHAR(255) NOT NULL,
    source_commit   VARCHAR(64)  NOT NULL,
    commit_message  VARCHAR(2000),
    sender_ref      VARCHAR(512),
    payload_sha256  VARCHAR(64)  NOT NULL,
    event_status    VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    builds_triggered INTEGER     NOT NULL DEFAULT 0,
    error_code      VARCHAR(64),
    processed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_source_event PRIMARY KEY (id),
    CONSTRAINT fk_deploy_source_event_repository
        FOREIGN KEY (source_repository_id) REFERENCES deploy_source_repository(id),
    CONSTRAINT chk_deploy_source_event_commit CHECK (source_commit ~ '^[0-9a-f]{7,64}$'),
    CONSTRAINT chk_deploy_source_event_hash CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_deploy_source_event_kind CHECK (event_kind IN ('PUSH'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_source_event_uuid
    ON deploy_source_event (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_source_event_commit
    ON deploy_source_event (source_repository_id, source_commit);

CREATE INDEX IF NOT EXISTS idx_deploy_source_event_tenant_created
    ON deploy_source_event (tenant_id, created_at DESC);

-- Application environment in the promotion chain. Environments carry the
-- current release pointer; promotion moves a release through the chain with
-- an immutable history row.
CREATE TABLE IF NOT EXISTS deploy_app_environment (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    env_key         VARCHAR(32)  NOT NULL,
    env_name        VARCHAR(100) NOT NULL,
    env_level       VARCHAR(16)  NOT NULL,
    approval_required BOOLEAN    NOT NULL DEFAULT FALSE,
    current_release_id BIGINT    NULL,
    env_status      VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_app_environment PRIMARY KEY (id),
    CONSTRAINT fk_deploy_app_environment_app
        FOREIGN KEY (app_id) REFERENCES deploy_app(id),
    CONSTRAINT fk_deploy_app_environment_release
        FOREIGN KEY (current_release_id) REFERENCES deploy_release(id),
    CONSTRAINT chk_deploy_app_environment_level CHECK (env_level IN
        ('DEVELOPMENT', 'STAGING', 'PRODUCTION')),
    CONSTRAINT chk_deploy_app_environment_status CHECK (env_status IN
        ('DRAFT', 'ACTIVE', 'ARCHIVED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_environment_uuid
    ON deploy_app_environment (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_app_environment_key
    ON deploy_app_environment (app_id, env_key)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_app_environment_app
    ON deploy_app_environment (app_id, env_status);

-- Immutable promotion history: who moved which release into which
-- environment, optionally from a source environment (chain enforcement).
CREATE TABLE IF NOT EXISTS deploy_environment_promotion (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    app_id          BIGINT       NOT NULL,
    environment_id  BIGINT       NOT NULL,
    release_id      BIGINT       NOT NULL,
    from_environment_id BIGINT   NULL,
    promoted_by     BIGINT,
    note            VARCHAR(500),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_deploy_environment_promotion PRIMARY KEY (id),
    CONSTRAINT fk_deploy_environment_promotion_env
        FOREIGN KEY (environment_id) REFERENCES deploy_app_environment(id),
    CONSTRAINT fk_deploy_environment_promotion_release
        FOREIGN KEY (release_id) REFERENCES deploy_release(id),
    CONSTRAINT fk_deploy_environment_promotion_from_env
        FOREIGN KEY (from_environment_id) REFERENCES deploy_app_environment(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_environment_promotion_uuid
    ON deploy_environment_promotion (uuid);

CREATE INDEX IF NOT EXISTS idx_deploy_environment_promotion_env_created
    ON deploy_environment_promotion (environment_id, created_at DESC);
