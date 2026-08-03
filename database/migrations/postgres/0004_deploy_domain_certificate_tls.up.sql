-- sdkwork:migration
-- id: 0004_deploy_domain_certificate_tls
-- engine: postgres
-- module: deploy
-- purpose: Create the DNS zone, domain verification, ACME certificate
--   lifecycle, and TLS runtime tables, plus the post-create certificate
--   version linkage columns. Added to the consolidated baseline on
--   2026-07-30; restored as forward migrations so databases initialized
--   before that date converge through migrate instead of baseline replay.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

CREATE TABLE IF NOT EXISTS deploy_dns_zone (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    apex_hostname   VARCHAR(253) NOT NULL,
    display_name    VARCHAR(200),
    dns_provider    VARCHAR(64),
    provider_zone_ref VARCHAR(512),
    status          VARCHAR(16)  NOT NULL DEFAULT 'ACTIVE',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version         BIGINT       NOT NULL DEFAULT 1,
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_dns_zone_uuid UNIQUE (uuid),
    CONSTRAINT chk_deploy_dns_zone_status CHECK (status IN ('ACTIVE', 'PAUSED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_dns_zone_active_apex
    ON deploy_dns_zone (apex_hostname)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_dns_zone_tenant_updated
    ON deploy_dns_zone (tenant_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_domain_verification (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    domain_id             BIGINT       NOT NULL,
    method                VARCHAR(16)  NOT NULL,
    record_name           VARCHAR(253) NOT NULL,
    proof_sha256          VARCHAR(64)  NOT NULL,
    status                VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    observed_sha256       VARCHAR(64),
    verifier_identity     VARCHAR(128),
    attempt_count         INTEGER      NOT NULL DEFAULT 0,
    next_attempt_at       TIMESTAMPTZ,
    expires_at            TIMESTAMPTZ  NOT NULL,
    checked_at            TIMESTAMPTZ,
    verified_at           TIMESTAMPTZ,
    failure_code          VARCHAR(64),
    created_by            BIGINT,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_domain_verification_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_domain_verification_domain FOREIGN KEY (domain_id) REFERENCES deploy_domain(id),
    CONSTRAINT chk_deploy_domain_verification_method CHECK (method IN ('DNS_TXT', 'DNS_CNAME', 'HTTP_FILE')),
    CONSTRAINT chk_deploy_domain_verification_status CHECK (status IN ('PENDING', 'CHECKING', 'VERIFIED', 'FAILED', 'EXPIRED')),
    CONSTRAINT chk_deploy_domain_verification_hash CHECK (
        proof_sha256 ~ '^[0-9a-f]{64}$'
        AND (observed_sha256 IS NULL OR observed_sha256 ~ '^[0-9a-f]{64}$')
    ),
    CONSTRAINT chk_deploy_domain_verification_attempts CHECK (attempt_count BETWEEN 0 AND 1000)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_domain_verification_active
    ON deploy_domain_verification (domain_id)
    WHERE status IN ('PENDING', 'CHECKING');

CREATE INDEX IF NOT EXISTS idx_deploy_domain_verification_due
    ON deploy_domain_verification (status, next_attempt_at, expires_at, id)
    WHERE status IN ('PENDING', 'CHECKING');

CREATE TABLE IF NOT EXISTS deploy_acme_account (
    id                 BIGINT        NOT NULL,
    uuid               VARCHAR(36)   NOT NULL,
    tenant_id          BIGINT        NOT NULL,
    ca_profile         VARCHAR(32)   NOT NULL,
    directory_url      VARCHAR(2048) NOT NULL,
    contact_email      VARCHAR(320)  NOT NULL,
    external_account_digest VARCHAR(64),
    account_key_secret_ref VARCHAR(1024) NOT NULL,
    status             VARCHAR(16)   NOT NULL DEFAULT 'ACTIVE',
    created_at         TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    version            BIGINT        NOT NULL DEFAULT 1,
    deleted_at         TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_acme_account_uuid UNIQUE (uuid),
    CONSTRAINT chk_deploy_acme_account_profile CHECK (ca_profile IN ('LETS_ENCRYPT_STAGING', 'LETS_ENCRYPT_PRODUCTION')),
    CONSTRAINT chk_deploy_acme_account_status CHECK (status IN ('ACTIVE', 'DISABLED', 'INVALID')),
    CONSTRAINT chk_deploy_acme_account_secret_ref CHECK (account_key_secret_ref LIKE 'secret://%'),
    CONSTRAINT chk_deploy_acme_account_external_digest CHECK (
        external_account_digest IS NULL OR external_account_digest ~ '^[0-9a-f]{64}$'
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_acme_account_tenant_profile_email
    ON deploy_acme_account (tenant_id, ca_profile, contact_email)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_certificate_order (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    certificate_id        BIGINT       NOT NULL,
    acme_account_id       BIGINT       NOT NULL,
    requested_version_no  BIGINT       NOT NULL,
    request_sha256        VARCHAR(64)  NOT NULL,
    idempotency_key       VARCHAR(128) NOT NULL,
    external_order_digest VARCHAR(64),
    status                VARCHAR(32)  NOT NULL DEFAULT 'REQUESTED',
    attempt_count         INTEGER      NOT NULL DEFAULT 0,
    next_attempt_at       TIMESTAMPTZ,
    lease_owner           VARCHAR(128),
    lease_expires_at      TIMESTAMPTZ,
    deadline_at           TIMESTAMPTZ  NOT NULL,
    last_error_code       VARCHAR(64),
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_order_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_certificate_order_idempotency UNIQUE (tenant_id, idempotency_key),
    CONSTRAINT uk_deploy_certificate_order_version UNIQUE (certificate_id, requested_version_no),
    CONSTRAINT fk_deploy_certificate_order_certificate FOREIGN KEY (certificate_id) REFERENCES deploy_certificate(id),
    CONSTRAINT fk_deploy_certificate_order_account FOREIGN KEY (acme_account_id) REFERENCES deploy_acme_account(id),
    CONSTRAINT chk_deploy_certificate_order_status CHECK (status IN (
        'REQUESTED', 'ACCOUNT_READY', 'ORDER_PENDING', 'CHALLENGE_PRESENTING',
        'CHALLENGE_VALIDATING', 'FINALIZING', 'VERSION_STORED', 'FAILED', 'CANCELLED'
    )),
    CONSTRAINT chk_deploy_certificate_order_hash CHECK (
        request_sha256 ~ '^[0-9a-f]{64}$'
        AND (external_order_digest IS NULL OR external_order_digest ~ '^[0-9a-f]{64}$')
    ),
    CONSTRAINT chk_deploy_certificate_order_attempts CHECK (attempt_count BETWEEN 0 AND 1000),
    CONSTRAINT chk_deploy_certificate_order_lease CHECK (
        (lease_owner IS NULL AND lease_expires_at IS NULL)
        OR (lease_owner IS NOT NULL AND lease_expires_at IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_order_due
    ON deploy_certificate_order (status, next_attempt_at, lease_expires_at, id)
    WHERE status NOT IN ('VERSION_STORED', 'FAILED', 'CANCELLED');

CREATE TABLE IF NOT EXISTS deploy_certificate_identifier (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    certificate_id  BIGINT       NOT NULL,
    domain_id       BIGINT       NOT NULL,
    identifier_type VARCHAR(16)  NOT NULL,
    hostname_ascii  VARCHAR(253) NOT NULL,
    position        INTEGER      NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_identifier_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_certificate_identifier_name UNIQUE (certificate_id, hostname_ascii),
    CONSTRAINT uk_deploy_certificate_identifier_position UNIQUE (certificate_id, position),
    CONSTRAINT fk_deploy_certificate_identifier_certificate FOREIGN KEY (certificate_id) REFERENCES deploy_certificate(id),
    CONSTRAINT fk_deploy_certificate_identifier_domain FOREIGN KEY (domain_id) REFERENCES deploy_domain(id),
    CONSTRAINT chk_deploy_certificate_identifier_type CHECK (identifier_type IN ('EXACT', 'WILDCARD')),
    CONSTRAINT chk_deploy_certificate_identifier_position CHECK (position BETWEEN 0 AND 99)
);

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_identifier_domain
    ON deploy_certificate_identifier (tenant_id, domain_id, certificate_id);

CREATE TABLE IF NOT EXISTS deploy_certificate_version (
    id                BIGINT       NOT NULL,
    uuid              VARCHAR(36)  NOT NULL,
    tenant_id         BIGINT       NOT NULL,
    certificate_id    BIGINT       NOT NULL,
    version_no        BIGINT       NOT NULL,
    serial_sha256     VARCHAR(64)  NOT NULL,
    fingerprint_sha256 VARCHAR(64) NOT NULL,
    spki_sha256       VARCHAR(64)  NOT NULL,
    chain_sha256      VARCHAR(64)  NOT NULL,
    issuer            VARCHAR(500) NOT NULL,
    subject           VARCHAR(500) NOT NULL,
    key_algorithm     VARCHAR(16)  NOT NULL,
    not_before        TIMESTAMPTZ  NOT NULL,
    not_after         TIMESTAMPTZ  NOT NULL,
    secret_bundle_ref VARCHAR(1024) NOT NULL,
    source_order_id   BIGINT,
    status            VARCHAR(16)  NOT NULL DEFAULT 'CANDIDATE',
    created_by        BIGINT,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_version_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_certificate_version_no UNIQUE (certificate_id, version_no),
    CONSTRAINT uk_deploy_certificate_version_fingerprint UNIQUE (tenant_id, fingerprint_sha256),
    CONSTRAINT fk_deploy_certificate_version_certificate FOREIGN KEY (certificate_id) REFERENCES deploy_certificate(id),
    CONSTRAINT chk_deploy_certificate_version_key_algorithm CHECK (key_algorithm IN ('RSA', 'ECDSA')),
    CONSTRAINT chk_deploy_certificate_version_status CHECK (status IN ('CANDIDATE', 'ACTIVE', 'SUPERSEDED', 'REVOKED', 'EXPIRED')),
    CONSTRAINT chk_deploy_certificate_version_validity CHECK (not_after > not_before),
    CONSTRAINT chk_deploy_certificate_version_hashes CHECK (
        serial_sha256 ~ '^[0-9a-f]{64}$'
        AND fingerprint_sha256 ~ '^[0-9a-f]{64}$'
        AND spki_sha256 ~ '^[0-9a-f]{64}$'
        AND chain_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT chk_deploy_certificate_version_secret_ref CHECK (
        secret_bundle_ref LIKE 'secret://%' OR secret_bundle_ref LIKE 'file:%'
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_version_lifecycle
    ON deploy_certificate_version (tenant_id, status, not_after, id);

CREATE TABLE IF NOT EXISTS deploy_certificate_challenge (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    order_id              BIGINT       NOT NULL,
    identifier_id         BIGINT       NOT NULL,
    challenge_type        VARCHAR(16)  NOT NULL,
    proof_sha256          VARCHAR(64)  NOT NULL,
    proof_secret_ref      VARCHAR(1024),
    presentation_ref      VARCHAR(1024),
    status                VARCHAR(24)  NOT NULL DEFAULT 'PENDING',
    attempt_count         INTEGER      NOT NULL DEFAULT 0,
    next_attempt_at       TIMESTAMPTZ,
    checked_at            TIMESTAMPTZ,
    validated_at          TIMESTAMPTZ,
    last_error_code       VARCHAR(64),
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_challenge_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_certificate_challenge_identifier UNIQUE (order_id, identifier_id),
    CONSTRAINT fk_deploy_certificate_challenge_order FOREIGN KEY (order_id) REFERENCES deploy_certificate_order(id),
    CONSTRAINT fk_deploy_certificate_challenge_identifier FOREIGN KEY (identifier_id) REFERENCES deploy_certificate_identifier(id),
    CONSTRAINT chk_deploy_certificate_challenge_type CHECK (challenge_type IN ('HTTP_01', 'DNS_01')),
    CONSTRAINT chk_deploy_certificate_challenge_status CHECK (status IN ('PENDING', 'PRESENTING', 'PRESENTED', 'VALIDATING', 'VALID', 'FAILED', 'CLEANED')),
    CONSTRAINT chk_deploy_certificate_challenge_hash CHECK (proof_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_deploy_certificate_challenge_secret_ref CHECK (
        proof_secret_ref IS NULL OR proof_secret_ref LIKE 'secret://%'
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_challenge_due
    ON deploy_certificate_challenge (status, next_attempt_at, id)
    WHERE status NOT IN ('VALID', 'FAILED', 'CLEANED');

CREATE TABLE IF NOT EXISTS deploy_certificate_distribution (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    certificate_version_id BIGINT      NOT NULL,
    node_target_id        BIGINT       NOT NULL,
    authorization_ref     VARCHAR(1024) NOT NULL,
    authorization_expires_at TIMESTAMPTZ NOT NULL,
    desired_state         VARCHAR(16)  NOT NULL DEFAULT 'PRESENT',
    status                VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_distribution_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_certificate_distribution_target UNIQUE (certificate_version_id, node_target_id),
    CONSTRAINT fk_deploy_certificate_distribution_version FOREIGN KEY (certificate_version_id) REFERENCES deploy_certificate_version(id),
    CONSTRAINT fk_deploy_certificate_distribution_target FOREIGN KEY (node_target_id) REFERENCES deploy_web_node_target(id),
    CONSTRAINT chk_deploy_certificate_distribution_auth_ref CHECK (authorization_ref LIKE 'secret://%'),
    CONSTRAINT chk_deploy_certificate_distribution_desired CHECK (desired_state IN ('PRESENT', 'ABSENT')),
    CONSTRAINT chk_deploy_certificate_distribution_status CHECK (status IN ('PENDING', 'AUTHORIZED', 'MATERIAL_READY', 'FAILED', 'EXPIRED', 'REVOKED'))
);

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_distribution_expiry
    ON deploy_certificate_distribution (status, authorization_expires_at, id)
    WHERE status IN ('PENDING', 'AUTHORIZED', 'MATERIAL_READY');

CREATE TABLE IF NOT EXISTS deploy_tls_policy (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    site_binding_id       BIGINT       NOT NULL,
    certificate_source    VARCHAR(16)  NOT NULL DEFAULT 'MANAGED',
    challenge_method      VARCHAR(16)  NOT NULL DEFAULT 'AUTO',
    minimum_tls_version   VARCHAR(8)   NOT NULL DEFAULT 'TLS1.2',
    maximum_tls_version   VARCHAR(8)   NOT NULL DEFAULT 'TLS1.3',
    alpn_json             JSONB        NOT NULL DEFAULT '["h2","http/1.1"]',
    auto_renew            BOOLEAN      NOT NULL DEFAULT TRUE,
    renew_before_days     INTEGER      NOT NULL DEFAULT 30,
    activation_quorum     INTEGER      NOT NULL DEFAULT 100,
    status                VARCHAR(16)  NOT NULL DEFAULT 'ACTIVE',
    created_by            BIGINT,
    updated_by            BIGINT,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    deleted_at            TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_tls_policy_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_tls_policy_binding FOREIGN KEY (site_binding_id) REFERENCES deploy_site_binding(id),
    CONSTRAINT chk_deploy_tls_policy_source CHECK (certificate_source IN ('MANAGED', 'CUSTOM', 'EXTERNAL')),
    CONSTRAINT chk_deploy_tls_policy_challenge CHECK (challenge_method IN ('AUTO', 'HTTP_01', 'DNS_01')),
    CONSTRAINT chk_deploy_tls_policy_versions CHECK (
        minimum_tls_version IN ('TLS1.2', 'TLS1.3')
        AND maximum_tls_version IN ('TLS1.2', 'TLS1.3')
        AND minimum_tls_version <= maximum_tls_version
    ),
    CONSTRAINT chk_deploy_tls_policy_renewal CHECK (renew_before_days BETWEEN 14 AND 90),
    CONSTRAINT chk_deploy_tls_policy_quorum CHECK (activation_quorum BETWEEN 1 AND 100),
    CONSTRAINT chk_deploy_tls_policy_status CHECK (status IN ('ACTIVE', 'PAUSED', 'ARCHIVED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_tls_policy_active_binding
    ON deploy_tls_policy (site_binding_id)
    WHERE status = 'ACTIVE' AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_listener_certificate_binding (
    id                    BIGINT      NOT NULL,
    uuid                  VARCHAR(36) NOT NULL,
    tenant_id             BIGINT      NOT NULL,
    site_binding_id       BIGINT      NOT NULL,
    certificate_id        BIGINT      NOT NULL,
    certificate_version_id BIGINT,
    key_algorithm         VARCHAR(16) NOT NULL,
    priority              INTEGER     NOT NULL DEFAULT 100,
    is_default            BOOLEAN     NOT NULL DEFAULT FALSE,
    status                VARCHAR(16) NOT NULL DEFAULT 'CANDIDATE',
    activated_at          TIMESTAMPTZ,
    created_by            BIGINT,
    updated_by            BIGINT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version               BIGINT      NOT NULL DEFAULT 1,
    deleted_at            TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_listener_certificate_binding_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_listener_certificate_binding_certificate UNIQUE (site_binding_id, certificate_id),
    CONSTRAINT fk_deploy_listener_certificate_binding_route FOREIGN KEY (site_binding_id) REFERENCES deploy_site_binding(id),
    CONSTRAINT fk_deploy_listener_certificate_binding_certificate FOREIGN KEY (certificate_id) REFERENCES deploy_certificate(id),
    CONSTRAINT fk_deploy_listener_certificate_binding_version FOREIGN KEY (certificate_version_id) REFERENCES deploy_certificate_version(id),
    CONSTRAINT chk_deploy_listener_certificate_binding_algorithm CHECK (key_algorithm IN ('RSA', 'ECDSA')),
    CONSTRAINT chk_deploy_listener_certificate_binding_priority CHECK (priority BETWEEN 0 AND 10000),
    CONSTRAINT chk_deploy_listener_certificate_binding_status CHECK (status IN ('CANDIDATE', 'ACTIVE', 'PAUSED', 'FAILED', 'ARCHIVED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_listener_certificate_binding_active_algorithm
    ON deploy_listener_certificate_binding (site_binding_id, key_algorithm)
    WHERE status = 'ACTIVE' AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_listener_certificate_binding_certificate
    ON deploy_listener_certificate_binding (tenant_id, certificate_id, status)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS deploy_tls_runtime_snapshot (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    node_target_id        BIGINT       NOT NULL,
    listener_key          VARCHAR(128) NOT NULL,
    generation            BIGINT       NOT NULL,
    snapshot_sha256       VARCHAR(64)  NOT NULL,
    assignment_count      INTEGER      NOT NULL,
    status                VARCHAR(16)  NOT NULL DEFAULT 'PENDING',
    published_at          TIMESTAMPTZ,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version               BIGINT       NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_tls_runtime_snapshot_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_tls_runtime_snapshot_generation UNIQUE (node_target_id, listener_key, generation),
    CONSTRAINT fk_deploy_tls_runtime_snapshot_target FOREIGN KEY (node_target_id) REFERENCES deploy_web_node_target(id),
    CONSTRAINT chk_deploy_tls_runtime_snapshot_generation CHECK (generation BETWEEN 1 AND 9007199254740991),
    CONSTRAINT chk_deploy_tls_runtime_snapshot_hash CHECK (snapshot_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_deploy_tls_runtime_snapshot_count CHECK (assignment_count BETWEEN 0 AND 10000),
    CONSTRAINT chk_deploy_tls_runtime_snapshot_status CHECK (status IN ('PENDING', 'PUBLISHED', 'ACTIVE', 'FAILED', 'SUPERSEDED'))
);

CREATE TABLE IF NOT EXISTS deploy_tls_runtime_assignment (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    snapshot_id           BIGINT       NOT NULL,
    listener_certificate_binding_id BIGINT NOT NULL,
    certificate_version_id BIGINT      NOT NULL,
    server_name           VARCHAR(253) NOT NULL,
    key_algorithm         VARCHAR(16)  NOT NULL,
    expected_fingerprint_sha256 VARCHAR(64) NOT NULL,
    material_ref          VARCHAR(128) NOT NULL,
    position              INTEGER      NOT NULL,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_tls_runtime_assignment_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_tls_runtime_assignment_sni UNIQUE (snapshot_id, server_name, key_algorithm),
    CONSTRAINT uk_deploy_tls_runtime_assignment_position UNIQUE (snapshot_id, position),
    CONSTRAINT fk_deploy_tls_runtime_assignment_snapshot FOREIGN KEY (snapshot_id) REFERENCES deploy_tls_runtime_snapshot(id),
    CONSTRAINT fk_deploy_tls_runtime_assignment_binding FOREIGN KEY (listener_certificate_binding_id) REFERENCES deploy_listener_certificate_binding(id),
    CONSTRAINT fk_deploy_tls_runtime_assignment_version FOREIGN KEY (certificate_version_id) REFERENCES deploy_certificate_version(id),
    CONSTRAINT chk_deploy_tls_runtime_assignment_algorithm CHECK (key_algorithm IN ('RSA', 'ECDSA')),
    CONSTRAINT chk_deploy_tls_runtime_assignment_fingerprint CHECK (expected_fingerprint_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT chk_deploy_tls_runtime_assignment_material CHECK (material_ref ~ '^file:[0-9a-zA-Z._-]{1,120}$'),
    CONSTRAINT chk_deploy_tls_runtime_assignment_position CHECK (position BETWEEN 0 AND 9999)
);

CREATE TABLE IF NOT EXISTS deploy_tls_target_observation (
    id                    BIGINT       NOT NULL,
    uuid                  VARCHAR(36)  NOT NULL,
    tenant_id             BIGINT       NOT NULL,
    snapshot_id           BIGINT       NOT NULL,
    node_target_id        BIGINT       NOT NULL,
    remote_observation_uuid VARCHAR(128) NOT NULL,
    generation            BIGINT       NOT NULL,
    state                 VARCHAR(24)  NOT NULL,
    served_fingerprint_sha256 VARCHAR(64),
    probe_vantage         VARCHAR(128),
    reason_code           VARCHAR(64),
    observed_at           TIMESTAMPTZ  NOT NULL,
    ingested_at           TIMESTAMPTZ  NOT NULL,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_tls_target_observation_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_tls_target_observation_remote UNIQUE (remote_observation_uuid),
    CONSTRAINT uk_deploy_tls_target_observation_state UNIQUE (snapshot_id, state, probe_vantage),
    CONSTRAINT fk_deploy_tls_target_observation_snapshot FOREIGN KEY (snapshot_id) REFERENCES deploy_tls_runtime_snapshot(id),
    CONSTRAINT fk_deploy_tls_target_observation_target FOREIGN KEY (node_target_id) REFERENCES deploy_web_node_target(id),
    CONSTRAINT chk_deploy_tls_target_observation_generation CHECK (generation BETWEEN 1 AND 9007199254740991),
    CONSTRAINT chk_deploy_tls_target_observation_state CHECK (state IN ('RECEIVED', 'MATERIAL_READY', 'LOADED', 'SERVED', 'PUBLIC_VERIFIED', 'FAILED')),
    CONSTRAINT chk_deploy_tls_target_observation_fingerprint CHECK (
        served_fingerprint_sha256 IS NULL OR served_fingerprint_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT chk_deploy_tls_target_observation_failure CHECK (
        (state = 'FAILED' AND reason_code IS NOT NULL)
        OR (state <> 'FAILED' AND reason_code IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_deploy_tls_target_observation_rollout
    ON deploy_tls_target_observation (tenant_id, snapshot_id, state, node_target_id, id DESC);

-- Post-create certificate version linkage (baseline lines 350-354): the
-- current/desired version columns point into deploy_certificate_version.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'deploy_certificate' AND column_name = 'desired_version_id') THEN
        ALTER TABLE deploy_certificate ADD COLUMN desired_version_id BIGINT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'deploy_certificate' AND column_name = 'current_version_id') THEN
        ALTER TABLE deploy_certificate ADD COLUMN current_version_id BIGINT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'fk_deploy_certificate_desired_version') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT fk_deploy_certificate_desired_version
            FOREIGN KEY (desired_version_id) REFERENCES deploy_certificate_version(id);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'fk_deploy_certificate_current_version') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT fk_deploy_certificate_current_version
            FOREIGN KEY (current_version_id) REFERENCES deploy_certificate_version(id);
    END IF;
END
$$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate_version'::regclass
                     AND conname = 'fk_deploy_certificate_version_order') THEN
        ALTER TABLE deploy_certificate_version
            ADD CONSTRAINT fk_deploy_certificate_version_order
            FOREIGN KEY (source_order_id) REFERENCES deploy_certificate_order(id);
    END IF;
END
$$;
