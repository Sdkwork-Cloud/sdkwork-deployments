-- Migration: 008_create_deploy_health_result
-- Description: 创建健康检查结果表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_health_result (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    health_check_id BIGINT       NOT NULL,
    site_id         BIGINT       NOT NULL,
    is_healthy      BOOLEAN      NOT NULL,
    response_ms     INTEGER,
    status_code     INTEGER,
    error_message   VARCHAR(1000),
    checked_at      TIMESTAMPTZ  NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

COMMENT ON TABLE deploy_health_result IS '健康检查结果表';
COMMENT ON COLUMN deploy_health_result.is_healthy IS '是否健康';
COMMENT ON COLUMN deploy_health_result.response_ms IS '响应时间（毫秒）';
COMMENT ON COLUMN deploy_health_result.status_code IS 'HTTP状态码';
COMMENT ON COLUMN deploy_health_result.checked_at IS '检查时间';

CREATE INDEX idx_deploy_health_result_check_time
    ON deploy_health_result (health_check_id, checked_at DESC);

CREATE INDEX idx_deploy_health_result_site_time
    ON deploy_health_result (site_id, checked_at DESC);
