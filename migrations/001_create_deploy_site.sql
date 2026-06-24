-- Migration: 001_create_deploy_site
-- Description: 创建站点管理主表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_site (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    data_scope      INTEGER      NOT NULL DEFAULT 1,
    user_id         BIGINT,
    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL,
    description     VARCHAR(500),
    site_type       INTEGER      NOT NULL DEFAULT 1,
    status          INTEGER      NOT NULL DEFAULT 0,
    runtime_config  JSONB        NOT NULL DEFAULT '{}',
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    deleted_at      TIMESTAMPTZ,
    deleted_by      BIGINT,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_site_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_site_slug UNIQUE (tenant_id, slug),
    CONSTRAINT chk_deploy_site_type CHECK (site_type BETWEEN 1 AND 6),
    CONSTRAINT chk_deploy_site_status CHECK (status BETWEEN 0 AND 3)
);

COMMENT ON TABLE deploy_site IS '站点主表';
COMMENT ON COLUMN deploy_site.id IS '雪花ID主键';
COMMENT ON COLUMN deploy_site.uuid IS '外部稳定标识';
COMMENT ON COLUMN deploy_site.tenant_id IS '租户ID';
COMMENT ON COLUMN deploy_site.organization_id IS '组织ID，0表示租户级';
COMMENT ON COLUMN deploy_site.data_scope IS '数据可见范围：1=私有，2=组织，3=租户';
COMMENT ON COLUMN deploy_site.user_id IS '创建者用户ID';
COMMENT ON COLUMN deploy_site.name IS '站点名称';
COMMENT ON COLUMN deploy_site.slug IS 'URL友好标识，租户内唯一';
COMMENT ON COLUMN deploy_site.site_type IS '站点类型：1=静态，2=SPA，3=Node，4=PHP，5=Python，6=自定义';
COMMENT ON COLUMN deploy_site.status IS '状态：0=草稿，1=活跃，2=暂停，3=归档';
COMMENT ON COLUMN deploy_site.runtime_config IS '运行时配置JSON';
COMMENT ON COLUMN deploy_site.version IS '乐观锁版本号';

CREATE INDEX idx_deploy_site_tenant_status_updated
    ON deploy_site (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX idx_deploy_site_user_updated
    ON deploy_site (tenant_id, user_id, updated_at DESC);

CREATE INDEX idx_deploy_site_slug
    ON deploy_site (tenant_id, slug);
