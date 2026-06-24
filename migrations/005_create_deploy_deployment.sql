-- Migration: 005_create_deploy_deployment
-- Description: 创建部署记录表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_deployment (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    user_id         BIGINT,
    site_id         BIGINT       NOT NULL,
    deploy_type     INTEGER      NOT NULL DEFAULT 1,
    version_tag     VARCHAR(100),
    commit_hash     VARCHAR(64),
    source_ref      VARCHAR(500),
    build_log       TEXT,
    deploy_log      TEXT,
    artifact_path   VARCHAR(500),
    artifact_size   BIGINT,
    artifact_hash   VARCHAR(64),
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    status          INTEGER      NOT NULL DEFAULT 0,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    duration_ms     BIGINT,
    rollback_from   BIGINT,
    idempotency_key VARCHAR(200),
    request_id      VARCHAR(128),
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_deployment_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_deployment_idempotency UNIQUE (tenant_id, idempotency_key),
    CONSTRAINT fk_deploy_deployment_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

COMMENT ON TABLE deploy_deployment IS '部署记录表';
COMMENT ON COLUMN deploy_deployment.deploy_type IS '部署类型：1=上传，2=Git，3=CI/CD，4=API';
COMMENT ON COLUMN deploy_deployment.status IS '状态：0=待处理，1=构建中，2=部署中，3=成功，4=失败，5=已回滚';
COMMENT ON COLUMN deploy_deployment.duration_ms IS '部署耗时（毫秒）';
COMMENT ON COLUMN deploy_deployment.rollback_from IS '回滚来源部署ID';
COMMENT ON COLUMN deploy_deployment.idempotency_key IS '幂等键，租户内唯一';

CREATE INDEX idx_deploy_deployment_site_created
    ON deploy_deployment (site_id, created_at DESC);

CREATE INDEX idx_deploy_deployment_tenant_status
    ON deploy_deployment (tenant_id, status, created_at DESC);

CREATE INDEX idx_deploy_deployment_status
    ON deploy_deployment (status)
    WHERE status IN (0, 1, 2);
