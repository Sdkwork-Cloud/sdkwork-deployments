-- Consolidated legacy baseline imported by bootstrap-database-module.mjs
-- Review and replace with contract-first migrations.

-- source: migrations/001_create_deploy_site.sql
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

-- source: migrations/002_create_deploy_domain.sql
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

-- source: migrations/003_create_deploy_nginx_config.sql
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

-- source: migrations/004_create_deploy_certificate.sql
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

-- source: migrations/005_create_deploy_deployment.sql
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

-- source: migrations/006_create_deploy_env_variable.sql
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

-- source: migrations/007_create_deploy_health_check.sql
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

-- source: migrations/008_create_deploy_health_result.sql
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

-- source: migrations/009_create_deploy_audit_log.sql
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

-- source: migrations/010_create_deploy_server.sql
-- Migration: 010_create_deploy_server
-- Description: 创建后端部署服务器表
-- Author: SDKWork Deploy Server
-- Date: 2026-06-23

CREATE TABLE deploy_server (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    name            VARCHAR(200) NOT NULL,
    host            VARCHAR(255) NOT NULL,
    ssh_port        INTEGER      NOT NULL DEFAULT 22,
    status          INTEGER      NOT NULL DEFAULT 0,
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_server_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_server_host UNIQUE (tenant_id, host)
);

COMMENT ON TABLE deploy_server IS '部署后端服务器表';
COMMENT ON COLUMN deploy_server.status IS '状态：0=未连接，1=在线，2=离线，3=维护中';

CREATE INDEX idx_deploy_server_tenant_status
    ON deploy_server (tenant_id, status, updated_at DESC);

