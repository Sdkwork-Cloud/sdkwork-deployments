-- sdkwork:migration
-- id: 0008_deploy_usage_metering
-- engine: postgres
-- module: deploy
-- purpose: Create the metering and entitlement read-model tables designed in
--   TECH-cloud-site-publishing-control-plane.md section 4.6: append-only
--   usage facts, the Commerce-backed entitlement projection read model, and
--   reconcilable daily usage aggregates.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- Append-only usage fact. The deduplication identity prevents double billing;
-- Deploy emits facts, Commerce remains the pricing/billing authority.
CREATE TABLE IF NOT EXISTS deploy_usage_event (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NULL,
    binding_id      BIGINT       NULL,
    period_start    TIMESTAMPTZ  NOT NULL,
    dimension       VARCHAR(64)  NOT NULL,
    quantity        BIGINT       NOT NULL DEFAULT 0,
    unit            VARCHAR(32)  NOT NULL,
    source_target_uuid VARCHAR(128),
    source_window_id VARCHAR(128),
    deduplication_key VARCHAR(200) NOT NULL,
    attribution_json JSONB        NOT NULL DEFAULT '{}',
    observed_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    ingested_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_deploy_usage_event PRIMARY KEY (id),
    CONSTRAINT fk_deploy_usage_event_site
        FOREIGN KEY (site_id) REFERENCES deploy_site(id),
    CONSTRAINT ck_deploy_usage_event_quantity
        CHECK (quantity >= 0),
    CONSTRAINT ck_deploy_usage_event_dimension
        CHECK (length(dimension) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_usage_event_uuid
    ON deploy_usage_event (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_usage_event_dedup
    ON deploy_usage_event (tenant_id, deduplication_key);

CREATE INDEX IF NOT EXISTS idx_deploy_usage_event_tenant_period
    ON deploy_usage_event (tenant_id, period_start DESC);

CREATE INDEX IF NOT EXISTS idx_deploy_usage_event_target
    ON deploy_usage_event (source_target_uuid)
    WHERE source_target_uuid IS NOT NULL;

-- Commerce-backed entitlement projection read model. Commerce is the write
-- authority; stale/absent projections fail closed for new capacity.
CREATE TABLE IF NOT EXISTS deploy_tenant_entitlement_projection (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    source_system   VARCHAR(64)  NOT NULL,
    source_subscription_uuid VARCHAR(128) NOT NULL,
    source_revision VARCHAR(64),
    plan_key        VARCHAR(64),
    entitlements_json JSONB      NOT NULL DEFAULT '{}',
    effective_at    TIMESTAMPTZ  NOT NULL,
    expires_at      TIMESTAMPTZ,
    projection_status VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',
    created_by      BIGINT,
    updated_by      BIGINT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_tenant_entitlement_projection PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_entitlement_projection_uuid
    ON deploy_tenant_entitlement_projection (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_entitlement_projection_scope
    ON deploy_tenant_entitlement_projection (tenant_id, source_system, source_subscription_uuid);

CREATE INDEX IF NOT EXISTS idx_deploy_entitlement_projection_tenant_status
    ON deploy_tenant_entitlement_projection (tenant_id, projection_status, expires_at);

-- Reconcilable daily aggregate; rebuildable from retained usage facts.
CREATE TABLE IF NOT EXISTS deploy_site_usage_daily (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(36)  NOT NULL,
    tenant_id       BIGINT       NOT NULL,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    usage_date      DATE         NOT NULL,
    dimension       VARCHAR(64)  NOT NULL,
    quantity        BIGINT       NOT NULL DEFAULT 0,
    unit            VARCHAR(32)  NOT NULL,
    source_revision VARCHAR(64),
    finalization_status VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    finalized_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version         BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT pk_deploy_site_usage_daily PRIMARY KEY (id),
    CONSTRAINT fk_deploy_site_usage_daily_site
        FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_usage_daily_uuid
    ON deploy_site_usage_daily (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_site_usage_daily_scope
    ON deploy_site_usage_daily (tenant_id, site_id, usage_date, dimension, unit);

CREATE INDEX IF NOT EXISTS idx_deploy_site_usage_daily_period
    ON deploy_site_usage_daily (tenant_id, usage_date DESC);
