-- sdkwork:migration
-- id: 0001_organization_id_not_null
-- engine: postgres
-- module: sdkwork-deployments
-- purpose: Enforce organization_id NOT NULL DEFAULT on all tables in the
--   consolidated baseline. NULL rows (pre-standard data anomalies) are
--   backfilled with the platform sentinel before NOT NULL is set, and
--   NOT NULL columns without an explicit default receive the sentinel
--   default, keeping existing deployments consistent with fresh baseline
--   installs.
-- reversible: false
-- rollback: forward-fix (sentinel backfill is the canonical fix; NULL
--   organization rows are data anomalies)
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

ALTER TABLE deploy_site ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_site SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_site ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_site ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_dns_zone ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_dns_zone SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_dns_zone ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_dns_zone ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_domain ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_domain SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_domain ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_domain ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_certificate ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_certificate SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_certificate ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_certificate ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_deployment ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_deployment SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_deployment ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_deployment ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_audit_log ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_audit_log SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_audit_log ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_audit_log ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_site_resource ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_site_resource SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_site_resource ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_site_resource ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_site_binding ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_site_binding SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_site_binding ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_site_binding ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_site_revision ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_site_revision SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_site_revision ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_site_revision ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_app ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_app SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_app ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_app ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_build_template ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_build_template SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_build_template ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_build_template ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_app_platform_target ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_app_platform_target SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_app_platform_target ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_app_platform_target ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_source_repository ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_source_repository SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_source_repository ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_source_repository ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_build ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_build SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_build ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_build ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_signing_identity ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_signing_identity SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_signing_identity ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_signing_identity ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_package ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_package SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_package ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_package ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_release_channel ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_release_channel SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_release_channel ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_release_channel ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_channel_rollout ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_channel_rollout SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_channel_rollout ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_channel_rollout ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_usage_event ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_usage_event SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_usage_event ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_usage_event ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_tenant_entitlement_projection ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_tenant_entitlement_projection SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_tenant_entitlement_projection ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_tenant_entitlement_projection ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_site_usage_daily ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_site_usage_daily SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_site_usage_daily ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_site_usage_daily ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_app_database_profile ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_app_database_profile SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_app_database_profile ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_app_database_profile ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_app_database_migration ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_app_database_migration SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_app_database_migration ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_app_database_migration ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_source_event ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_source_event SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_source_event ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_source_event ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_app_environment ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_app_environment SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_app_environment ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_app_environment ALTER COLUMN organization_id SET NOT NULL;

ALTER TABLE deploy_environment_promotion ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;
UPDATE deploy_environment_promotion SET organization_id = 0 WHERE organization_id IS NULL;
ALTER TABLE deploy_environment_promotion ALTER COLUMN organization_id SET DEFAULT 0;
ALTER TABLE deploy_environment_promotion ALTER COLUMN organization_id SET NOT NULL;

COMMIT;
