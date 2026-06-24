-- Consolidated legacy baseline (SQLite adaptation)
-- Booleans as INTEGER, timestamps as ISO8601 TEXT, JSON as TEXT.

CREATE TABLE deploy_site (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    data_scope      INTEGER      NOT NULL DEFAULT 1,
    user_id         INTEGER,
    name            TEXT         NOT NULL,
    slug            TEXT         NOT NULL,
    description     TEXT,
    site_type       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 0,
    runtime_config  TEXT         NOT NULL DEFAULT '{}',
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    deleted_at      TEXT,
    deleted_by      INTEGER,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_site_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_slug UNIQUE (tenant_id, slug),
    CONSTRAINT chk_deploy_site_type CHECK (site_type BETWEEN 1 AND 6),
    CONSTRAINT chk_deploy_site_status CHECK (status BETWEEN 0 AND 3)
);

CREATE INDEX idx_deploy_site_tenant_status_updated
    ON deploy_site (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX idx_deploy_site_user_updated
    ON deploy_site (tenant_id, user_id, updated_at DESC);

CREATE INDEX idx_deploy_site_slug
    ON deploy_site (tenant_id, slug);

CREATE TABLE deploy_domain (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    hostname        TEXT         NOT NULL,
    is_primary      INTEGER      NOT NULL DEFAULT 0,
    is_verified     INTEGER      NOT NULL DEFAULT 0,
    verify_token    TEXT,
    ssl_enabled     INTEGER      NOT NULL DEFAULT 0,
    ssl_provider    TEXT,
    redirect_target TEXT,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    deleted_at      TEXT,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_domain_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_domain_hostname UNIQUE (hostname),
    CONSTRAINT fk_deploy_domain_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE INDEX idx_deploy_domain_site
    ON deploy_domain (site_id);

CREATE INDEX idx_deploy_domain_tenant_status
    ON deploy_domain (tenant_id, status);

CREATE TABLE deploy_nginx_config (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    domain_id       INTEGER,
    config_type     INTEGER      NOT NULL DEFAULT 1,
    config_name     TEXT         NOT NULL,
    config_content  TEXT         NOT NULL,
    config_hash     TEXT         NOT NULL,
    is_active       INTEGER      NOT NULL DEFAULT 0,
    version_no      INTEGER      NOT NULL DEFAULT 1,
    deployed_at     TEXT,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_nginx_config_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_nginx_config_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE INDEX idx_deploy_nginx_config_site_active
    ON deploy_nginx_config (site_id, is_active);

CREATE INDEX idx_deploy_nginx_config_type_status
    ON deploy_nginx_config (config_type, status);

CREATE TABLE deploy_certificate (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    domain_id       INTEGER,
    site_id         INTEGER,
    cert_name       TEXT         NOT NULL,
    cert_type       INTEGER      NOT NULL DEFAULT 1,
    issuer          TEXT,
    subject         TEXT,
    san_list        TEXT,
    fingerprint     TEXT,
    cert_path       TEXT,
    key_path        TEXT,
    chain_path      TEXT,
    not_before      TEXT,
    not_after       TEXT,
    auto_renew      INTEGER      NOT NULL DEFAULT 1,
    renewal_status  INTEGER      NOT NULL DEFAULT 0,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_uuid UNIQUE (uuid)
);

CREATE INDEX idx_deploy_certificate_domain
    ON deploy_certificate (domain_id);

CREATE INDEX idx_deploy_certificate_expiry
    ON deploy_certificate (not_after)
    WHERE status = 1;

CREATE INDEX idx_deploy_certificate_renewal
    ON deploy_certificate (renewal_status, not_after)
    WHERE auto_renew = 1 AND status = 1;

CREATE TABLE deploy_deployment (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    user_id         INTEGER,
    site_id         INTEGER      NOT NULL,
    deploy_type     INTEGER      NOT NULL DEFAULT 1,
    version_tag     TEXT,
    commit_hash     TEXT,
    source_ref      TEXT,
    build_log       TEXT,
    deploy_log      TEXT,
    artifact_path   TEXT,
    artifact_size   INTEGER,
    artifact_hash   TEXT,
    environment     TEXT         NOT NULL DEFAULT 'production',
    status          INTEGER      NOT NULL DEFAULT 0,
    started_at      TEXT,
    completed_at    TEXT,
    duration_ms     INTEGER,
    rollback_from   INTEGER,
    idempotency_key TEXT,
    request_id      TEXT,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_deployment_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_deployment_idempotency UNIQUE (tenant_id, idempotency_key),
    CONSTRAINT fk_deploy_deployment_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE INDEX idx_deploy_deployment_site_created
    ON deploy_deployment (site_id, created_at DESC);

CREATE INDEX idx_deploy_deployment_tenant_status
    ON deploy_deployment (tenant_id, status, created_at DESC);

CREATE INDEX idx_deploy_deployment_status
    ON deploy_deployment (status)
    WHERE status IN (0, 1, 2);

CREATE TABLE deploy_env_variable (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    environment     TEXT         NOT NULL DEFAULT 'production',
    key             TEXT         NOT NULL,
    value_encrypted TEXT         NOT NULL,
    is_secret       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_env_variable_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_env_variable_key UNIQUE (site_id, environment, key)
);

CREATE INDEX idx_deploy_env_variable_site_env
    ON deploy_env_variable (site_id, environment);

CREATE TABLE deploy_health_check (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    site_id         INTEGER      NOT NULL,
    domain_id       INTEGER,
    check_type      INTEGER      NOT NULL DEFAULT 1,
    check_url       TEXT,
    check_interval  INTEGER      NOT NULL DEFAULT 60,
    timeout_ms      INTEGER      NOT NULL DEFAULT 5000,
    retry_count     INTEGER      NOT NULL DEFAULT 3,
    expected_status INTEGER,
    expected_body   TEXT,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_health_check_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_health_check_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE INDEX idx_deploy_health_check_site
    ON deploy_health_check (site_id);

CREATE TABLE deploy_health_result (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    health_check_id INTEGER      NOT NULL,
    site_id         INTEGER      NOT NULL,
    is_healthy      INTEGER      NOT NULL,
    response_ms     INTEGER,
    status_code     INTEGER,
    error_message   TEXT,
    checked_at      TEXT         NOT NULL,
    created_at      TEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX idx_deploy_health_result_check_time
    ON deploy_health_result (health_check_id, checked_at DESC);

CREATE INDEX idx_deploy_health_result_site_time
    ON deploy_health_result (site_id, checked_at DESC);

CREATE TABLE deploy_audit_log (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    organization_id INTEGER      NOT NULL DEFAULT 0,
    operator_id     INTEGER      NOT NULL,
    operator_type   TEXT         NOT NULL DEFAULT 'USER',
    action          TEXT         NOT NULL,
    target_type     TEXT         NOT NULL,
    target_id       INTEGER,
    target_uuid     TEXT,
    request_id      TEXT,
    ip_address      TEXT,
    user_agent      TEXT,
    changes         TEXT,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX idx_deploy_audit_log_target
    ON deploy_audit_log (target_type, target_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_operator
    ON deploy_audit_log (operator_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_tenant_action
    ON deploy_audit_log (tenant_id, action, created_at DESC);

CREATE TABLE deploy_server (
    id              INTEGER      NOT NULL,
    uuid            TEXT         NOT NULL,
    tenant_id       INTEGER      NOT NULL DEFAULT 0,
    name            TEXT         NOT NULL,
    host            TEXT         NOT NULL,
    ssh_port        INTEGER      NOT NULL DEFAULT 22,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        TEXT         NOT NULL DEFAULT '{}',
    created_at      TEXT         NOT NULL,
    updated_at      TEXT         NOT NULL,
    version         INTEGER      NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_server_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_server_host UNIQUE (tenant_id, host)
);

CREATE INDEX idx_deploy_server_tenant_status
    ON deploy_server (tenant_id, status, updated_at DESC);
