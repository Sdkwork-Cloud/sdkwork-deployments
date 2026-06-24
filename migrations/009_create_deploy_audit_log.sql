-- Migration: 009_create_deploy_audit_log
-- Description: 创建操作审计日志表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_audit_log (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    operator_id     BIGINT       NOT NULL,
    operator_type   VARCHAR(32)  NOT NULL DEFAULT 'USER',
    action          VARCHAR(100) NOT NULL,
    target_type     VARCHAR(100) NOT NULL,
    target_id       BIGINT,
    target_uuid     VARCHAR(64),
    request_id      VARCHAR(128),
    ip_address      VARCHAR(45),
    user_agent      VARCHAR(500),
    changes         JSONB,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

COMMENT ON TABLE deploy_audit_log IS '操作审计日志表';
COMMENT ON COLUMN deploy_audit_log.operator_id IS '操作人ID';
COMMENT ON COLUMN deploy_audit_log.operator_type IS '操作人类型：USER, SYSTEM, ADMIN, JOB, SERVICE';
COMMENT ON COLUMN deploy_audit_log.action IS '操作类型';
COMMENT ON COLUMN deploy_audit_log.target_type IS '目标对象类型';
COMMENT ON COLUMN deploy_audit_log.target_id IS '目标对象ID';
COMMENT ON COLUMN deploy_audit_log.changes IS '变更内容JSON：{"field": {"old": x, "new": y}}';

CREATE INDEX idx_deploy_audit_log_target
    ON deploy_audit_log (target_type, target_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_operator
    ON deploy_audit_log (operator_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_tenant_action
    ON deploy_audit_log (tenant_id, action, created_at DESC);
