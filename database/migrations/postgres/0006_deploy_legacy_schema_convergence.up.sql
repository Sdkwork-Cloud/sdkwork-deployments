-- sdkwork:migration
-- id: 0006_deploy_legacy_schema_convergence
-- engine: postgres
-- module: deploy
-- purpose: Converge databases initialized before 2026-07-31 (consolidated
--   baseline 5ce5b0c) to the current deploy contract:
--   * deploy_domain was redesigned from site-bound legacy rows (site_id,
--     hostname, INTEGER status) to DNS-zone-bound rows (zone_id,
--     hostname_ascii, verification_status, VARCHAR(16) status);
--   * deploy_certificate was redesigned from a path-based certificate store
--     to the ACME lifecycle aggregate (certificate_source, ca_profile,
--     idempotency_key, request_sha256, VARCHAR(16) status);
--   * deploy_site gained variant/revision linkage columns;
--   * deploy_server gained cluster/SSH columns.
-- All statements are idempotent; databases already on the current baseline
-- apply this migration as a no-op. Rows that cannot be mapped (for example a
-- legacy domain without a DNS zone) fail loudly with a reconciliation error.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: access-exclusive
-- lock_timeout: 2s
-- statement_timeout: 300s
-- contract_version: 1.0.0

-- ────────────────────────────────────────────────────────────────────────
-- deploy_site: variant/revision linkage (nullable, no backfill needed)
-- ────────────────────────────────────────────────────────────────────────
ALTER TABLE deploy_site ADD COLUMN IF NOT EXISTS default_variant_id BIGINT;
ALTER TABLE deploy_site ADD COLUMN IF NOT EXISTS current_revision_id BIGINT;
ALTER TABLE deploy_site ADD COLUMN IF NOT EXISTS desired_revision_id BIGINT;

-- ────────────────────────────────────────────────────────────────────────
-- deploy_server: cluster/SSH columns
-- ────────────────────────────────────────────────────────────────────────
ALTER TABLE deploy_server ADD COLUMN IF NOT EXISTS cluster_id BIGINT;
ALTER TABLE deploy_server ADD COLUMN IF NOT EXISTS node_role INTEGER NOT NULL DEFAULT 0;
ALTER TABLE deploy_server ADD COLUMN IF NOT EXISTS ssh_user VARCHAR(64);
ALTER TABLE deploy_server ADD COLUMN IF NOT EXISTS ssh_key_path VARCHAR(500);
ALTER TABLE deploy_server ADD COLUMN IF NOT EXISTS description VARCHAR(500);

CREATE INDEX IF NOT EXISTS idx_deploy_server_cluster
    ON deploy_server (tenant_id, cluster_id);

-- ────────────────────────────────────────────────────────────────────────
-- deploy_domain: legacy (site-bound) shape -> current (zone-bound) shape
-- ────────────────────────────────────────────────────────────────────────

-- New contract columns. hostname_type / verification_status carry defaults
-- so existing rows converge without a data backfill.
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS hostname_ascii VARCHAR(253);
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS hostname_type VARCHAR(16) NOT NULL DEFAULT 'EXACT';
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS verification_status VARCHAR(16) NOT NULL DEFAULT 'PENDING';
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS verified_at TIMESTAMPTZ;
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS created_by BIGINT;
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS updated_by BIGINT;
ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS zone_id BIGINT;

-- Backfill hostname_ascii from the legacy hostname column (lowercased).
-- Executed as dynamic SQL so databases on the current baseline (which no
-- longer have the hostname column) skip it at parse time.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'deploy_domain' AND column_name = 'hostname') THEN
        EXECUTE 'UPDATE deploy_domain
                    SET hostname_ascii = LOWER(hostname)
                  WHERE hostname_ascii IS NULL AND hostname IS NOT NULL';
    END IF;
END
$$;

-- Converge column types and defaults. On current-shape databases every
-- ALTER is a no-op (PostgreSQL skips the USING clause when the target type
-- already matches). Legacy indexes whose partial predicates reference the
-- converted columns are dropped first: PostgreSQL rebuilds index predicates
-- against the new column type, and `status = 1` would otherwise fail after
-- `status` becomes VARCHAR(16).
DROP INDEX IF EXISTS idx_deploy_domain_tenant_status;

ALTER TABLE deploy_domain ALTER COLUMN uuid TYPE VARCHAR(36);
ALTER TABLE deploy_domain ALTER COLUMN status TYPE VARCHAR(16)
    USING (CASE status::text
               WHEN '0' THEN 'PENDING'
               WHEN '1' THEN 'ACTIVE'
               WHEN '2' THEN 'PAUSED'
               ELSE 'PENDING'
           END);
