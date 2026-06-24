-- Migration: 007_create_deploy_health_check
-- Description: 创建健康检查配置表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_health_check (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    check_type      INTEGER      NOT NULL DEFAULT 1,
    check_url       VARCHAR(2000),
    check_interval  INTEGER      NOT NULL DEFAULT 60,
    timeout_ms      INTEGER      NOT NULL DEFAULT 5000,
    retry_count     INTEGER      NOT NULL DEFAULT 3,
    expected_status INTEGER,
    expected_body   VARCHAR(500),
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_health_check_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_health_check_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

COMMENT ON TABLE deploy_health_check IS '健康检查配置表';
COMMENT ON COLUMN deploy_health_check.check_type IS '检查类型：1=HTTP，2=TCP，3=Ping';
COMMENT ON COLUMN deploy_health_check.check_interval IS '检查间隔（秒）';
COMMENT ON COLUMN deploy_health_check.timeout_ms IS '超时时间（毫秒）';
COMMENT ON COLUMN deploy_health_check.retry_count IS '重试次数';

CREATE INDEX idx_deploy_health_check_site
    ON deploy_health_check (site_id);
