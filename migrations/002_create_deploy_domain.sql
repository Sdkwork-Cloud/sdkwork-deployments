-- Migration: 002_create_deploy_domain
-- Description: 创建域名绑定表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_domain (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    hostname        VARCHAR(255) NOT NULL,
    is_primary      BOOLEAN      NOT NULL DEFAULT false,
    is_verified     BOOLEAN      NOT NULL DEFAULT false,
    verify_token    VARCHAR(128),
    ssl_enabled     BOOLEAN      NOT NULL DEFAULT false,
    ssl_provider    VARCHAR(32),
    redirect_target VARCHAR(2000),
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMPTZ,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_domain_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_domain_hostname UNIQUE (hostname),
    CONSTRAINT fk_deploy_domain_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

COMMENT ON TABLE deploy_domain IS '域名绑定表';
COMMENT ON COLUMN deploy_domain.hostname IS '域名，全局唯一';
COMMENT ON COLUMN deploy_domain.is_primary IS '是否主域名';
COMMENT ON COLUMN deploy_domain.is_verified IS '是否已验证所有权';
COMMENT ON COLUMN deploy_domain.ssl_enabled IS '是否启用SSL';
COMMENT ON COLUMN deploy_domain.ssl_provider IS '证书提供者：letsencrypt, custom, none';
COMMENT ON COLUMN deploy_domain.status IS '状态：0=待处理，1=活跃，2=错误';

CREATE INDEX idx_deploy_domain_site
    ON deploy_domain (site_id);

CREATE INDEX idx_deploy_domain_tenant_status
    ON deploy_domain (tenant_id, status);
