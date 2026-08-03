-- sdkwork:migration
-- id: 0005_deploy_node_cluster
-- engine: postgres
-- module: deploy
-- purpose: Create the node cluster grouping table referenced by
--   deploy_server.cluster_id. Added to the working baseline after the
--   2026-07-31 consolidated snapshot; kept as a forward migration so
--   existing databases converge through migrate instead of baseline replay.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 120s
-- contract_version: 1.0.0

CREATE TABLE IF NOT EXISTS deploy_node_cluster (
    id          BIGINT       NOT NULL,
    uuid        VARCHAR(64)  NOT NULL,
    tenant_id   BIGINT       NOT NULL DEFAULT 0,
    name        VARCHAR(200) NOT NULL,
    description VARCHAR(500) NULL,
    region      VARCHAR(64)  NULL,
    status      INTEGER      NOT NULL DEFAULT 0,
    metadata    JSONB        NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ  NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL,
    version     BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_node_cluster_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_node_cluster_name UNIQUE (tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_deploy_node_cluster_tenant_status
    ON deploy_node_cluster (tenant_id, status, updated_at DESC);
