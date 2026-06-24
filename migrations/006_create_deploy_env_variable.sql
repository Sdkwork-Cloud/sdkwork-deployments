-- Migration: 006_create_deploy_env_variable
-- Description: 创建环境变量表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_env_variable (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    key             VARCHAR(200) NOT NULL,
    value_encrypted TEXT         NOT NULL,
    is_secret       BOOLEAN      NOT NULL DEFAULT true,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_env_variable_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_env_variable_key UNIQUE (site_id, environment, key)
);

COMMENT ON TABLE deploy_env_variable IS '环境变量表';
COMMENT ON COLUMN deploy_env_variable.key IS '变量名';
COMMENT ON COLUMN deploy_env_variable.value_encrypted IS '加密存储的变量值';
COMMENT ON COLUMN deploy_env_variable.is_secret IS '是否为密钥类型';
COMMENT ON COLUMN deploy_env_variable.environment IS '所属环境';

CREATE INDEX idx_deploy_env_variable_site_env
    ON deploy_env_variable (site_id, environment);
