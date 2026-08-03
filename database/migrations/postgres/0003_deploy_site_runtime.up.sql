-- sdkwork:migration
-- id: 0003_deploy_site_runtime
-- engine: postgres
-- module: deploy
-- purpose: Create the live website composition tables (resource, variant,
--   mount, binding, revision, node target, runtime assignment, observation).
--   Added to the consolidated baseline between 2026-07-22 and 2026-07-23;
--   restored as forward migrations so databases initialized before those
--   dates converge through migrate instead of baseline replay.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

CREATE TABLE IF NOT EXISTS deploy_site_resource (
    id                        BIGINT PRIMARY KEY NOT NULL,
    uuid                      VARCHAR(36) NOT NULL,
    tenant_id                 BIGINT NOT NULL,
    organization_id           BIGINT NOT NULL DEFAULT 0,
    site_id                   BIGINT NOT NULL,
    resource_key              VARCHAR(64) NOT NULL,
    provider_type             VARCHAR(32) NOT NULL,
    provider_resource_uuid    VARCHAR(128) NOT NULL,
    provider_contract_version VARCHAR(64) NOT NULL,
    capabilities_json         JSONB NOT NULL DEFAULT '{}',
    status                    VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    last_validated_at         TIMESTAMPTZ NULL,
    last_error_code           VARCHAR(64) NULL,
    metadata                  JSONB NOT NULL DEFAULT '{}',
    created_by                BIGINT NULL,
    updated_by                BIGINT NULL,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version                   BIGINT NOT NULL DEFAULT 1,
    deleted_at                TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_site_resource_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_resource_key UNIQUE (site_id, resource_key),
    CONSTRAINT fk_deploy_site_resource_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT chk_deploy_site_resource_provider CHECK (provider_type IN ('DRIVE', 'KNOWLEDGEBASE')),
    CONSTRAINT chk_deploy_site_resource_status CHECK (status IN ('PENDING', 'VALID', 'INVALID', 'UNAVAILABLE', 'REVOKED'))
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_resource_site_status
    ON deploy_site_resource (tenant_id, site_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_site_resource_provider
    ON deploy_site_resource (tenant_id, provider_type, provider_resource_uuid)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_site_variant (
    id              BIGINT PRIMARY KEY NOT NULL,
    uuid            VARCHAR(36) NOT NULL,
    tenant_id       BIGINT NOT NULL,
    site_id         BIGINT NOT NULL,
    variant_key     VARCHAR(64) NOT NULL,
    label           VARCHAR(64) NOT NULL,
    client_class    VARCHAR(16) NOT NULL DEFAULT 'OTHER',
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    priority        INTEGER NOT NULL DEFAULT 0,
    status          VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_by      BIGINT NULL,
    updated_by      BIGINT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version         BIGINT NOT NULL DEFAULT 1,
    deleted_at      TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_site_variant_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_variant_key UNIQUE (site_id, variant_key),
    CONSTRAINT fk_deploy_site_variant_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT chk_deploy_site_variant_client CHECK (client_class IN ('DESKTOP', 'MOBILE', 'TABLET', 'TV', 'BOT', 'OTHER')),
    CONSTRAINT chk_deploy_site_variant_status CHECK (status IN ('ACTIVE', 'DISABLED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_variant_default
    ON deploy_site_variant (site_id)
    WHERE is_default = TRUE AND status = 'ACTIVE' AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_site_variant_site_priority
    ON deploy_site_variant (tenant_id, site_id, status, priority, uuid)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_site_variant_rule (
    id                BIGINT PRIMARY KEY NOT NULL,
    uuid              VARCHAR(36) NOT NULL,
    tenant_id         BIGINT NOT NULL,
    site_id           BIGINT NOT NULL,
    rule_key          VARCHAR(64) NOT NULL,
    target_variant_id BIGINT NOT NULL,
    rule_type         VARCHAR(16) NOT NULL,
    match_value       VARCHAR(4096) NOT NULL,
    priority          INTEGER NOT NULL DEFAULT 0,
    status            VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    created_by        BIGINT NULL,
    updated_by        BIGINT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version           BIGINT NOT NULL DEFAULT 1,
    deleted_at        TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_site_variant_rule_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_variant_rule_key UNIQUE (site_id, rule_key),
    CONSTRAINT fk_deploy_site_variant_rule_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_site_variant_rule_variant FOREIGN KEY (target_variant_id) REFERENCES deploy_site_variant(id),
    CONSTRAINT chk_deploy_site_variant_rule_type CHECK (rule_type IN ('PATH_PREFIX', 'CLIENT_CLASS')),
    CONSTRAINT chk_deploy_site_variant_rule_status CHECK (status IN ('ACTIVE', 'DISABLED')),
    CONSTRAINT chk_deploy_site_variant_rule_priority CHECK (priority BETWEEN 0 AND 65535)
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_variant_rule_order
    ON deploy_site_variant_rule (tenant_id, site_id, status, priority, uuid)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_site_mount (
    id                BIGINT PRIMARY KEY NOT NULL,
    uuid              VARCHAR(36) NOT NULL,
    tenant_id         BIGINT NOT NULL,
    site_id           BIGINT NOT NULL,
    mount_key         VARCHAR(64) NOT NULL,
    variant_id        BIGINT NOT NULL,
    resource_id       BIGINT NOT NULL,
    path_prefix       VARCHAR(4096) NOT NULL,
    resource_subpath  VARCHAR(4096) NOT NULL DEFAULT '/',
    mount_mode        VARCHAR(16) NOT NULL DEFAULT 'ROOT',
    handler_type      VARCHAR(16) NOT NULL,
    index_files_json  JSONB NOT NULL DEFAULT '[]',
    spa_fallback_path VARCHAR(4096) NULL,
    priority          INTEGER NOT NULL DEFAULT 0,
    status            VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    created_by        BIGINT NULL,
    updated_by        BIGINT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version           BIGINT NOT NULL DEFAULT 1,
    deleted_at        TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_site_mount_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_mount_key UNIQUE (site_id, mount_key),
    CONSTRAINT uk_deploy_site_mount_prefix UNIQUE (variant_id, path_prefix),
    CONSTRAINT fk_deploy_site_mount_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_site_mount_variant FOREIGN KEY (variant_id) REFERENCES deploy_site_variant(id),
    CONSTRAINT fk_deploy_site_mount_resource FOREIGN KEY (resource_id) REFERENCES deploy_site_resource(id),
    CONSTRAINT chk_deploy_site_mount_mode CHECK (mount_mode IN ('ROOT', 'ALIAS')),
    CONSTRAINT chk_deploy_site_mount_handler CHECK (handler_type IN ('STATIC', 'SPA', 'WIKI')),
    CONSTRAINT chk_deploy_site_mount_status CHECK (status IN ('ACTIVE', 'DISABLED', 'INVALID'))
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_mount_route
    ON deploy_site_mount (tenant_id, site_id, variant_id, status, path_prefix)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_site_binding (
    id                   BIGINT PRIMARY KEY NOT NULL,
    uuid                 VARCHAR(36) NOT NULL,
    tenant_id            BIGINT NOT NULL,
    organization_id      BIGINT NOT NULL DEFAULT 0,
    site_id              BIGINT NOT NULL,
    binding_key           VARCHAR(64) NOT NULL,
    domain_id            BIGINT NOT NULL,
    hostname_ascii       VARCHAR(255) NOT NULL,
    environment          VARCHAR(16) NOT NULL,
    path_prefix          VARCHAR(4096) NOT NULL DEFAULT '/',
    action_type          VARCHAR(16) NOT NULL DEFAULT 'SERVE',
    is_canonical         BOOLEAN NOT NULL DEFAULT FALSE,
    default_variant_id   BIGINT NULL,
    forced_variant_id    BIGINT NULL,
    redirect_scheme      VARCHAR(8) NULL,
    redirect_hostname    VARCHAR(255) NULL,
    redirect_path_prefix VARCHAR(4096) NULL,
    redirect_status_code INTEGER NULL,
    preserve_path        BOOLEAN NOT NULL DEFAULT TRUE,
    preserve_query       BOOLEAN NOT NULL DEFAULT TRUE,
    status               VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    verified_at          TIMESTAMPTZ NULL,
    activated_at         TIMESTAMPTZ NULL,
    created_by           BIGINT NULL,
    updated_by           BIGINT NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version              BIGINT NOT NULL DEFAULT 1,
    deleted_at           TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_site_binding_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_site_binding_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_site_binding_domain FOREIGN KEY (domain_id) REFERENCES deploy_domain(id),
    CONSTRAINT fk_deploy_site_binding_default_variant FOREIGN KEY (default_variant_id) REFERENCES deploy_site_variant(id),
    CONSTRAINT fk_deploy_site_binding_forced_variant FOREIGN KEY (forced_variant_id) REFERENCES deploy_site_variant(id),
    CONSTRAINT chk_deploy_site_binding_environment CHECK (environment IN ('development', 'test', 'staging', 'production')),
    CONSTRAINT chk_deploy_site_binding_action CHECK (action_type IN ('SERVE', 'REDIRECT')),
    CONSTRAINT chk_deploy_site_binding_status CHECK (status IN ('PENDING', 'VERIFIED', 'ACTIVE', 'PAUSED', 'FAILED', 'ARCHIVED')),
    CONSTRAINT chk_deploy_site_binding_redirect_status CHECK (redirect_status_code IS NULL OR redirect_status_code IN (301, 302, 307, 308))
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_binding_site_status
    ON deploy_site_binding (tenant_id, site_id, environment, status)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_binding_active_key
    ON deploy_site_binding (site_id, binding_key)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_binding_active_route
    ON deploy_site_binding (hostname_ascii, path_prefix, environment)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_binding_canonical
    ON deploy_site_binding (site_id, environment)
    WHERE is_canonical = TRUE AND status = 'ACTIVE' AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_site_revision (
    id                        BIGINT PRIMARY KEY NOT NULL,
    uuid                      VARCHAR(36) NOT NULL,
    tenant_id                 BIGINT NOT NULL,
    organization_id           BIGINT NOT NULL DEFAULT 0,
    site_id                   BIGINT NOT NULL,
    revision_no               BIGINT NOT NULL,
    environment               VARCHAR(16) NOT NULL,
    descriptor_schema_version VARCHAR(64) NOT NULL,
    descriptor_json           JSONB NOT NULL,
    descriptor_sha256         VARCHAR(64) NOT NULL,
    compiler_version          VARCHAR(128) NOT NULL,
    source_config_version     BIGINT NOT NULL,
    idempotency_key           VARCHAR(128) NOT NULL,
    request_sha256            VARCHAR(64) NOT NULL,
    result_json               JSONB NOT NULL DEFAULT '{}',
    validation_status         VARCHAR(16) NOT NULL DEFAULT 'VALID',
    validation_report_json    JSONB NOT NULL DEFAULT '{}',
    supersedes_revision_id    BIGINT NULL,
    created_by                BIGINT NULL,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_deploy_site_revision_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_revision_no UNIQUE (site_id, revision_no),
    CONSTRAINT uk_deploy_site_revision_hash UNIQUE (site_id, descriptor_sha256),
    CONSTRAINT uk_deploy_site_revision_idempotency UNIQUE (tenant_id, site_id, idempotency_key),
    CONSTRAINT fk_deploy_site_revision_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_site_revision_supersedes FOREIGN KEY (supersedes_revision_id) REFERENCES deploy_site_revision(id),
    CONSTRAINT chk_deploy_site_revision_environment CHECK (environment IN ('development', 'test', 'staging', 'production')),
    CONSTRAINT chk_deploy_site_revision_validation CHECK (validation_status IN ('VALID', 'INVALID'))
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_revision_site_created
    ON deploy_site_revision (tenant_id, site_id, revision_no DESC);

CREATE TABLE IF NOT EXISTS deploy_web_node_target (
    id                BIGINT PRIMARY KEY NOT NULL,
    uuid              VARCHAR(36) NOT NULL,
    tenant_id         BIGINT NOT NULL,
    node_uuid         VARCHAR(128) NOT NULL,
    environment       VARCHAR(16) NOT NULL,
    tenant_scope_hash VARCHAR(64) NOT NULL,
    region            VARCHAR(64) NULL,
    status            VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version           BIGINT NOT NULL DEFAULT 1,
    deleted_at        TIMESTAMPTZ NULL,
    CONSTRAINT uk_deploy_web_node_target_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_web_node_target_node UNIQUE (node_uuid, environment),
    CONSTRAINT chk_deploy_web_node_target_environment CHECK (environment IN ('development', 'test', 'staging', 'production')),
    CONSTRAINT chk_deploy_web_node_target_status CHECK (status IN ('ACTIVE', 'DRAINING', 'DISABLED'))
);

CREATE INDEX IF NOT EXISTS idx_deploy_web_node_target_tenant
    ON deploy_web_node_target (tenant_id, environment, status, node_uuid)
    WHERE deleted_at IS NULL;

-- Durable desired-state/outbox row. Web Server owns the delivery projection and Node observation;
-- Deployments retains the exact bytes and publication result needed for idempotent reconciliation.
CREATE TABLE IF NOT EXISTS deploy_runtime_assignment (
    id                     BIGINT PRIMARY KEY NOT NULL,
    uuid                   VARCHAR(36) NOT NULL,
    tenant_id              BIGINT NOT NULL,
    node_target_id         BIGINT NOT NULL,
    trigger_site_revision_id BIGINT NULL,
    generation             BIGINT NOT NULL,
    snapshot_uuid          VARCHAR(128) NOT NULL,
    snapshot_sha256        VARCHAR(64) NOT NULL,
    desired_state_sha256   VARCHAR(64) NOT NULL,
    runtime_set_json       JSONB NOT NULL,
    runtime_set_bytes      BIGINT NOT NULL,
    publish_status         VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    remote_assignment_uuid VARCHAR(128) NULL,
    attempt_count          INTEGER NOT NULL DEFAULT 0,
    next_attempt_at        TIMESTAMPTZ NULL,
    lease_owner            VARCHAR(128) NULL,
    lease_expires_at       TIMESTAMPTZ NULL,
    last_error_code        VARCHAR(64) NULL,
    published_at           TIMESTAMPTZ NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version                BIGINT NOT NULL DEFAULT 1,
    CONSTRAINT uk_deploy_runtime_assignment_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_runtime_assignment_generation UNIQUE (node_target_id, generation),
    CONSTRAINT uk_deploy_runtime_assignment_snapshot UNIQUE (snapshot_uuid),
    CONSTRAINT fk_deploy_runtime_assignment_target FOREIGN KEY (node_target_id) REFERENCES deploy_web_node_target(id),
    CONSTRAINT fk_deploy_runtime_assignment_revision FOREIGN KEY (trigger_site_revision_id) REFERENCES deploy_site_revision(id),
    CONSTRAINT chk_deploy_runtime_assignment_generation CHECK (generation BETWEEN 1 AND 9007199254740991),
    CONSTRAINT chk_deploy_runtime_assignment_bytes CHECK (runtime_set_bytes > 0 AND runtime_set_bytes <= 67108864),
    CONSTRAINT chk_deploy_runtime_assignment_status CHECK (publish_status IN ('PENDING', 'PUBLISHING', 'PUBLISHED', 'FAILED', 'SUPERSEDED')),
    CONSTRAINT chk_deploy_runtime_assignment_lease CHECK (
        (publish_status = 'PUBLISHING' AND lease_owner IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (publish_status <> 'PUBLISHING' AND lease_owner IS NULL AND lease_expires_at IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_runtime_assignment_delivery
    ON deploy_runtime_assignment (publish_status, next_attempt_at, lease_expires_at, created_at)
    WHERE publish_status IN ('PENDING', 'PUBLISHING', 'FAILED');

CREATE INDEX IF NOT EXISTS idx_deploy_runtime_assignment_target_latest
    ON deploy_runtime_assignment (tenant_id, node_target_id, generation DESC);

-- Authenticated, append-only evidence read from the Web-owned runtime observation API.
CREATE TABLE IF NOT EXISTS deploy_site_target_observation (
    id                       BIGINT PRIMARY KEY NOT NULL,
    uuid                     VARCHAR(36) NOT NULL,
    tenant_id                BIGINT NOT NULL,
    site_id                  BIGINT NULL,
    site_revision_id         BIGINT NULL,
    node_target_id           BIGINT NOT NULL,
    runtime_assignment_id    BIGINT NOT NULL,
    remote_observation_uuid  VARCHAR(128) NOT NULL,
    remote_assignment_uuid   VARCHAR(128) NOT NULL,
    generation               BIGINT NOT NULL,
    snapshot_uuid            VARCHAR(128) NOT NULL,
    snapshot_sha256          VARCHAR(64) NOT NULL,
    environment              VARCHAR(16) NOT NULL,
    state                    VARCHAR(16) NOT NULL,
    node_version             VARCHAR(64) NULL,
    reason_code              VARCHAR(64) NULL,
    detail                   VARCHAR(512) NULL,
    observed_at              TIMESTAMPTZ NOT NULL,
    ingested_at              TIMESTAMPTZ NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_deploy_site_target_observation_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_target_observation_remote UNIQUE (remote_observation_uuid),
    CONSTRAINT uk_deploy_site_target_observation_state UNIQUE (runtime_assignment_id, state),
    CONSTRAINT fk_deploy_site_target_observation_site FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT fk_deploy_site_target_observation_revision FOREIGN KEY (site_revision_id) REFERENCES deploy_site_revision(id),
    CONSTRAINT fk_deploy_site_target_observation_target FOREIGN KEY (node_target_id) REFERENCES deploy_web_node_target(id),
    CONSTRAINT fk_deploy_site_target_observation_assignment FOREIGN KEY (runtime_assignment_id) REFERENCES deploy_runtime_assignment(id),
    CONSTRAINT chk_deploy_site_target_observation_site_pair CHECK (
        (site_id IS NULL AND site_revision_id IS NULL)
        OR (site_id IS NOT NULL AND site_revision_id IS NOT NULL)
    ),
    CONSTRAINT chk_deploy_site_target_observation_generation CHECK (generation BETWEEN 1 AND 9007199254740991),
    CONSTRAINT chk_deploy_site_target_observation_snapshot_sha256 CHECK (snapshot_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_deploy_site_target_observation_environment CHECK (environment IN ('development', 'test', 'staging', 'production')),
    CONSTRAINT chk_deploy_site_target_observation_state CHECK (state IN ('RECEIVED', 'VALIDATED', 'STAGED', 'ACTIVE', 'REJECTED')),
    CONSTRAINT chk_deploy_site_target_observation_reason CHECK (
        (state = 'REJECTED' AND reason_code IS NOT NULL)
        OR (state <> 'REJECTED' AND reason_code IS NULL AND detail IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_site_target_observation_rollout
    ON deploy_site_target_observation (tenant_id, site_revision_id, state, node_target_id);

CREATE INDEX IF NOT EXISTS idx_deploy_site_target_observation_assignment
    ON deploy_site_target_observation (runtime_assignment_id, id DESC);
