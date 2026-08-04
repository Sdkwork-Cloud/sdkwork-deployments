//! Deployment package standard `sdkwork.deploy-package.v1` manifest validation
//! and canonical hashing (REQ-2026-0002 requirement 8).

use sdkwork_utils_rust::sha256_hash;
use serde_json::{Map, Value};

use crate::versioning::SemanticVersion;

pub const PACKAGE_MANIFEST_SCHEMA_VERSION: &str = "sdkwork.deploy-package.v1";
pub const PACKAGE_MANIFEST_KIND: &str = "sdkwork.deploy-package.manifest";
pub const MAXIMUM_MANIFEST_BYTES: usize = 64 * 1024;
pub const MANIFEST_HASH_FIELD: &str = "manifestSha256";

const REQUIRED_FIELDS: &[&str] = &[
    "schemaVersion",
    "kind",
    "packageUuid",
    "appUuid",
    "platformTargetUuid",
    "platform",
    "packageFormat",
    "semanticVersion",
    "buildNumber",
    "buildUuid",
    "sourceCommit",
    "artifactHashSha256",
    "packageSizeBytes",
];

/// Result of validating a package manifest document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageManifestValidation {
    /// Canonical digest over the manifest excluding the `manifestSha256` field.
    pub manifest_sha256: String,
    /// The validated semantic version from the manifest.
    pub semantic_version: String,
}

/// Validates a parsed manifest and returns the canonical manifest digest.
pub fn validate_package_manifest(manifest: &Value) -> Result<PackageManifestValidation, String> {
    let bytes = serde_json::to_vec(manifest)
        .map_err(|error| format!("serialize package manifest: {error}"))?;
    if bytes.is_empty() || bytes.len() > MAXIMUM_MANIFEST_BYTES {
        return Err(format!(
            "package manifest exceeds the {MAXIMUM_MANIFEST_BYTES}-byte limit"
        ));
    }
    let object = manifest
        .as_object()
        .ok_or_else(|| "package manifest must be a JSON object".to_owned())?;
    for field in REQUIRED_FIELDS {
        if object.get(*field).is_none() {
            return Err(format!(
                "package manifest is missing required field {field}"
            ));
        }
    }
    require_string(object, "schemaVersion", PACKAGE_MANIFEST_SCHEMA_VERSION)?;
    require_string(object, "kind", PACKAGE_MANIFEST_KIND)?;
    validate_opaque_id(required_string(object, "packageUuid")?, "packageUuid")?;
    validate_opaque_id(required_string(object, "appUuid")?, "appUuid")?;
    validate_opaque_id(
        required_string(object, "platformTargetUuid")?,
        "platformTargetUuid",
    )?;
    validate_opaque_id(required_string(object, "buildUuid")?, "buildUuid")?;

    let semantic_version = required_string(object, "semanticVersion")?;
    let parsed = SemanticVersion::parse(semantic_version)
        .map_err(|error| format!("semanticVersion: {error}"))?;

    let artifact_hash = required_string(object, "artifactHashSha256")?;
    validate_sha256_hex(artifact_hash, "artifactHashSha256")?;
    let build_number = object
        .get("buildNumber")
        .and_then(Value::as_u64)
        .ok_or_else(|| "package manifest buildNumber must be a positive integer".to_owned())?;
    if build_number == 0 {
        return Err("package manifest buildNumber must be positive".into());
    }
    let package_size = object
        .get("packageSizeBytes")
        .and_then(Value::as_u64)
        .ok_or_else(|| "package manifest packageSizeBytes must be a positive integer".to_owned())?;
    if package_size == 0 {
        return Err("package manifest packageSizeBytes must be positive".into());
    }

    // Optional embedded digest must agree with the canonical hash when present.
    if let Some(embedded) = object.get(MANIFEST_HASH_FIELD).and_then(Value::as_str) {
        validate_sha256_hex(embedded, MANIFEST_HASH_FIELD)?;
        let computed = canonical_manifest_sha256(manifest)?;
        if embedded != computed {
            return Err(
                "package manifest embedded manifestSha256 does not match canonical content".into(),
            );
        }
    }

    Ok(PackageManifestValidation {
        manifest_sha256: canonical_manifest_sha256(manifest)?,
        semantic_version: parsed.to_canonical_string(),
    })
}

