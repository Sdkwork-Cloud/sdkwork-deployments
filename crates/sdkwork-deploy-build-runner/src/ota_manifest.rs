//! OTA distribution manifest generation for mobile and mini-program delivery
//! (REQ-2026-0002, deployment target `OTA`/`ENTERPRISE`).
//!
//! Generates the two standard OTA manifests from bounded package metadata:
//!
//! - iOS: an `itms-services` plist manifest that points an enterprise
//!   installation at the package download URL;
//! - Android: a JSON update manifest consumed by the in-app updater.
//!
//! The generators are pure logic with bounded inputs; download URLs are
//! validated as HTTPS and never contain credentials.

use sdkwork_deploy_core::SemanticVersion;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtaManifestInput {
    pub package_uuid: String,
    pub bundle_identity: String,
    pub semantic_version: String,
    pub build_number: i64,
    pub download_url: String,
    pub package_size_bytes: u64,
    pub minimum_version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OtaManifestError {
    #[error("ota manifest validation: {0}")]
    Validation(String),
}

fn validate_input(
    input: &OtaManifestInput,
    allow_minimum_android: bool,
) -> Result<(), OtaManifestError> {
    if input.package_uuid.is_empty() || input.package_uuid.len() > 128 {
        return Err(OtaManifestError::Validation(
            "packageUuid is invalid".into(),
        ));
    }
    if input.bundle_identity.is_empty() || input.bundle_identity.len() > 255 {
        return Err(OtaManifestError::Validation(
            "bundleIdentity is invalid".into(),
        ));
    }
    SemanticVersion::parse(&input.semantic_version)
        .map_err(|error| OtaManifestError::Validation(format!("semanticVersion: {error}")))?;
    if input.build_number <= 0 {
        return Err(OtaManifestError::Validation(
            "buildNumber must be positive".into(),
        ));
    }
    if !is_https_url(&input.download_url) {
        return Err(OtaManifestError::Validation(
            "downloadUrl must be an https URL without credentials".into(),
        ));
    }
    if input.package_size_bytes == 0 {
        return Err(OtaManifestError::Validation(
            "packageSizeBytes must be positive".into(),
        ));
    }
    if allow_minimum_android {
        if input.minimum_version.is_empty() || input.minimum_version.len() > 32 {
            return Err(OtaManifestError::Validation(
                "minimumVersion is invalid for Android manifests".into(),
            ));
        }
    }
    Ok(())
}

fn is_https_url(url: &str) -> bool {
    // https only; no userinfo credentials (reject any '@').
    url.starts_with("https://") && !url.contains('@')
}

/// Generates the iOS enterprise OTA `itms-services` plist manifest.
pub fn generate_ios_ota_plist(input: &OtaManifestInput) -> Result<String, OtaManifestError> {
    validate_input(input, false)?;
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>items</key>
    <array>
        <dict>
            <key>metadata</key>
            <dict>
                <key>bundle-identifier</key>
                <string>{bundle}</string>
                <key>bundle-version</key>
                <string>{version}</string>
                <key>kind</key>
                <string>software</string>
                <key>title</key>
                <string>{title}</string>
            </dict>
            <key>assets</key>
            <array>
                <dict>
                    <key>kind</key>
                    <string>software-package</string>
                    <key>url</key>
                    <string>{url}</string>
                </dict>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
        bundle = escape_plist(&input.bundle_identity),
        version = escape_plist(&input.semantic_version),
        title = escape_plist(&format!(
            "{}-{}",
            input.bundle_identity, input.semantic_version
        )),
        url = escape_plist(&input.download_url),
    ))
}

/// Generates the Android OTA update manifest consumed by the in-app updater.
pub fn generate_android_ota_json(
    input: &OtaManifestInput,
) -> Result<serde_json::Value, OtaManifestError> {
    validate_input(input, true)?;
    Ok(serde_json::json!({
        "schemaVersion": "sdkwork.ota.android.v1",
        "packageUuid": input.package_uuid,
        "packageName": input.bundle_identity,
        "version": input.semantic_version,
        "buildNumber": input.build_number,
        "minimumVersion": input.minimum_version,
        "downloadUrl": input.download_url,
        "packageSizeBytes": input.package_size_bytes,
        "checksumSha256": null,
    }))
}

fn escape_plist(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> OtaManifestInput {
        OtaManifestInput {
            package_uuid: "package-1".to_owned(),
            bundle_identity: "com.sdkwork.example".to_owned(),
            semantic_version: "2.1.0".to_owned(),
            build_number: 42,
            download_url: "https://cdn.sdkwork.com/packages/package-1.ipa".to_owned(),
            package_size_bytes: 4096,
            minimum_version: "24".to_owned(),
        }
    }

    #[test]
    fn generates_ios_ota_plist() {
        let plist = generate_ios_ota_plist(&input()).expect("plist");
        // The plist is the manifest referenced by an itms-services URL.
        assert!(plist.starts_with("<?xml"));
        assert!(plist.contains("<plist version="));
        assert!(plist.contains("software-package"));
        assert!(plist.contains("bundle-identifier"));
        assert!(plist.contains("com.sdkwork.example"));
        assert!(plist.contains("2.1.0"));
        assert!(plist.contains("https://cdn.sdkwork.com"));
    }

    #[test]
    fn generates_android_ota_json() {
        let manifest = generate_android_ota_json(&input()).expect("json");
        assert_eq!(manifest["version"], "2.1.0");
        assert_eq!(manifest["minimumVersion"], "24");
        assert_eq!(manifest["packageName"], "com.sdkwork.example");
    }

    #[test]
    fn rejects_insecure_and_credentialed_urls() {
        let mut bad = input();
        bad.download_url = "http://cdn.sdkwork.com/p.ipa".to_owned();
        assert!(generate_ios_ota_plist(&bad).is_err());
        bad.download_url = "https://user:pass@cdn.sdkwork.com/p.ipa".to_owned();
        assert!(generate_android_ota_json(&bad).is_err());
    }

    #[test]
    fn rejects_invalid_versions() {
        let mut bad = input();
        bad.semantic_version = "not-a-version".to_owned();
        assert!(generate_ios_ota_plist(&bad).is_err());
    }

    #[test]
    fn escapes_plist_special_characters() {
        assert_eq!(escape_plist("a<b&c"), "a&lt;b&amp;c");
    }
}
