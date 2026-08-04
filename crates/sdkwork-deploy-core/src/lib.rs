//! Deploy core runtime helpers.

pub mod app_kind_rules;
pub mod package_manifest;
pub mod runtime_env;
pub mod util;
pub mod versioning;

pub use app_kind_rules::{
    package_size_ceiling, required_identity_field, validate_app_kind_platform,
    validate_package_format_for_platform, validate_package_size, validate_platform_identity,
    RequiredIdentityField, DOUYIN_MINIPROGRAM_MAIN_PACKAGE_BYTES,
    DOUYIN_MINIPROGRAM_TOTAL_PACKAGE_BYTES, PROCESS_BUNDLE_MAXIMUM_BYTES, WEB_BUNDLE_MAXIMUM_BYTES,
    WECHAT_MINIPROGRAM_MAIN_PACKAGE_BYTES, WECHAT_MINIPROGRAM_TOTAL_PACKAGE_BYTES,
};
pub use package_manifest::{
    canonical_manifest_sha256, validate_package_manifest, validate_sha256_hex,
    PackageManifestValidation, PACKAGE_MANIFEST_KIND, PACKAGE_MANIFEST_SCHEMA_VERSION,
};
pub use runtime_env::{
    deploy_dev_auth_bypass_enabled, deploy_environment_name, deploy_is_production_like_environment,
    deploy_use_dev_inline_auth_resolver,
};
pub use util::{normalize_pagination, pagination_offset};
pub use versioning::{SemanticVersion, MAXIMUM_IDENTIFIER_LENGTH, MAXIMUM_VERSION_LENGTH};