ALTER TABLE deploy_domain ALTER COLUMN version SET DEFAULT 1;
ALTER TABLE deploy_domain ALTER COLUMN created_at SET DEFAULT NOW();
ALTER TABLE deploy_domain ALTER COLUMN updated_at SET DEFAULT NOW();
ALTER TABLE deploy_domain ALTER COLUMN tenant_id DROP DEFAULT;

-- Drop legacy shape: constraints first, then columns (indexes on dropped
-- columns are removed by PostgreSQL automatically).
ALTER TABLE deploy_domain DROP CONSTRAINT IF EXISTS uk_deploy_domain_hostname;
ALTER TABLE deploy_domain DROP CONSTRAINT IF EXISTS fk_deploy_domain_site;

ALTER TABLE deploy_domain DROP COLUMN IF EXISTS site_id;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS hostname;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS is_primary;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS is_verified;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS verify_token;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS ssl_enabled;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS ssl_provider;
ALTER TABLE deploy_domain DROP COLUMN IF EXISTS redirect_target;

-- Rebuild the tenant-status index with the current definition (the legacy
-- index shares the name but has a different column set).
DROP INDEX IF EXISTS idx_deploy_domain_tenant_status;

-- zone_id is NOT NULL in the current contract. Legacy rows without a DNS
-- zone cannot be mapped automatically; fail loudly instead of leaving a
-- silently drifted schema.
DO $$
DECLARE
    orphan_count BIGINT;
BEGIN
    SELECT COUNT(*) INTO orphan_count FROM deploy_domain WHERE zone_id IS NULL;
    IF orphan_count > 0 THEN
        RAISE EXCEPTION
            'deploy_domain upgrade requires manual reconciliation: % row(s) have no DNS zone; assign zone_id (or archive the rows) before re-running this migration. Current contract requires zone_id NOT NULL',
            orphan_count;
    END IF;
END
$$;

ALTER TABLE deploy_domain ALTER COLUMN zone_id SET NOT NULL;
ALTER TABLE deploy_domain ALTER COLUMN hostname_ascii SET NOT NULL;

