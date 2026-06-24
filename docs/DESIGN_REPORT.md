# SDKWork Deploy Server - 完整设计报告

- 版本：1.0
- 日期：2026-06-14
- 定位：基于 Rust 的专业 WebServer 部署管理服务，完美兼容 Nginx 配置
- 遵循：SDKWORK 规范体系

## 目录

1. [项目概述](#1-项目概述)
2. [架构设计](#2-架构设计)
3. [数据库设计](#3-数据库设计)
4. [API 设计](#4-api-设计)
5. [SDK 设计](#5-sdk-设计)
6. [Nginx 兼容设计](#6-nginx-兼容设计)
7. [Rust 开源组件集成](#7-rust-开源组件集成)
8. [UI/UX 功能规划](#8-uiux-功能规划)
9. [安全设计](#9-安全设计)
10. [部署方案](#10-部署方案)

---

## 1. 项目概述

### 1.1 产品定位

SDKWork Deploy Server 是一个专业的 WebServer 部署管理平台后端服务，提供：

- **站点管理**：Web 站点的创建、配置、部署、监控全生命周期管理
- **Nginx 管理**：完美兼容 Nginx 配置，支持配置生成、热加载、证书管理
- **域名管理**：域名绑定、SSL/TLS 证书自动申请与续期
- **部署流水线**：应用打包、版本管理、灰度发布、回滚
- **监控告警**：站点可用性监控、性能指标采集、异常告警
- **资源管理**：服务器资源、存储、网络拓扑管理

### 1.2 核心设计原则

| 原则 | 说明 |
| --- | --- |
| 合约优先 | OpenAPI 作为 API 唯一事实来源 |
| 分层清晰 | Handler → Service → Repository 严格分层 |
| 多租户隔离 | 基于 tenant_id 的数据隔离 |
| Nginx 原生兼容 | 生成标准 Nginx 配置，支持所有主流指令 |
| Rust 生态优先 | 优先使用 Rust 开源组件 |
| 可观测性 | 结构化日志、指标采集、链路追踪 |

### 1.3 技术栈选型

| 层次 | 技术选型 | 说明 |
| --- | --- | --- |
| HTTP 框架 | Axum 0.7+ | Tokio 生态，类型安全，性能优秀 |
| 异步运行时 | Tokio 1.x | Rust 标准异步运行时 |
| 数据库 | PostgreSQL 15+ / SQLite | 服务端 PG，桌面端 SQLite |
| ORM/查询 | SQLx 0.7+ | 编译期校验，异步原生 |
| 缓存 | Redis 7+ / 本地缓存 | 服务端 Redis，桌面端内存缓存 |
| 序列化 | Serde + serde_json | JSON 标准序列化 |
| 配置 | TOML + 环境变量 | 层次化配置 |
| 日志 | tracing + tracing-subscriber | 结构化日志 |
| 错误处理 | thiserror + anyhow | 分层错误处理 |
| HTTP 客户端 | reqwest | 外部 API 调用 |
| SSH 远程 | tokio-serde + ssh2 | 远程服务器管理 |
| 定时任务 | tokio-cron-scheduler | 证书续期、健康检查 |
| UUID/ID | uuid + 雪花ID | 全局唯一标识 |

---

## 2. 架构设计

### 2.1 系统架构总览

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Nginx / 反向代理层                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  静态资源服务  │  │  API 网关路由  │  │  WebSocket 代理      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SDKWork Deploy API Server                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Middleware Chain                         │ │
│  │  Request ID → CORS → Rate Limit → Auth → Tenant → Log     │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Route Layer (Router Crates)              │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────┐ │ │
│  │  │ site-router│ │nginx-router│ │deploy-router│ │ ...     │ │ │
│  │  └────────────┘ └────────────┘ └────────────┘ └─────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Service Layer                            │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────┐ │ │
│  │  │SiteService │ │NginxService│ │DeployService│ │ ...     │ │ │
│  │  └────────────┘ └────────────┘ └────────────┘ └─────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Repository Layer (SQLx)                  │ │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────┐ │ │
│  │  │SiteRepo    │ │NginxRepo   │ │DeployRepo  │ │ ...     │ │ │
│  │  └────────────┘ └────────────┘ └────────────┘ └─────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              │                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Infrastructure                           │ │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────┐ │ │
│  │  │PostgreSQL│ │ Redis  │ │ Nginx  │ │SSH远程 │ │ 文件存储  │ │ │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └──────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Crate 架构设计

遵循 SDKWORK RUST_CODE_SPEC.md，crate 按职责命名和拆分：

```
crates/
├── sdkwork-deploy-api-server/              # HTTP API 服务器进程
│   ├── src/
│   │   ├── main.rs                         # 进程启动
│   │   ├── lib.rs                          # 模块组装
│   │   ├── bootstrap/                      # 依赖注入、路由挂载
│   │   │   ├── config.rs
│   │   │   ├── state.rs
│   │   │   ├── database.rs
│   │   │   ├── repositories.rs
│   │   │   ├── services.rs
│   │   │   ├── adapters.rs
│   │   │   └── routers.rs
│   │   ├── server/                         # HTTP 监听、关闭
│   │   │   ├── listen.rs
│   │   │   ├── shutdown.rs
│   │   │   └── middleware.rs
│   │   ├── preflight/                      # 启动前检查
│   │   │   ├── config.rs
│   │   │   ├── database.rs
│   │   │   └── nginx.rs
│   │   └── health.rs
│   └── tests/
│
├── sdkwork-router-site-app-api/            # 站点管理路由
│   ├── src/
│   │   ├── lib.rs
│   │   ├── paths.rs                        # 路径常量
│   │   ├── routes.rs                       # 路由组合
│   │   ├── handlers.rs                     # HTTP 处理器
│   │   ├── manifest.rs                     # 路由清单
│   │   ├── error.rs
│   │   └── mapper/
│   │       ├── mod.rs
│   │       ├── request.rs
│   │       ├── response.rs
│   │       └── problem.rs
│   └── tests/
│
├── sdkwork-router-nginx-backend-api/       # Nginx 管理路由
│   └── ...
│
├── sdkwork-router-deploy-app-api/          # 部署管理路由
│   └── ...
│
├── sdkwork-router-domain-app-api/          # 域名管理路由
│   └── ...
│
├── sdkwork-router-cert-app-api/            # 证书管理路由
│   └── ...
│
├── sdkwork-router-monitor-app-api/         # 监控管理路由
│   └── ...
│
├── sdkwork-deploy-site-service/            # 站点业务逻辑
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── context.rs
│   │   ├── error.rs
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── models.rs
│   │   │   ├── value_objects.rs
│   │   │   ├── commands.rs
│   │   │   ├── results.rs
│   │   │   └── events.rs
│   │   ├── ports/
│   │   │   ├── mod.rs
│   │   │   ├── repository.rs
│   │   │   ├── provider.rs
│   │   │   └── events.rs
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── site_service.rs
│   │       ├── create_site.rs
│   │       ├── update_site.rs
│   │       └── delete_site.rs
│   └── tests/
│
├── sdkwork-deploy-nginx-service/           # Nginx 配置管理业务
│   └── ...
│
├── sdkwork-deploy-deploy-service/          # 部署流水线业务
│   └── ...
│
├── sdkwork-deploy-domain-service/          # 域名管理业务
│   └── ...
│
├── sdkwork-deploy-cert-service/            # 证书管理业务
│   └── ...
│
├── sdkwork-deploy-monitor-service/         # 监控告警业务
│   └── ...
│
├── sdkwork-deploy-site-repository-sqlx/    # 站点数据访问
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   ├── rows.rs
│   │   │   ├── columns.rs
│   │   │   └── indexes.rs
│   │   ├── mapper/
│   │   │   ├── mod.rs
│   │   │   └── row_mapper.rs
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── queries.rs
│   │       └── site_repository.rs
│   └── tests/
│
├── sdkwork-deploy-nginx-repository-sqlx/   # Nginx 配置数据访问
│   └── ...
│
├── sdkwork-deploy-deploy-repository-sqlx/  # 部署记录数据访问
│   └── ...
│
├── sdkwork-deploy-nginx-adapter/           # Nginx 命令适配器
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config_generator.rs             # Nginx 配置生成器
│   │   ├── config_parser.rs                # Nginx 配置解析器
│   │   ├── process_manager.rs              # Nginx 进程管理
│   │   └── reload_executor.rs              # 热加载执行器
│   └── tests/
│
├── sdkwork-deploy-ssh-adapter/             # SSH 远程管理适配器
│   └── ...
│
├── sdkwork-deploy-cert-adapter/            # 证书管理适配器(ACME/Let's Encrypt)
│   └── ...
│
└── sdkwork-deploy-worker/                  # 后台任务
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs
    │   ├── jobs/
    │   │   ├── mod.rs
    │   │   ├── cert_renewal.rs             # 证书自动续期
    │   │   ├── health_check.rs             # 健康检查
    │   │   ├── cleanup.rs                  # 过期数据清理
    │   │   └── metrics_collector.rs        # 指标采集
    │   ├── scheduler/
    │   │   ├── mod.rs
    │   │   └── cron.rs
    │   └── bootstrap/
    └── tests/
```

### 2.3 依赖关系图

```text
sdkwork-deploy-api-server
  ├── sdkwork-router-site-app-api
  ├── sdkwork-router-nginx-backend-api
  ├── sdkwork-router-deploy-app-api
  ├── sdkwork-router-domain-app-api
  ├── sdkwork-router-cert-app-api
  ├── sdkwork-router-monitor-app-api
  ├── sdkwork-deploy-site-service
  ├── sdkwork-deploy-nginx-service
  ├── sdkwork-deploy-deploy-service
  ├── sdkwork-deploy-domain-service
  ├── sdkwork-deploy-cert-service
  ├── sdkwork-deploy-monitor-service
  ├── sdkwork-deploy-site-repository-sqlx
  ├── sdkwork-deploy-nginx-repository-sqlx
  ├── sdkwork-deploy-deploy-repository-sqlx
  ├── sdkwork-deploy-nginx-adapter
  ├── sdkwork-deploy-ssh-adapter
  ├── sdkwork-deploy-cert-adapter
  └── sdkwork-appbase (依赖)

Route Crates → Service Crates (通过 trait)
Service Crates → Repository Crates (通过 trait 实现)
Service Crates → Adapter Crates (通过 trait)
Repository Crates → SQLx + PostgreSQL/SQLite
```

### 2.4 分层职责

| 层 | 职责 | 禁止 |
| --- | --- | --- |
| Router/Handler | HTTP 解码、上下文注入、调用 Service、映射响应 | 业务逻辑、SQL 查询 |
| Service | 业务规则、权限决策、事务编排、幂等、事件 | HTTP 类型、原始 Header 解析 |
| Repository | 持久化查询/命令、Schema 映射、乐观锁 | 业务策略、权限检查 |
| Adapter | 外部系统集成：Nginx、SSH、ACME、监控 | 业务逻辑 |

---

## 3. 数据库设计

### 3.1 数据库选型与策略

| 环境 | 数据库 | 说明 |
| --- | --- | --- |
| Server/Production | PostgreSQL 15+ | 主数据存储，支持 JSONB、全文索引 |
| Desktop/Local | SQLite | 轻量本地存储 |
| Cache | Redis 7+ | 缓存、会话、限流、实时数据 |

### 3.2 Schema 设计

遵循 SDKWORK DATABASE_SPEC.md 标准，所有表使用业务模块前缀 `deploy_`。

#### 3.2.1 租户与用户相关（复用 appbase iam_* 表）

本服务不重复创建 IAM 相关表，通过 sdkwork-appbase 依赖获取：
- `iam_tenant` - 租户
- `iam_organization` - 组织
- `iam_user` - 用户
- `iam_role` - 角色
- `iam_permission` - 权限

#### 3.2.2 站点管理表

```sql
-- deploy_site: 站点主表 (core_entity + tenant_entity)
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
    site_type       INTEGER      NOT NULL DEFAULT 1,     -- 1=static, 2=spa, 3=node, 4=php, 5=python, 6=custom
    status          INTEGER      NOT NULL DEFAULT 0,     -- 0=draft, 1=active, 2=paused, 3=archived
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

CREATE INDEX idx_deploy_site_tenant_status_updated
    ON deploy_site (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX idx_deploy_site_user_updated
    ON deploy_site (tenant_id, user_id, updated_at DESC);

CREATE INDEX idx_deploy_site_slug
    ON deploy_site (tenant_id, slug);
```

#### 3.2.3 域名绑定表

```sql
-- deploy_domain: 域名绑定表 (tenant_entity)
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
    ssl_provider    VARCHAR(32),          -- letsencrypt, custom, none
    redirect_target VARCHAR(2000),
    status          INTEGER      NOT NULL DEFAULT 0,  -- 0=pending, 1=active, 2=error
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

CREATE INDEX idx_deploy_domain_site
    ON deploy_domain (site_id);

CREATE INDEX idx_deploy_domain_tenant_status
    ON deploy_domain (tenant_id, status);
```

#### 3.2.4 Nginx 配置表

```sql
-- deploy_nginx_config: Nginx 配置版本表 (core_entity)
CREATE TABLE deploy_nginx_config (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    config_type     INTEGER      NOT NULL DEFAULT 1,  -- 1=site, 2=upstream, 3=ssl, 4=custom
    config_name     VARCHAR(200) NOT NULL,
    config_content  TEXT         NOT NULL,
    config_hash     VARCHAR(64)  NOT NULL,             -- SHA-256
    is_active       BOOLEAN      NOT NULL DEFAULT false,
    version_no      INTEGER      NOT NULL DEFAULT 1,
    deployed_at     TIMESTAMPTZ,
    status          INTEGER      NOT NULL DEFAULT 0,  -- 0=draft, 1=active, 2=archived
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_nginx_config_uuid UNIQUE (uuid),
    CONSTRAINT fk_deploy_nginx_config_site FOREIGN KEY (site_id) REFERENCES deploy_site(id)
);

CREATE INDEX idx_deploy_nginx_config_site_active
    ON deploy_nginx_config (site_id, is_active);

CREATE INDEX idx_deploy_nginx_config_type_status
    ON deploy_nginx_config (config_type, status);
```

#### 3.2.5 SSL 证书表

```sql
-- deploy_certificate: SSL 证书表 (core_entity)
CREATE TABLE deploy_certificate (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    domain_id       BIGINT,
    site_id         BIGINT,
    cert_name       VARCHAR(200) NOT NULL,
    cert_type       INTEGER      NOT NULL DEFAULT 1,  -- 1=letsencrypt, 2=custom, 3=self_signed
    issuer          VARCHAR(200),
    subject         VARCHAR(500),
    san_list        TEXT,                              -- Subject Alternative Names
    fingerprint     VARCHAR(128),
    cert_path       VARCHAR(500),
    key_path        VARCHAR(500),
    chain_path      VARCHAR(500),
    not_before      TIMESTAMPTZ,
    not_after       TIMESTAMPTZ,
    auto_renew      BOOLEAN      NOT NULL DEFAULT true,
    renewal_status  INTEGER      NOT NULL DEFAULT 0,  -- 0=none, 1=scheduled, 2=processing, 3=failed
    status          INTEGER      NOT NULL DEFAULT 0,  -- 0=pending, 1=active, 2=expired, 3=revoked
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_certificate_uuid UNIQUE (uuid)
);

CREATE INDEX idx_deploy_certificate_domain
    ON deploy_certificate (domain_id);

CREATE INDEX idx_deploy_certificate_expiry
    ON deploy_certificate (not_after)
    WHERE status = 1;

CREATE INDEX idx_deploy_certificate_renewal
    ON deploy_certificate (renewal_status, not_after)
    WHERE auto_renew = true AND status = 1;
```

#### 3.2.6 部署记录表

```sql
-- deploy_deployment: 部署记录表 (event_log)
CREATE TABLE deploy_deployment (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    organization_id BIGINT       NOT NULL DEFAULT 0,
    user_id         BIGINT,
    site_id         BIGINT       NOT NULL,
    deploy_type     INTEGER      NOT NULL DEFAULT 1,  -- 1=upload, 2=git, 3=ci_cd, 4=api
    version_tag     VARCHAR(100),
    commit_hash     VARCHAR(64),
    source_ref      VARCHAR(500),                      -- git URL, upload path, etc.
    build_log       TEXT,
    deploy_log      TEXT,
    artifact_path   VARCHAR(500),
    artifact_size   BIGINT,
    artifact_hash   VARCHAR(64),
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    status          INTEGER      NOT NULL DEFAULT 0,  -- 0=pending, 1=building, 2=deploying, 3=active, 4=failed, 5=rolled_back
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    duration_ms     BIGINT,
    rollback_from   BIGINT,                            -- 关联被回滚的部署 ID
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

CREATE INDEX idx_deploy_deployment_site_created
    ON deploy_deployment (site_id, created_at DESC);

CREATE INDEX idx_deploy_deployment_tenant_status
    ON deploy_deployment (tenant_id, status, created_at DESC);

CREATE INDEX idx_deploy_deployment_status
    ON deploy_deployment (status)
    WHERE status IN (0, 1, 2);
```

#### 3.2.7 部署环境变量表

```sql
-- deploy_env_variable: 环境变量表 (tenant_entity)
CREATE TABLE deploy_env_variable (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    environment     VARCHAR(32)  NOT NULL DEFAULT 'production',
    key             VARCHAR(200) NOT NULL,
    value_encrypted TEXT         NOT NULL,             -- 加密存储
    is_secret       BOOLEAN      NOT NULL DEFAULT true,
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_env_variable_uuid UNIQUE (uuid),
    CONSTRAINT uk_deploy_env_variable_key UNIQUE (site_id, environment, key)
);

CREATE INDEX idx_deploy_env_variable_site_env
    ON deploy_env_variable (site_id, environment);
```

#### 3.2.8 监控健康检查表

```sql
-- deploy_health_check: 健康检查配置表 (core_entity)
CREATE TABLE deploy_health_check (
    id              BIGINT       NOT NULL,
    uuid            VARCHAR(64)  NOT NULL,
    tenant_id       BIGINT       NOT NULL DEFAULT 0,
    site_id         BIGINT       NOT NULL,
    domain_id       BIGINT,
    check_type      INTEGER      NOT NULL DEFAULT 1,  -- 1=http, 2=tcp, 3=ping
    check_url       VARCHAR(2000),
    check_interval  INTEGER      NOT NULL DEFAULT 60,  -- seconds
    timeout_ms      INTEGER      NOT NULL DEFAULT 5000,
    retry_count     INTEGER      NOT NULL DEFAULT 3,
    expected_status INTEGER,
    expected_body   VARCHAR(500),
    status          INTEGER      NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL,
    updated_at      TIMESTAMPTZ  NOT NULL,
    version         BIGINT       NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT uk_deploy_health_check_uuid UNIQUE (uuid)
);

CREATE INDEX idx_deploy_health_check_site
    ON deploy_health_check (site_id);
```

#### 3.2.9 健康检查结果表

```sql
-- deploy_health_result: 健康检查结果表 (event_log)
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

-- 按时间分区，保留最近 30 天
CREATE INDEX idx_deploy_health_result_check_time
    ON deploy_health_result (health_check_id, checked_at DESC);

CREATE INDEX idx_deploy_health_result_site_time
    ON deploy_health_result (site_id, checked_at DESC);
```

#### 3.2.10 操作审计日志表

```sql
-- deploy_audit_log: 操作审计表 (audit_log)
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
    changes         JSONB,                              -- {"field": {"old": x, "new": y}}
    metadata        JSONB        NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (id)
);

CREATE INDEX idx_deploy_audit_log_target
    ON deploy_audit_log (target_type, target_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_operator
    ON deploy_audit_log (operator_id, created_at DESC);

CREATE INDEX idx_deploy_audit_log_tenant_action
    ON deploy_audit_log (tenant_id, action, created_at DESC);
```

### 3.3 数据库 ER 关系图

```text
┌──────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  deploy_site │────<│  deploy_domain   │     │ deploy_certificate│
│              │     │                  │────>│                  │
│  id (PK)     │     │  id (PK)         │     │  id (PK)         │
│  uuid        │     │  site_id (FK)    │     │  domain_id (FK)  │
│  tenant_id   │     │  hostname        │     │  cert_name       │
│  name        │     │  ssl_enabled     │     │  not_after       │
│  slug        │     │  is_primary      │     │  auto_renew      │
│  site_type   │     └──────────────────┘     └──────────────────┘
│  status      │
└──────┬───────┘
       │
       ├────<┌──────────────────┐     ┌──────────────────┐
       │     │deploy_nginx_config│    │ deploy_deployment│
       │     │                  │     │                  │
       │     │  id (PK)         │     │  id (PK)         │
       │     │  site_id (FK)    │     │  site_id (FK)    │
       │     │  config_content  │     │  deploy_type     │
       │     │  config_hash     │     │  status          │
       │     │  is_active       │     │  version_tag     │
       │     └──────────────────┘     └──────────────────┘
       │
       ├────<┌──────────────────┐     ┌──────────────────┐
       │     │deploy_env_variable│    │deploy_health_check│
       │     │                  │     │                  │
       │     │  id (PK)         │     │  id (PK)         │
       │     │  site_id (FK)    │     │  site_id (FK)    │
       │     │  key             │     │  check_type      │
       │     │  value_encrypted │     │  check_interval  │
       │     └──────────────────┘     └────────┬─────────┘
       │                                       │
       └────<┌──────────────────┐     ┌────────┴─────────┐
             │ deploy_audit_log │     │deploy_health_result│
             │                  │     │                  │
             │  id (PK)         │     │  id (PK)         │
             │  operator_id     │     │  health_check_id │
             │  action          │     │  is_healthy      │
             │  target_type     │     │  response_ms     │
             └──────────────────┘     └──────────────────┘
```

### 3.4 索引策略

| 表 | 索引 | 用途 |
| --- | --- | --- |
| deploy_site | (tenant_id, slug) UK | 唯一站点标识 |
| deploy_site | (tenant_id, status, updated_at) | 列表查询 |
| deploy_domain | (hostname) UK | 域名查找 |
| deploy_domain | (site_id) | 站点关联查询 |
| deploy_nginx_config | (site_id, is_active) | 活跃配置查找 |
| deploy_certificate | (not_after) WHERE active | 证书到期查询 |
| deploy_deployment | (site_id, created_at DESC) | 部署历史 |
| deploy_deployment | (tenant_id, idempotency_key) UK | 幂等去重 |
| deploy_audit_log | (target_type, target_id, created_at) | 审计追溯 |

---

## 4. API 设计

### 4.1 API 总览

遵循 SDKWORK API_SPEC.md，API 分为三个表面：

| Surface | 前缀 | 用途 | 认证 |
| --- | --- | --- | --- |
| App API | `/app/v3/api` | 应用客户端 | Dual Token |
| Backend API | `/backend/v3/api` | 管理后台 | Dual Token |
| Open API | `/deploy/v3/api` | 外部集成 | API Key |

### 4.2 App API 路由定义

#### 4.2.1 站点管理 (Site)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/sites` | `sites.list` | 获取站点列表 |
| POST | `/app/v3/api/sites` | `sites.create` | 创建站点 |
| GET | `/app/v3/api/sites/{siteId}` | `sites.retrieve` | 获取站点详情 |
| PATCH | `/app/v3/api/sites/{siteId}` | `sites.update` | 更新站点 |
| DELETE | `/app/v3/api/sites/{siteId}` | `sites.delete` | 删除站点 |
| POST | `/app/v3/api/sites/{siteId}/activate` | `sites.activate` | 激活站点 |
| POST | `/app/v3/api/sites/{siteId}/pause` | `sites.pause` | 暂停站点 |

#### 4.2.2 域名管理 (Domain)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/sites/{siteId}/domains` | `sites.domains.list` | 获取站点域名列表 |
| POST | `/app/v3/api/sites/{siteId}/domains` | `sites.domains.create` | 绑定域名 |
| GET | `/app/v3/api/sites/{siteId}/domains/{domainId}` | `sites.domains.retrieve` | 获取域名详情 |
| DELETE | `/app/v3/api/sites/{siteId}/domains/{domainId}` | `sites.domains.delete` | 解绑域名 |
| POST | `/app/v3/api/sites/{siteId}/domains/{domainId}/verify` | `sites.domains.verify` | 验证域名所有权 |
| POST | `/app/v3/api/sites/{siteId}/domains/{domainId}/set_primary` | `sites.domains.setPrimary` | 设为主域名 |

#### 4.2.3 部署管理 (Deploy)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/sites/{siteId}/deployments` | `sites.deployments.list` | 部署历史列表 |
| POST | `/app/v3/api/sites/{siteId}/deployments` | `sites.deployments.create` | 发起部署 |
| GET | `/app/v3/api/sites/{siteId}/deployments/{deploymentId}` | `sites.deployments.retrieve` | 部署详情 |
| POST | `/app/v3/api/sites/{siteId}/deployments/{deploymentId}/rollback` | `sites.deployments.rollback` | 回滚部署 |
| GET | `/app/v3/api/sites/{siteId}/deployments/{deploymentId}/logs` | `sites.deployments.logs` | 部署日志 |

#### 4.2.4 环境变量 (EnvVariable)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/sites/{siteId}/env_variables` | `sites.envVariables.list` | 环境变量列表 |
| POST | `/app/v3/api/sites/{siteId}/env_variables` | `sites.envVariables.create` | 创建环境变量 |
| PATCH | `/app/v3/api/sites/{siteId}/env_variables/{variableId}` | `sites.envVariables.update` | 更新环境变量 |
| DELETE | `/app/v3/api/sites/{siteId}/env_variables/{variableId}` | `sites.envVariables.delete` | 删除环境变量 |
| POST | `/app/v3/api/sites/{siteId}/env_variables/batch_update` | `sites.envVariables.batchUpdate` | 批量更新 |

#### 4.2.5 SSL 证书 (Certificate)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/certificates` | `certificates.list` | 证书列表 |
| POST | `/app/v3/api/certificates` | `certificates.create` | 申请证书 |
| GET | `/app/v3/api/certificates/{certificateId}` | `certificates.retrieve` | 证书详情 |
| POST | `/app/v3/api/certificates/{certificateId}/renew` | `certificates.renew` | 续期证书 |
| DELETE | `/app/v3/api/certificates/{certificateId}` | `certificates.delete` | 删除证书 |
| POST | `/app/v3/api/certificates/upload` | `certificates.upload` | 上传自定义证书 |

#### 4.2.6 监控 (Monitor)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/sites/{siteId}/health_checks` | `sites.healthChecks.list` | 健康检查配置列表 |
| POST | `/app/v3/api/sites/{siteId}/health_checks` | `sites.healthChecks.create` | 创建健康检查 |
| PATCH | `/app/v3/api/sites/{siteId}/health_checks/{checkId}` | `sites.healthChecks.update` | 更新健康检查 |
| DELETE | `/app/v3/api/sites/{siteId}/health_checks/{checkId}` | `sites.healthChecks.delete` | 删除健康检查 |
| GET | `/app/v3/api/sites/{siteId}/health_checks/{checkId}/results` | `sites.healthChecks.results` | 检查结果历史 |
| GET | `/app/v3/api/sites/{siteId}/metrics` | `sites.metrics.retrieve` | 站点性能指标 |

### 4.3 Backend API 路由定义

#### 4.3.1 Nginx 管理 (Nginx)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/backend/v3/api/nginx/configs` | `nginx.configs.list` | Nginx 配置列表 |
| POST | `/backend/v3/api/nginx/configs` | `nginx.configs.create` | 创建 Nginx 配置 |
| GET | `/backend/v3/api/nginx/configs/{configId}` | `nginx.configs.retrieve` | 配置详情 |
| PUT | `/backend/v3/api/nginx/configs/{configId}` | `nginx.configs.update` | 更新配置 |
| POST | `/backend/v3/api/nginx/configs/{configId}/validate` | `nginx.configs.validate` | 校验配置 |
| POST | `/backend/v3/api/nginx/configs/{configId}/deploy` | `nginx.configs.deploy` | 部署配置 |
| POST | `/backend/v3/api/nginx/reload` | `nginx.reload` | 热加载 Nginx |
| GET | `/backend/v3/api/nginx/status` | `nginx.status.retrieve` | Nginx 状态 |
| GET | `/backend/v3/api/nginx/sites` | `nginx.sites.list` | Nginx 站点列表 |

#### 4.3.2 服务器管理 (Server)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/backend/v3/api/servers` | `servers.list` | 服务器列表 |
| POST | `/backend/v3/api/servers` | `servers.create` | 注册服务器 |
| GET | `/backend/v3/api/servers/{serverId}` | `servers.retrieve` | 服务器详情 |
| PATCH | `/backend/v3/api/servers/{serverId}` | `servers.update` | 更新服务器 |
| DELETE | `/backend/v3/api/servers/{serverId}` | `servers.delete` | 删除服务器 |
| POST | `/backend/v3/api/servers/{serverId}/test_connection` | `servers.testConnection` | 测试连接 |
| GET | `/backend/v3/api/servers/{serverId}/resources` | `servers.resources.retrieve` | 资源使用情况 |

#### 4.3.3 审计日志 (Audit)

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/backend/v3/api/audit_logs` | `auditLogs.list` | 审计日志列表 |
| GET | `/backend/v3/api/audit_logs/{logId}` | `auditLogs.retrieve` | 日志详情 |

### 4.4 Open API 路由定义

| Method | Path | operationId | Security | 说明 |
| --- | --- | --- | --- | --- |
| GET | `/deploy/v3/api/webhooks/deploy` | `webhooks.deploy.create` | API Key | Webhook 触发部署 |
| POST | `/deploy/v3/api/webhooks/deploy` | `webhooks.deploy.create` | API Key | Webhook 触发部署 |
| GET | `/deploy/v3/api/sites/{siteId}/status` | `sites.status.retrieve` | API Key | 站点状态查询 |

### 4.5 请求/响应 Schema 示例

#### 4.5.1 创建站点请求

```yaml
CreateSiteRequest:
  type: object
  required: [name, siteType]
  properties:
    name:
      type: string
      minLength: 1
      maxLength: 100
      description: 站点名称
    slug:
      type: string
      minLength: 1
      maxLength: 100
      pattern: '^[a-z0-9][a-z0-9\-]*[a-z0-9]$'
      description: URL 友好的站点标识
    description:
      type: string
      maxLength: 500
    siteType:
      type: integer
      enum: [1, 2, 3, 4, 5, 6]
      description: "1=static, 2=spa, 3=node, 4=php, 5=python, 6=custom"
    runtimeConfig:
      type: object
      properties:
        buildCommand:
          type: string
        outputDirectory:
          type: string
        nodeVersion:
          type: string
        installCommand:
          type: string
        startCommand:
          type: string
```

#### 4.5.2 站点响应

```yaml
SiteResponse:
  type: object
  properties:
    id:
      type: string
      description: 站点 UUID
    name:
      type: string
    slug:
      type: string
    siteType:
      type: integer
    status:
      type: integer
    domains:
      type: array
      items:
        $ref: '#/components/schemas/DomainSummary'
    latestDeployment:
      $ref: '#/components/schemas/DeploymentSummary'
    runtimeConfig:
      type: object
    createdAt:
      type: string
      format: date-time
    updatedAt:
      type: string
      format: date-time
```

#### 4.5.3 Nginx 配置 Schema

```yaml
CreateNginxConfigRequest:
  type: object
  required: [configType, configName, configContent]
  properties:
    configType:
      type: integer
      enum: [1, 2, 3, 4]
      description: "1=site, 2=upstream, 3=ssl, 4=custom"
    configName:
      type: string
      maxLength: 200
    configContent:
      type: string
      description: Nginx 配置内容
    siteId:
      type: string
    domainId:
      type: string

NginxConfigResponse:
  type: object
  properties:
    id:
      type: string
    configType:
      type: integer
    configName:
      type: string
    configContent:
      type: string
    configHash:
      type: string
    isActive:
      type: boolean
    versionNo:
      type: integer
    deployedAt:
      type: string
      format: date-time
    status:
      type: integer
```

#### 4.5.4 部署请求

```yaml
CreateDeploymentRequest:
  type: object
  required: [deployType]
  properties:
    deployType:
      type: integer
      enum: [1, 2, 3, 4]
      description: "1=upload, 2=git, 3=ci_cd, 4=api"
    versionTag:
      type: string
      maxLength: 100
    commitHash:
      type: string
    sourceRef:
      type: string
      description: Git URL 或上传路径
    environment:
      type: string
      default: production
    idempotencyKey:
      type: string
      maxLength: 200

DeploymentResponse:
  type: object
  properties:
    id:
      type: string
    siteId:
      type: string
    deployType:
      type: integer
    versionTag:
      type: string
    status:
      type: integer
    startedAt:
      type: string
      format: date-time
    completedAt:
      type: string
      format: date-time
    durationMs:
      type: integer
      format: int64
```

### 4.6 错误响应规范

所有错误遵循 RFC 9457 Problem Details：

```json
{
  "type": "https://api.sdkwork.com/errors/validation-error",
  "title": "Validation Error",
  "status": 422,
  "detail": "The request body contains invalid fields",
  "instance": "/app/v3/api/sites",
  "requestId": "req_01HXYZ123",
  "errors": [
    {
      "field": "name",
      "message": "Site name is required",
      "code": "required"
    }
  ]
}
```

---

## 5. SDK 设计

### 5.1 SDK 族谱

遵循 SDKWORK SDK_SPEC.md：

| SDK Family | 来源 | 语言 | 说明 |
| --- | --- | --- | --- |
| sdkwork-deploy-sdk | sdkwork-deploy-open-api | TypeScript, Rust | Open API SDK |
| sdkwork-deploy-app-sdk | sdkwork-deploy-app-api | TypeScript, Rust, Dart | App API SDK |
| sdkwork-deploy-backend-sdk | sdkwork-deploy-backend-api | TypeScript, Rust | Backend API SDK |

### 5.2 SDK 目录结构

```
sdks/
├── sdkwork-deploy-sdk/
│   ├── .sdkwork-assembly.json
│   ├── specs/
│   │   └── component.spec.json
│   ├── openapi/
│   │   └── sdkwork-deploy-open-api.sdkgen.yaml
│   └── generated/
│       └── server-openapi/
│           ├── typescript/
│           └── rust/
│
├── sdkwork-deploy-app-sdk/
│   ├── .sdkwork-assembly.json
│   ├── specs/
│   │   └── component.spec.json
│   ├── openapi/
│   │   └── sdkwork-deploy-app-api.sdkgen.yaml
│   └── generated/
│       └── server-openapi/
│           ├── typescript/
│           ├── rust/
│           └── dart/
│
└── sdkwork-deploy-backend-sdk/
    ├── .sdkwork-assembly.json
    ├── specs/
    │   └── component.spec.json
    ├── openapi/
    │   └── sdkwork-deploy-backend-api.sdkgen.yaml
    └── generated/
        └── server-openapi/
            ├── typescript/
            └── rust/
```

### 5.3 SDK 使用示例

#### TypeScript App SDK

```typescript
import { DeployClient } from '@sdkwork/deploy-app-sdk';

const client = new DeployClient({
  baseUrl: 'https://api.deploy.sdkwork.com',
  tokenManager: globalTokenManager,
});

// 站点管理
const sites = await client.sites.list({ page: 1, pageSize: 20 });
const site = await client.sites.create({
  name: 'My Website',
  slug: 'my-website',
  siteType: 1,
});
await client.sites.activate(site.id);

// 部署
const deployment = await client.sites.deployments.create(site.id, {
  deployType: 1,
  versionTag: 'v1.0.0',
});

// 域名
await client.sites.domains.create(site.id, {
  hostname: 'example.com',
  isPrimary: true,
});
```

#### Rust SDK

```rust
use sdkwork_deploy_app_sdk::DeployClient;

let client = DeployClient::builder()
    .base_url("https://api.deploy.sdkwork.com")
    .token_manager(global_token_manager)
    .build()?;

// 站点管理
let sites = client.sites().list(ListSitesParams::default()).await?;
let site = client.sites().create(CreateSiteRequest {
    name: "My Website".into(),
    slug: Some("my-website".into()),
    site_type: 1,
    ..Default::default()
}).await?;

// 部署
let deployment = client.sites().deployments()
    .create(&site.id, CreateDeploymentRequest {
        deploy_type: 1,
        version_tag: Some("v1.0.0".into()),
        ..Default::default()
    }).await?;
```

### 5.4 SDK 依赖声明

```yaml
# .sdkwork-assembly.json
{
  "sdkOwner": "sdkwork-deploy",
  "apiAuthority": "sdkwork-deploy-app-api",
  "sdkFamily": "sdkwork-deploy-app-sdk",
  "sdkDependencies": [
    {
      "workspace": "sdkwork-appbase-app-sdk",
      "role": "foundation",
      "required": true,
      "dependencyMode": "consumer-sdk",
      "apiPrefix": "/app/v3/api",
      "generatedTransportImportPolicy": "forbidden",
      "packageByLanguage": {
        "typescript": "@sdkwork/appbase-app-sdk",
        "rust": "sdkwork_appbase_app_sdk"
      }
    }
  ],
  "ownerOnlyOperationCount": 42
}
```

---

## 6. Nginx 兼容设计

### 6.1 Nginx 配置生成器

核心能力：将站点配置转换为标准 Nginx 配置文件。

#### 6.1.1 配置模板体系

```rust
pub struct NginxConfigTemplates {
    pub site_static: &'static str,      // 静态站点
    pub site_spa: &'static str,          // SPA 应用
    pub site_node: &'static str,         // Node.js 反向代理
    pub site_php: &'static str,          # PHP-FPM
    pub upstream: &'static str,          // 上游服务
    pub ssl: &'static str,              // SSL 配置
    pub gzip: &'static str,             // Gzip 压缩
    pub security_headers: &'static str,  // 安全头
    pub rate_limit: &'static str,        // 限流
    pub access_log: &'static str,        // 日志格式
}
```

#### 6.1.2 静态站点配置示例

```nginx
# Generated by SDKWork Deploy Server
# Site: {site_name}
# Domain: {domain}
# Generated at: {timestamp}

server {
    listen 80;
    listen [::]:80;
    server_name {domain};

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name {domain};

    # SSL Configuration
    ssl_certificate {cert_path}/fullchain.pem;
    ssl_certificate_key {cert_path}/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # Security Headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;

    # Root directory
    root {site_root}/public;
    index index.html;

    # Gzip
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/javascript application/json application/javascript application/xml+rss application/atom+xml image/svg+xml;

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # SPA fallback
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Health check
    location /healthz {
        access_log off;
        return 200 'ok';
        add_header Content-Type text/plain;
    }

    # Deny access to hidden files
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }

    # Logging
    access_log {log_path}/{domain}.access.log;
    error_log {log_path}/{domain}.error.log;
}
```

#### 6.1.3 Node.js 反向代理配置

```nginx
upstream {site_slug}_backend {
    server 127.0.0.1:{app_port};
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name {domain};

    # SSL (同上)

    location / {
        proxy_pass http://{site_slug}_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Request-ID $request_id;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Buffering
        proxy_buffering off;
        proxy_buffer_size 4k;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://{site_slug}_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
}
```

### 6.2 Nginx 配置解析器

支持读取和解析现有 Nginx 配置：

```rust
pub struct NginxConfigParser;

impl NginxConfigParser {
    /// 解析 Nginx 配置文件为结构化数据
    pub fn parse(content: &str) -> Result<NginxConfig, ParseError>;

    /// 提取 server 块
    pub fn extract_server_blocks(content: &str) -> Vec<ServerBlock>;

    /// 提取 upstream 块
    pub fn extract_upstream_blocks(content: &str) -> Vec<UpstreamBlock>;

    /// 提取 SSL 配置
    pub fn extract_ssl_config(block: &ServerBlock) -> Option<SslConfig>;

    /// 提取 location 块
    pub fn extract_locations(block: &ServerBlock) -> Vec<LocationBlock>;
}
```

### 6.3 Nginx 进程管理

```rust
pub struct NginxProcessManager {
    nginx_bin: PathBuf,
    config_path: PathBuf,
}

impl NginxProcessManager {
    /// 校验配置
    pub async fn test_config(&self) -> Result<NginxTestResult>;

    /// 热加载
    pub async fn reload(&self) -> Result<()>;

    /// 获取状态
    pub async fn status(&self) -> Result<NginxStatus>;

    /// 获取版本
    pub async fn version(&self) -> Result<String>;

    /// 列出活跃站点
    pub async fn list_active_sites(&self) -> Result<Vec<NginxSiteInfo>>;
}
```

### 6.4 完整 Nginx 指令支持

| 类别 | 支持的指令 |
| --- | --- |
| 核心 | listen, server_name, root, index, error_page, keepalive_timeout |
| SSL | ssl_certificate, ssl_certificate_key, ssl_protocols, ssl_ciphers, ssl_session_cache, ssl_stapling |
| 代理 | proxy_pass, proxy_set_header, proxy_http_version, proxy_buffering, proxy_connect_timeout |
| 缓存 | expires, add_header Cache-Control, open_file_cache |
| 压缩 | gzip, gzip_types, gzip_comp_level, gzip_vary |
| 安全 | add_header X-Frame-Options, X-Content-Type-Options, HSTS, CSP |
| 日志 | access_log, error_log, log_format |
| 限流 | limit_req_zone, limit_req, limit_conn_zone, limit_conn |
| 重写 | rewrite, return, try_files |
| WebSocket | proxy_set_header Upgrade, proxy_read_timeout |
| 上游 | upstream, server, keepalive, weight, max_fails, fail_timeout |
| 流式 | proxy_buffering off, proxy_read_timeout (长连接) |

---

## 7. Rust 开源组件集成

### 7.1 核心依赖

```toml
[dependencies]
# HTTP 框架
axum = "0.7"
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression-full"] }

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio", "tls-rustls", "postgres", "sqlite", "uuid", "chrono", "json"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 配置
toml = "0.8"
figment = { version = "0.10", features = ["toml", "env"] }

# 错误处理
thiserror = "1"
anyhow = "1"

# 日志和追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.22"
opentelemetry = "0.21"

# UUID 和 ID
uuid = { version = "1", features = ["v4", "v7"] }

# 时间
chrono = { version = "0.4", features = ["serde"] }

# HTTP 客户端
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# 加密
rustls = "0.21"
ring = "0.17"
aes-gcm = "0.10"

# SSH 远程
async-ssh2-tokio = "0.4"

# ACME (Let's Encrypt)
instant-acme = "0.4"

# 配置验证
validator = { version = "0.16", features = ["derive"] }

# 限流
governor = "0.6"

# JSON Schema
jsonschema = "0.17"

# Nginx 配置解析
nom = "7"  # 解析器组合子

# 定时任务
tokio-cron-scheduler = "0.10"

# 指标
prometheus = "0.13"
metrics = "0.21"
metrics-exporter-prometheus = "0.12"
```

### 7.2 组件用途映射

| 组件 | 用途 | 对标 Nginx 能力 |
| --- | --- | --- |
| axum + tower-http | HTTP 服务、中间件、CORS | Nginx HTTP 模块 |
| sqlx + PostgreSQL | 持久化存储 | Nginx 无（使用文件） |
| redis | 缓存、限流计数器 | Nginx limit_req |
| rustls | TLS 处理 | Nginx SSL 模块 |
| instant-acme | 自动证书申请 | certbot |
| tokio-cron-scheduler | 定时任务（证书续期） | cron |
| reqwest | HTTP 健康检查 | Nginx health_check |
| async-ssh2-tokio | 远程服务器管理 | 无 |
| nom | Nginx 配置解析 | Nginx -t |
| prometheus | 指标采集 | Nginx stub_status |
| governor | API 限流 | Nginx limit_req |
| ring + aes-gcm | 环境变量加密 | 无 |
| tracing + opentelemetry | 链路追踪 | 无 |

### 7.3 Nginx 配置解析（nom 实现）

```rust
use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
};

/// 解析 Nginx 配置指令
pub fn parse_directive(input: &str) -> IResult<&str, NginxDirective> {
    let (input, name) = preceded(
        multispace0,
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    )(input)?;
    let (input, _) = multispace1(input)?;
    let (input, value) = take_until(";")(input)?;
    let (input, _) = char(';')(input)?;
    Ok((input, NginxDirective {
        name: name.to_string(),
        value: value.to_string(),
    }))
}

/// 解析 server 块
pub fn parse_server_block(input: &str) -> IResult<&str, ServerBlock> {
    // ...
}
```

---

## 8. UI/UX 功能规划

### 8.1 功能模块规划

本服务仅提供后端 API，前端由独立进程开发。以下是 API 支持的完整功能规划：

#### 8.1.1 仪表盘 (Dashboard)

| 功能 | API 支持 | 说明 |
| --- | --- | --- |
| 站点总数统计 | `GET /app/v3/api/dashboard/stats` | 站点、域名、部署、证书统计 |
| 最近部署 | `GET /app/v3/api/dashboard/recent_deployments` | 最近 10 次部署 |
| 健康状态总览 | `GET /app/v3/api/dashboard/health_overview` | 所有站点健康状态 |
| 资源使用概览 | `GET /app/v3/api/dashboard/resource_usage` | 服务器资源使用 |
| 告警通知 | `GET /app/v3/api/dashboard/alerts` | 最近告警列表 |

#### 8.1.2 站点管理

| 功能 | 流程 |
| --- | --- |
| 创建站点 | 填写信息 → 选择运行时 → 配置构建命令 → 创建 |
| 部署站点 | 选择部署方式 → 上传/选择 Git → 触发构建 → 部署 |
| 域名绑定 | 添加域名 → 验证所有权 → 申请 SSL → 生效 |
| 环境变量 | 添加/编辑变量 → 加密存储 → 部署时注入 |
| 回滚 | 选择历史版本 → 确认回滚 → 执行回滚 |

#### 8.1.3 Nginx 管理

| 功能 | 说明 |
| --- | --- |
| 配置编辑器 | 语法高亮、实时校验、配置预览 |
| 配置模板 | 预置常用模板、自定义模板 |
| 配置对比 | 版本间差异对比 |
| 热加载 | 一键校验 + 热加载 |
| 站点列表 | 所有 Nginx 站点一览 |

#### 8.1.4 监控告警

| 功能 | 说明 |
| --- | --- |
| 可用性监控 | HTTP/TCP/Ping 检查，自定义间隔 |
| 响应时间图表 | 实时 + 历史响应时间趋势 |
| 错误率统计 | 4xx/5xx 错误率趋势 |
| 告警规则 | 自定义告警条件、通知渠道 |
| 告警历史 | 告警记录、处理状态 |

### 8.2 API 端点扩展（仪表盘和统计）

| Method | Path | operationId | 说明 |
| --- | --- | --- | --- |
| GET | `/app/v3/api/dashboard/stats` | `dashboard.stats.retrieve` | 总体统计 |
| GET | `/app/v3/api/dashboard/recent_deployments` | `dashboard.recentDeployments.list` | 最近部署 |
| GET | `/app/v3/api/dashboard/health_overview` | `dashboard.healthOverview.retrieve` | 健康总览 |
| GET | `/app/v3/api/dashboard/resource_usage` | `dashboard.resourceUsage.retrieve` | 资源使用 |
| GET | `/app/v3/api/dashboard/alerts` | `dashboard.alerts.list` | 告警列表 |

### 8.3 WebSocket 实时推送

| 事件 | 说明 |
| --- | --- |
| `deployment.status` | 部署状态变更实时推送 |
| `deployment.log` | 部署日志实时流 |
| `health.status` | 健康状态变更 |
| `alert.new` | 新告警通知 |
| `nginx.reload` | Nginx 热加载结果 |

```
WS /app/v3/api/ws?token={access_token}

// 订阅
{ "action": "subscribe", "events": ["deployment.status", "deployment.log"], "siteId": "xxx" }

// 推送
{ "event": "deployment.status", "data": { "deploymentId": "xxx", "status": 3 } }
```

---

## 9. 安全设计

### 9.1 认证与授权

遵循 SDKWORK SECURITY_SPEC.md 和 IAM_SPEC.md：

| 安全层 | 实现 |
| --- | --- |
| 认证 | Dual Token: AuthToken (JWT) + Access-Token |
| 授权 | RBAC: 角色 + 权限检查 |
| 租户隔离 | tenant_id 强制过滤 |
| API Key | Open API 使用 X-API-Key |
| CORS | 严格域名白名单 |
| Rate Limit | 基于 tenant_id + IP 的限流 |
| HTTPS | 强制 HTTPS，HSTS |

### 9.2 敏感数据保护

| 数据 | 保护方式 |
| --- | --- |
| 环境变量值 | AES-256-GCM 加密存储 |
| SSL 私钥 | 文件系统权限 600 |
| 数据库密码 | 环境变量 / Secret 文件 |
| API Key | SHA-256 哈希存储，只显示前 8 位 |
| 日志 | 自动脱敏：密码、Token、密钥 |

### 9.3 审计追踪

所有写操作记录审计日志：
- 操作人 (operator_id)
- 操作类型 (action)
- 目标对象 (target_type, target_id)
- 变更内容 (changes JSON)
- 请求链路 (request_id, ip_address)

---

## 10. 部署方案

### 10.1 部署模式

| 模式 | 说明 | 数据库 | Redis |
| --- | --- | --- | --- |
| Server | Linux 服务器部署 | PostgreSQL | Redis |
| Container | Docker 容器部署 | PostgreSQL | Redis |
| Desktop | 本地桌面部署 | SQLite | 可选 |

### 10.2 Nginx 部署配置

遵循 SDKWORK NGINX_SPEC.md：

```nginx
# /etc/nginx/sites-enabled/sdkwork/api.deploy.sdkwork.com.conf

upstream deploy_api_server {
    server 127.0.0.1:3900;
    keepalive 32;
}

server {
    listen 80;
    server_name api.deploy.sdkwork.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.deploy.sdkwork.com;

    ssl_certificate /opt/certs/letsencrypt/live/deploy.sdkwork.com/fullchain.pem;
    ssl_certificate_key /opt/certs/letsencrypt/live/deploy.sdkwork.com/privkey.pem;

    client_max_body_size 1100m;

    location / {
        proxy_pass http://deploy_api_server;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_buffering off;
    }

    location /ws {
        proxy_pass http://deploy_api_server;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }

    location /healthz {
        proxy_pass http://deploy_api_server;
        access_log off;
    }

    location /readyz {
        proxy_pass http://deploy_api_server;
        access_log off;
    }
}
```

### 10.3 健康检查端点

| 端点 | 用途 |
| --- | --- |
| `GET /healthz` | 存活检查 |
| `GET /readyz` | 就绪检查（含数据库、Redis 连接） |

### 10.4 环境变量

```bash
# 部署模式
SDKWORK_DEPLOY_MODE=server                    # server|container|desktop
SDKWORK_DEPLOY_CONFIG_FILE=/etc/sdkwork/deploy/deploy.toml

# 数据库
SDKWORK_DEPLOY_DATABASE_ENGINE=postgresql     # postgresql|sqlite
SDKWORK_DEPLOY_DATABASE_HOST=localhost
SDKWORK_DEPLOY_DATABASE_PORT=5432
SDKWORK_DEPLOY_DATABASE_NAME=sdkwork_deploy
SDKWORK_DEPLOY_DATABASE_USERNAME=sdkwork
SDKWORK_DEPLOY_DATABASE_PASSWORD_FILE=/etc/sdkwork/deploy/database.secret
SDKWORK_DEPLOY_DATABASE_SSL_MODE=require
SDKWORK_DEPLOY_DATABASE_MAX_CONNECTIONS=16

# Redis
SDKWORK_DEPLOY_REDIS_ENABLED=true
SDKWORK_DEPLOY_REDIS_HOST=localhost
SDKWORK_DEPLOY_REDIS_PORT=6379
SDKWORK_DEPLOY_REDIS_KEY_PREFIX=deploy
SDKWORK_DEPLOY_REDIS_MAX_CONNECTIONS=16

# Nginx
SDKWORK_DEPLOY_NGINX_BIN=/usr/sbin/nginx
SDKWORK_DEPLOY_NGINX_CONFIG_PATH=/etc/nginx
SDKWORK_DEPLOY_NGINX_SITES_PATH=/etc/nginx/sites-enabled/sdkwork
SDKWORK_DEPLOY_NGINX_CERT_PATH=/opt/certs/letsencrypt/live

# 服务
SDKWORK_DEPLOY_HOST=0.0.0.0
SDKWORK_DEPLOY_PORT=3900
SDKWORK_DEPLOY_LOG_LEVEL=info
SDKWORK_DEPLOY_LOG_FORMAT=json
```

---

## 附录 A：Cargo.toml Workspace 配置

```toml
[workspace]
resolver = "2"
members = [
    "crates/sdkwork-deploy-api-server",
    "crates/sdkwork-router-site-app-api",
    "crates/sdkwork-router-nginx-backend-api",
    "crates/sdkwork-router-deploy-app-api",
    "crates/sdkwork-router-domain-app-api",
    "crates/sdkwork-router-cert-app-api",
    "crates/sdkwork-router-monitor-app-api",
    "crates/sdkwork-deploy-site-service",
    "crates/sdkwork-deploy-nginx-service",
    "crates/sdkwork-deploy-deploy-service",
    "crates/sdkwork-deploy-domain-service",
    "crates/sdkwork-deploy-cert-service",
    "crates/sdkwork-deploy-monitor-service",
    "crates/sdkwork-deploy-site-repository-sqlx",
    "crates/sdkwork-deploy-nginx-repository-sqlx",
    "crates/sdkwork-deploy-deploy-repository-sqlx",
    "crates/sdkwork-deploy-nginx-adapter",
    "crates/sdkwork-deploy-ssh-adapter",
    "crates/sdkwork-deploy-cert-adapter",
    "crates/sdkwork-deploy-worker",
]

[workspace.dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "tls-rustls", "postgres", "sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tracing = "0.1"
uuid = { version = "1", features = ["v4", "v7"] }
chrono = { version = "0.4", features = ["serde"] }
```

## 附录 B：目录结构总览

```
sdkwork-deploy-server/
├── AGENTS.md
├── CLAUDE.md
├── GEMINI.md
├── CODEX.md
├── Cargo.toml                          # Workspace 配置
├── README.md
├── apis/                               # API 合约
│   ├── open-api/deploy/
│   │   └── openapi.yaml
│   ├── app-api/deploy/
│   │   └── openapi.yaml
│   └── backend-api/deploy/
│       └── openapi.yaml
├── crates/                             # Rust crates
│   ├── sdkwork-deploy-api-server/
│   ├── sdkwork-router-site-app-api/
│   ├── sdkwork-router-nginx-backend-api/
│   ├── sdkwork-router-deploy-app-api/
│   ├── sdkwork-router-domain-app-api/
│   ├── sdkwork-router-cert-app-api/
│   ├── sdkwork-router-monitor-app-api/
│   ├── sdkwork-deploy-site-service/
│   ├── sdkwork-deploy-nginx-service/
│   ├── sdkwork-deploy-deploy-service/
│   ├── sdkwork-deploy-domain-service/
│   ├── sdkwork-deploy-cert-service/
│   ├── sdkwork-deploy-monitor-service/
│   ├── sdkwork-deploy-site-repository-sqlx/
│   ├── sdkwork-deploy-nginx-repository-sqlx/
│   ├── sdkwork-deploy-deploy-repository-sqlx/
│   ├── sdkwork-deploy-nginx-adapter/
│   ├── sdkwork-deploy-ssh-adapter/
│   ├── sdkwork-deploy-cert-adapter/
│   └── sdkwork-deploy-worker/
├── sdks/                               # SDK 族
│   ├── sdkwork-deploy-sdk/
│   ├── sdkwork-deploy-app-sdk/
│   └── sdkwork-deploy-backend-sdk/
├── configs/                            # 配置模板
│   ├── deploy.toml
│   └── nginx/
│       ├── static-site.conf.template
│       ├── spa-site.conf.template
│       ├── node-proxy.conf.template
│       └── php-fpm.conf.template
├── deployments/                        # 部署配置
│   ├── docker/
│   │   ├── Dockerfile
│   │   └── docker-compose.yml
│   └── systemd/
│       └── sdkwork-deploy.service
├── migrations/                         # 数据库迁移
│   ├── 001_create_deploy_site.sql
│   ├── 002_create_deploy_domain.sql
│   ├── 003_create_deploy_nginx_config.sql
│   ├── 004_create_deploy_certificate.sql
│   ├── 005_create_deploy_deployment.sql
│   ├── 006_create_deploy_env_variable.sql
│   ├── 007_create_deploy_health_check.sql
│   ├── 008_create_deploy_health_result.sql
│   └── 009_create_deploy_audit_log.sql
├── scripts/                            # 脚本
│   ├── nginx-plan.sh
│   ├── nginx-render.sh
│   └── nginx-deploy.sh
├── tests/                              # 集成测试
├── tools/                              # 开发工具
├── docs/                               # 文档
│   └── DESIGN_REPORT.md
└── .sdkwork/                           # 本地 workspace
    ├── README.md
    ├── skills/
    └── plugins/
```

---

## 附录 C：合规检查清单

### 架构合规

- [x] Crate 按职责命名：api-server, router-*-app-api, *-service, *-repository-sqlx, *-adapter, *-worker
- [x] 禁止通用后缀：product, runtime, backend, core, common, manager
- [x] lib.rs 仅包含模块声明和重导出
- [x] 路由 crate 包含 paths.rs, routes.rs, handlers.rs, manifest.rs
- [x] Service crate 不依赖 HTTP 框架类型
- [x] Repository crate 不包含业务策略
- [x] Handler 不解析原始 Header

### API 合规

- [x] OpenAPI 3.1.2 合约
- [x] App API 前缀 /app/v3/api
- [x] Backend API 前缀 /backend/v3/api
- [x] Open API 前缀 /deploy/v3/api
- [x] operationId 遵循 resource.action 格式
- [x] Dual Token 认证 (App/Backend)
- [x] API Key 认证 (Open API)
- [x] Problem Detail 错误响应

### 数据库合规

- [x] 表名使用 deploy_ 前缀
- [x] 标准字段：id, uuid, tenant_id, created_at, updated_at, version
- [x] 雪花 ID 策略
- [x] 逻辑类型映射
- [x] 索引命名规范

### SDK 合规

- [x] SDK Family 命名：sdkwork-deploy-*-sdk
- [x] .sdkwork-assembly.json 元数据
- [x] sdkDependencies 声明
- [x] 生成代码不手编辑

### Nginx 兼容

- [x] 标准 Nginx 配置生成
- [x] TLS 1.2/1.3 支持
- [x] WebSocket 代理
- [x] 流式响应支持
- [x] 热加载 (nginx -s reload)
- [x] 配置校验 (nginx -t)

### 安全合规

- [x] 租户数据隔离
- [x] 敏感数据加密
- [x] 审计日志
- [x] Rate Limiting
- [x] CORS 配置
- [x] HTTPS 强制

---

*本报告遵循 SDKWORK 规范体系设计，后续实现需严格按照各规范文件执行。*
