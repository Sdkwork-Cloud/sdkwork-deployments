-- Migration: 003_create_deploy_nginx_config
-- Description: 创建 Nginx 配置版本表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_nginx_config (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    config_type     INTEGER      NOT NULL DEFAULT 1,
    config_name     VARCHAR(200) NOT NULL,
    config_content  TEXT         NOT NULL,
    config_hash     VARCHAR(64)  NOT NULL,
    is_active       BOOLEAN      NOT NULL DEFAULT false,
    version_no      INTEGER      NOT NULL DEFAULT 1,
    deployed_at     TIMESTAMPTZ,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_nginx_config_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_nginx_config_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

COMMENT ON TABLE deploy_nginx_config IS 'Nginx配置版本表';
COMMENT ON COLUMN deploy_nginx_config.config_type IS '配置类型：1=站点，2=上游，3=SSL，4=自定义';
COMMENT ON COLUMN deploy_nginx_config.config_content IS 'Nginx配置内容';
COMMENT ON COLUMN deploy_nginx_config.config_hash IS '配置内容SHA-256哈希';
COMMENT ON COLUMN deploy_nginx_config.is_active IS '是否为当前活跃配置';
COMMENT ON COLUMN deploy_nginx_config.version_no IS '配置版本号';
COMMENT ON COLUMN deploy_nginx_config.deployed_at IS '部署时间';
COMMENT ON COLUMN deploy_nginx_config.status IS '状态：0=草稿，1=活跃，2=归档';

CREATE INDEX idx_deploy_nginx_config_site_active
    ON deploy_nginx_config (site_id, is_active);

CREATE INDEX idx_deploy_nginx_config_type_status
    ON deploy_nginx_config (config_type, status);
