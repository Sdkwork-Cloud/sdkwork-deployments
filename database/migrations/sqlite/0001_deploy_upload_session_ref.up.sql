CREATE TABLE IF NOT EXISTS deploy_upload_session_ref (
    id INTEGER PRIMARY KEY NOT NULL,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    site_id INTEGER NULL,
    drive_upload_session_id TEXT NOT NULL,
    drive_upload_item_id TEXT NULL,
    drive_space_id TEXT NULL,
    drive_node_id TEXT NULL,
    package_type INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content_length INTEGER NOT NULL,
    checksum TEXT NULL,
    status INTEGER NOT NULL DEFAULT 0,
    idempotency_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_upload_session_ref_uuid
    ON deploy_upload_session_ref (uuid);

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_upload_session_ref_idempotency
    ON deploy_upload_session_ref (tenant_id, idempotency_key)
    WHERE deleted_at IS NULL;