/// Canonical SHA-256 of the manifest document excluding the
/// `manifestSha256` field, with recursively ordered object keys.
pub fn canonical_manifest_sha256(manifest: &Value) -> Result<String, String> {
    let mut value = manifest.clone();
    value
        .as_object_mut()
        .ok_or_else(|| "package manifest must be a JSON object".to_owned())?
        .remove(MANIFEST_HASH_FIELD);
    let mut canonical = String::new();
    write_canonical_json(&value, &mut canonical)?;
    Ok(sha256_hash(canonical.as_bytes()))
}

fn required_string<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("package manifest field {field} must be a string"))
}

fn require_string(object: &Map<String, Value>, field: &str, expected: &str) -> Result<(), String> {
    let actual = required_string(object, field)?;
    if actual != expected {
        return Err(format!(
            "package manifest field {field} must be {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn validate_opaque_id(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'-' | b'_' | b'.' | b':'))
        })
    {
        return Err(format!(
            "package manifest {field} is not a bounded opaque identifier"
        ));
    }
    Ok(())
}

/// Validates a lowercase SHA-256 digest string (shared by manifest and
/// registration validation).
pub fn validate_sha256_hex(value: &str, field: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "package manifest {field} must be a lowercase SHA-256 digest"
        ));
    }
    Ok(())
}

fn write_canonical_json(value: &Value, output: &mut String) -> Result<(), String> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => {
            output.push_str(&serde_json::to_string(value).map_err(|error| error.to_string())?)
        }
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(']');
        }
        Value::Object(values) => {
            output.push('{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key).map_err(|error| error.to_string())?);
                output.push(':');
                write_canonical_json(value, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> Value {
        serde_json::json!({
            "schemaVersion": "sdkwork.deploy-package.v1",
            "kind": "sdkwork.deploy-package.manifest",
            "packageUuid": "package-1",
            "appUuid": "app-1",
            "platformTargetUuid": "target-1",
            "platform": "ANDROID",
            "packageFormat": "APK",
            "semanticVersion": "1.4.2",
            "buildNumber": 117,
            "buildUuid": "build-1",
            "sourceCommit": "0123456789abcdef0123456789abcdef01234567",
            "sourceRef": "refs/tags/v1.4.2",
            "toolchainVersion": "flutter/3.24.3",
            "artifactHashSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "packageSizeBytes": 42,
            "architectures": ["arm64-v8a"]
        })
    }

    #[test]
    fn validates_and_hashes_manifest() {
        let result = validate_package_manifest(&valid_manifest()).expect("valid manifest");
        assert_eq!(result.semantic_version, "1.4.2");
        assert_eq!(result.manifest_sha256.len(), 64);
        // The digest is deterministic for identical canonical content.
        let again = validate_package_manifest(&valid_manifest()).expect("valid manifest");
        assert_eq!(result.manifest_sha256, again.manifest_sha256);
    }

    #[test]
    fn embedded_digest_must_agree() {
        let mut manifest = valid_manifest();
        let result = validate_package_manifest(&manifest).expect("valid manifest");
        let digest = result.manifest_sha256.clone();
        manifest.as_object_mut().unwrap().insert(
            MANIFEST_HASH_FIELD.to_owned(),
            Value::String(result.manifest_sha256),
        );
        let verified = validate_package_manifest(&manifest).expect("embedded digest agrees");
        assert_eq!(verified.manifest_sha256, digest);

        manifest.as_object_mut().unwrap().insert(
            MANIFEST_HASH_FIELD.to_owned(),
            Value::String("0".repeat(64)),
        );
        assert!(validate_package_manifest(&manifest).is_err());
    }

    #[test]
    fn rejects_missing_and_invalid_fields() {
        for field in REQUIRED_FIELDS {
            let mut manifest = valid_manifest();
            manifest.as_object_mut().unwrap().remove(*field);
            assert!(
                validate_package_manifest(&manifest).is_err(),
                "should reject missing {field}"
            );
        }
        let mut manifest = valid_manifest();
        manifest.as_object_mut().unwrap().insert(
            "semanticVersion".to_owned(),
            Value::String("not-a-version".to_owned()),
        );
        assert!(validate_package_manifest(&manifest).is_err());
    }
}
