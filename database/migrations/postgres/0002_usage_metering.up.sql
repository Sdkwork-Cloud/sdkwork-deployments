-- sdkwork:migration
-- id: 0002_usage_metering
-- engine: postgres
-- module: sdkwork-deployments
-- purpose: Traffic usage metering (per-domain / per-server-IP billing).
--   `deploy_site_usage_daily` gains binding granularity (per-domain daily
--   rollups) and a new `deploy_tenant_usage_daily` table holds tenant-level
--   daily rollups for SaaS billing (including unmanaged traffic attributed
--   to the platform tenant).
-- reversible: false
-- rollback: forward-fix (dropping the new table/column would lose billing
--   aggregates; rebuild from `deploy_usage_event` facts if ever needed)
-- transactional: true
-- lock: lightweight

ALTER TABLE deploy_site_usage_daily
    ADD COLUMN IF NOT EXISTS binding_id BIGINT NULL
        REFERENCES deploy_site_binding (id);

DROP INDEX IF EXISTS uk_deploy_site_usage_daily_scope;
CREATE UNIQUE INDEX uk_deploy_site_usage_daily_scope
    ON deploy_site_usage_daily (tenant_id, site_id, binding_id, usage_date, dimension, unit);

CREATE TABLE IF NOT EXISTS deploy_tenant_usage_daily (
    id                 BIGINT PRIMARY KEY NOT NULL,
    uuid               VARCHAR(36)  NOT NULL,
    tenant_id          BIGINT       NOT NULL,
    organization_id    BIGINT       NOT NULL DEFAULT 0,
    usage_date         DATE         NOT NULL,
    dimension          VARCHAR(64)  NOT NULL,
    quantity           BIGINT       NOT NULL DEFAULT 0,
    unit               VARCHAR(32)  NOT NULL,
    source_revision    VARCHAR(64),
    finalization_status VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    finalized_at       TIMESTAMPTZ,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    version            BIGINT       NOT NULL DEFAULT 1,
    CONSTRAINT ck_deploy_tenant_usage_daily_quantity CHECK (quantity >= 0),
    CONSTRAINT ck_deploy_tenant_usage_daily_status
        CHECK (finalization_status IN ('PENDING', 'FINALIZED'))
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_tenant_usage_daily_uuid
    ON deploy_tenant_usage_daily (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_tenant_usage_daily_scope
    ON deploy_tenant_usage_daily (tenant_id, dimension, usage_date, unit);

CREATE INDEX IF NOT EXISTS idx_deploy_tenant_usage_daily_period
    ON deploy_tenant_usage_daily (tenant_id, usage_date DESC);
