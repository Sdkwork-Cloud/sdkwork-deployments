-- Migration: 004_create_deploy_certificate
-- Description: 创建 SSL 证书表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-14

CREATE TABLE deploy_certificate (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    domain_id       BIGINT,
    site_id         BIGINT,
    cert_name       VARCHAR(200) NOT NULL,
    cert_type       INTEGER      NOT NULL DEFAULT 1,
    issuer          VARCHAR(200),
    subject         VARCHAR(500),
    san_list        TEXT,
    fingerprint     VARCHAR(128),
    cert_path       VARCHAR(500),
    key_path        VARCHAR(500),
    chain_path      VARCHAR(500),
    not_before      TIMESTAMPTZ,
    not_after       TIMESTAMPTZ,
    auto_renew      BOOLEAN      NOT NULL DEFAULT true,
    renewal_status  INTEGER      NOT NULL DEFAULT 0,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_uuid UNIQUE (uuid)
);

COMMENT ON TABLE deploy_certificate IS 'SSL证书表';
COMMENT ON COLUMN deploy_certificate.cert_type IS '证书类型：1=Let\'s Encrypt，2=自定义，3=自签名';
COMMENT ON COLUMN deploy_certificate.san_list IS 'Subject Alternative Names，逗号分隔';
COMMENT ON COLUMN deploy_certificate.auto_renew IS '是否自动续期';
COMMENT ON COLUMN deploy_certificate.renewal_status IS '续期状态：0=无，1=已计划，2=处理中，3=失败';
COMMENT ON COLUMN deploy_certificate.status IS '状态：0=待处理，1=活跃，2=过期，3=已撤销';

CREATE INDEX idx_deploy_certificate_domain
    ON deploy_certificate (domain_id);

CREATE INDEX idx_deploy_certificate_expiry
    ON deploy_certificate (not_after)
    WHERE status = 1;

CREATE INDEX idx_deploy_certificate_renewal
    ON deploy_certificate (renewal_status, not_after)
    WHERE auto_renew = true AND status = 1;