-- New contract constraints and indexes (guarded; current-shape databases
-- already carry them).
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_domain'::regclass
                     AND conname = 'fk_deploy_domain_zone') THEN
        ALTER TABLE deploy_domain ADD CONSTRAINT fk_deploy_domain_zone
            FOREIGN KEY (zone_id) REFERENCES deploy_dns_zone(id);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_domain'::regclass
                     AND conname = 'chk_deploy_domain_hostname_type') THEN
        ALTER TABLE deploy_domain ADD CONSTRAINT chk_deploy_domain_hostname_type
            CHECK (hostname_type IN ('EXACT', 'WILDCARD'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_domain'::regclass
                     AND conname = 'chk_deploy_domain_verification_status') THEN
        ALTER TABLE deploy_domain ADD CONSTRAINT chk_deploy_domain_verification_status
            CHECK (verification_status IN ('PENDING', 'VERIFIED', 'FAILED', 'EXPIRED'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_domain'::regclass
                     AND conname = 'chk_deploy_domain_status') THEN
        ALTER TABLE deploy_domain ADD CONSTRAINT chk_deploy_domain_status
            CHECK (status IN ('ACTIVE', 'PAUSED'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_domain'::regclass
                     AND conname = 'chk_deploy_domain_verified_at') THEN
        ALTER TABLE deploy_domain ADD CONSTRAINT chk_deploy_domain_verified_at
            CHECK (
                (verification_status = 'VERIFIED' AND verified_at IS NOT NULL)
                OR (verification_status <> 'VERIFIED' AND verified_at IS NULL)
            );
    END IF;
END
$$;

CREATE UNIQUE INDEX IF NOT EXISTS uk_deploy_domain_active_hostname
    ON deploy_domain (hostname_ascii)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_domain_tenant_status
    ON deploy_domain (tenant_id, status, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_domain_zone_updated
    ON deploy_domain (tenant_id, zone_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- ────────────────────────────────────────────────────────────────────────
-- deploy_certificate: legacy (path-based) shape -> current (ACME aggregate) shape
-- ────────────────────────────────────────────────────────────────────────

-- New contract columns. status / renewal_status already exist on legacy
-- tables as INTEGER; they are converted below. idempotency_key /
-- request_sha256 are backfilled deterministically from the stable uuid.
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS certificate_source VARCHAR(16) NOT NULL DEFAULT 'MANAGED';
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS ca_profile VARCHAR(32) NOT NULL DEFAULT 'LETS_ENCRYPT_PRODUCTION';
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS preferred_key_algorithm VARCHAR(16) NOT NULL DEFAULT 'ECDSA';
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS renewal_status VARCHAR(16) NOT NULL DEFAULT 'NONE';
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS status VARCHAR(16) NOT NULL DEFAULT 'PENDING';
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128);
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS request_sha256 VARCHAR(64);
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS created_by BIGINT;
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS updated_by BIGINT;
ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

UPDATE deploy_certificate
   SET idempotency_key = 'legacy-' || uuid::text,
       request_sha256 = md5(uuid::text) || md5(uuid::text)
 WHERE idempotency_key IS NULL;

-- Converge column types and defaults. No-op on current-shape databases.
-- Legacy partial indexes whose predicates reference renewal_status / status
-- / not_after are dropped first: PostgreSQL rebuilds index predicates
-- against the new column type, and `status = 1` would otherwise fail after
-- `status` becomes VARCHAR(16).
DROP INDEX IF EXISTS idx_deploy_certificate_expiry;
DROP INDEX IF EXISTS idx_deploy_certificate_renewal;

ALTER TABLE deploy_certificate ALTER COLUMN uuid TYPE VARCHAR(36);
ALTER TABLE deploy_certificate ALTER COLUMN renewal_status TYPE VARCHAR(16)
    USING (CASE renewal_status::text
               WHEN '0' THEN 'NONE'
               WHEN '1' THEN 'PLANNED'
               WHEN '2' THEN 'PROCESSING'
               WHEN '3' THEN 'FAILED'
               ELSE 'NONE'
           END);
ALTER TABLE deploy_certificate ALTER COLUMN status TYPE VARCHAR(16)
    USING (CASE status::text
               WHEN '0' THEN 'PENDING'
               WHEN '1' THEN 'ACTIVE'
               WHEN '2' THEN 'EXPIRED'
               WHEN '3' THEN 'REVOKED'
               ELSE 'PENDING'
           END);
ALTER TABLE deploy_certificate ALTER COLUMN version SET DEFAULT 1;
ALTER TABLE deploy_certificate ALTER COLUMN created_at SET DEFAULT NOW();
ALTER TABLE deploy_certificate ALTER COLUMN updated_at SET DEFAULT NOW();
ALTER TABLE deploy_certificate ALTER COLUMN tenant_id DROP DEFAULT;

-- Drop legacy shape. Indexes on dropped columns are removed automatically.
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS domain_id;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS site_id;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS cert_type;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS issuer;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS subject;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS san_list;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS fingerprint;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS cert_path;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS key_path;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS chain_path;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS not_before;
ALTER TABLE deploy_certificate DROP COLUMN IF EXISTS not_after;

-- Rebuild the renewal index with the current definition (the legacy index
-- was dropped with not_after above).
CREATE INDEX IF NOT EXISTS idx_deploy_certificate_renewal
    ON deploy_certificate (tenant_id, renewal_status, updated_at, id)
    WHERE auto_renew = TRUE AND status IN ('ACTIVE', 'FAILED') AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_deploy_certificate_tenant_updated
    ON deploy_certificate (tenant_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- New contract constraints (guarded). The legacy unique constraint on
-- (tenant_id, idempotency_key) is added as a table constraint; any earlier
-- partial index with the same name (folded migration 0002 shape) is removed
-- first so the constraint-owned index can take the name.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'uk_deploy_certificate_idempotency') THEN
        DROP INDEX IF EXISTS uk_deploy_certificate_idempotency;
        ALTER TABLE deploy_certificate ADD CONSTRAINT uk_deploy_certificate_idempotency
            UNIQUE (tenant_id, idempotency_key);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_source') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_source
            CHECK (certificate_source IN ('MANAGED', 'CUSTOM'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_ca_profile') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_ca_profile
            CHECK (ca_profile IN ('LETS_ENCRYPT_STAGING', 'LETS_ENCRYPT_PRODUCTION', 'CUSTOM'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_key_algorithm') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_key_algorithm
            CHECK (preferred_key_algorithm IN ('RSA', 'ECDSA'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_renewal_status') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_renewal_status
            CHECK (renewal_status IN ('NONE', 'PLANNED', 'PROCESSING', 'FAILED'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_status') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_status
            CHECK (status IN ('PENDING', 'ISSUING', 'ACTIVE', 'EXPIRED', 'FAILED', 'REVOKED'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
                   WHERE conrelid = 'deploy_certificate'::regclass
                     AND conname = 'chk_deploy_certificate_request_hash') THEN
        ALTER TABLE deploy_certificate ADD CONSTRAINT chk_deploy_certificate_request_hash
            CHECK (request_sha256 ~ '^[0-9a-f]{64}$');
    END IF;
END
$$;

-- idempotency_key / request_sha256 are backfilled above and must become
-- NOT NULL to match the contract.
ALTER TABLE deploy_certificate ALTER COLUMN idempotency_key SET NOT NULL;
ALTER TABLE deploy_certificate ALTER COLUMN request_sha256 SET NOT NULL;
