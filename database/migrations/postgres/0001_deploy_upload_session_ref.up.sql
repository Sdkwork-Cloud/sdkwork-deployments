-- deploy upload session references (Drive-backed artifact lifecycle)
CREATE TABLE IF NOT EXISTS deploy_upload_session_ref (
    id BIGINT PRIMARY KEY NOT NULL,
    uuid VARCHAR(36) NOT NULL,
    tenant_id BIGINT NOT NULL,
    site_id BIGINT NULL,
    drive_upload_session_id VARCHAR(128) NOT NULL,
    drive_upload_item_id VARCHAR(128) NULL,
    drive_space_id VARCHAR(128) NULL,
    drive_node_id VARCHAR(128) NULL,
    package_type INTEGER NOT NULL,
    file_name VARCHAR(500) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    content_length BIGINT NOT NULL,
    checksum VARCHAR(128) NULL,
    status INTEGER NOT NULL DEFAULT 0,
    idempotency_key VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_upload_session_ref_uuid
    ON deploy_upload_session_ref (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_upload_session_ref_idempotency
    ON deploy_upload_session_ref (tenant_id, idempotency_key)
    WHERE deleted_at IS NULL;
