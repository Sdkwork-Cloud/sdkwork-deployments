//! OTA distribution manifest generation for mobile, mini-program, and
//! desktop delivery (REQ-2026-0002, deployment target `OTA`/`ENTERPRISE`).
//!
//! Generates the standard OTA manifests from bounded package metadata:
//!
//! - iOS: an `itms-services` plist manifest that points an enterprise
//!   installation at the package download URL;
//! - Android: a JSON update manifest consumed by the in-app updater;
//! - Electron: the `latest.yml` manifest consumed by `electron-updater`;
//! - Tauri: the `latest.json` manifest consumed by `tauri-plugin-updater`;
//! - Sparkle: the `appcast.xml` manifest consumed by native macOS Sparkle.
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

/// Desktop auto-update manifest input. `checksum_sha512_base64` is the
/// base64-encoded SHA-512 of the installer bytes — the integrity evidence
/// Electron/Tauri updaters verify before applying an update; missing or
/// malformed checksums are rejected so unsigned descriptors never ship.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopOtaManifestInput {
    pub package_uuid: String,
    pub bundle_identity: String,
    pub semantic_version: String,
    pub build_number: i64,
    pub download_url: String,
    pub package_size_bytes: u64,
    pub package_format: String,
    pub cpu_arch: String,
    pub checksum_sha512_base64: String,
    pub release_notes: Option<String>,
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

fn validate_desktop_input(input: &DesktopOtaManifestInput) -> Result<(), OtaManifestError> {
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
    if input.package_format.is_empty() || input.package_format.len() > 16 {
        return Err(OtaManifestError::Validation(
            "packageFormat is invalid".into(),
        ));
    }
    if input.cpu_arch != "X86_64" && input.cpu_arch != "ARM64" {
        return Err(OtaManifestError::Validation(
            "cpuArch must be X86_64 or ARM64".into(),
        ));
    }
    // base64 of 64 raw bytes is 88 characters incl. padding.
    let checksum = input.checksum_sha512_base64.as_str();
    if checksum.len() != 88
        || !checksum
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
    {
        return Err(OtaManifestError::Validation(
            "checksumSha512 must be base64-encoded SHA-512 (88 characters)".into(),
        ));
    }
    if let Some(notes) = input.release_notes.as_deref() {
        if notes.len() > 4000 {
            return Err(OtaManifestError::Validation(
                "releaseNotes exceeds the 4000-character limit".into(),
            ));
        }
    }
    Ok(())
}

/// Maps a platform architecture to the Tauri/electron updater platform key.
fn updater_platform_key(cpu_arch: &str) -> &'static str {
    match cpu_arch {
        "ARM64" => "aarch64",
        _ => "x86_64",
    }
}

/// Generates the Electron `latest.yml` manifest consumed by electron-updater.
/// The file name is `latest.yml` (Windows/Linux) or `latest-mac.yml`
/// (macOS); the caller places it on the OTA endpoint next to the installer.
pub fn generate_electron_latest_yml(
    input: &DesktopOtaManifestInput,
) -> Result<String, OtaManifestError> {
    validate_desktop_input(input)?;
    let file_name = format!(
        "{}-{}-{}.{}",
        input.bundle_identity.replace(['@', '/', '\\'], "-"),
        input.semantic_version,
        input.build_number,
        installer_extension(&input.package_format)
    );
    Ok(format!(
        "version: {version}\n\
         files:\n\
         \x20 - url: {url}\n\
         \x20   sha512: {sha512}\n\
         \x20   size: {size}\n\
         path: {file_name}\n\
         sha512: {sha512}\n\
         releaseDate: '{date}'\n",
        version = input.semantic_version,
        url = input.download_url,
        sha512 = input.checksum_sha512_base64,
        size = input.package_size_bytes,
        file_name = file_name,
        date = release_date(),
    ))
}

/// Generates the Tauri v2 `latest.json` manifest consumed by
/// tauri-plugin-updater. The platform entry carries the installer URL and
/// size; `signature` is bound by the signing step before publication.
pub fn generate_tauri_latest_json(
    input: &DesktopOtaManifestInput,
) -> Result<serde_json::Value, OtaManifestError> {
    validate_desktop_input(input)?;
    let arch = updater_platform_key(&input.cpu_arch);
    let platform = match input.package_format.as_str() {
        "MSI" | "NSIS" | "MSIX" | "EXE" => format!("windows-{arch}"),
        "DMG" | "PKG" => format!("darwin-{arch}"),
        _ => format!("linux-{arch}"),
    };
    Ok(serde_json::json!({
        "version": input.semantic_version,
        "notes": input.release_notes.as_deref().unwrap_or(""),
        "pub_date": release_date(),
        "platforms": {
            platform: {
                "url": input.download_url,
                "size": input.package_size_bytes,
                "signature": null,
            }
        }
    }))
}

/// Generates the Sparkle `appcast.xml` manifest for native macOS auto-update.
pub fn generate_sparkle_appcast_xml(
    input: &DesktopOtaManifestInput,
) -> Result<String, OtaManifestError> {
    validate_desktop_input(input)?;
    let title = format!(
        "{} {version}",
        input.bundle_identity,
        version = input.semantic_version
    );
    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <title>{title}</title>
    <item>
      <title>{version}</title>
      <sparkle:version>{version}</sparkle:version>
      <sparkle:shortVersionString>{version}</sparkle:shortVersionString>
      <pubDate>{date}</pubDate>
      <enclosure url="{url}" sparkle:version="{version}" length="{size}" type="application/octet-stream"/>
      <sparkle:minimumSystemVersion>{minimum}</sparkle:minimumSystemVersion>
    </item>
  </channel>
</rss>
"#,
        title = escape_xml(&title),
        version = escape_xml(&input.semantic_version),
        url = escape_xml(&input.download_url),
        size = input.package_size_bytes,
        date = release_date(),
        minimum = escape_xml(&macos_minimum_version()),
    ))
}

fn installer_extension(package_format: &str) -> &'static str {
    match package_format {
        "MSI" => "msi",
        "MSIX" => "msix",
        "NSIS" => "exe",
        "EXE" => "exe",
        "DMG" => "dmg",
        "PKG" => "pkg",
        "DEB" => "deb",
        "RPM" => "rpm",
        "APPIMAGE" => "AppImage",
        _ => "bin",
    }
}

fn macos_minimum_version() -> &'static str {
    "10.15"
}

fn release_date() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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

    fn desktop_input() -> DesktopOtaManifestInput {
        DesktopOtaManifestInput {
            package_uuid: "package-2".to_owned(),
            bundle_identity: "com.sdkwork.desktop".to_owned(),
            semantic_version: "2.1.0".to_owned(),
            build_number: 42,
            download_url: "https://cdn.sdkwork.com/packages/desktop-setup.exe".to_owned(),
            package_size_bytes: 8192,
            package_format: "NSIS".to_owned(),
            cpu_arch: "X86_64".to_owned(),
            checksum_sha512_base64: "A".repeat(88),
            release_notes: Some("Fixed the thing".to_owned()),
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

    #[test]
    fn generates_electron_latest_yml() {
        let yml = generate_electron_latest_yml(&desktop_input()).expect("electron yml");
        assert!(yml.starts_with("version: 2.1.0"));
        assert!(yml.contains("sha512: "));
        assert!(yml.contains("path: com.sdkwork.desktop-2.1.0-42.exe"));
        assert!(yml.contains("releaseDate: '"));
    }

    #[test]
    fn generates_tauri_latest_json() {
        let manifest = generate_tauri_latest_json(&desktop_input()).expect("tauri json");
        assert_eq!(manifest["version"], "2.1.0");
        assert_eq!(manifest["notes"], "Fixed the thing");
        assert!(manifest["pub_date"].as_str().unwrap().contains('T'));
        let windows = manifest["platforms"]["windows-x86_64"].clone();
        assert_eq!(
            windows["url"],
            "https://cdn.sdkwork.com/packages/desktop-setup.exe"
        );
        assert_eq!(windows["size"], 8192);
        assert!(windows["signature"].is_null());

        let mut mac = desktop_input();
        mac.package_format = "DMG".to_owned();
        mac.cpu_arch = "ARM64".to_owned();
        let manifest = generate_tauri_latest_json(&mac).expect("tauri json mac");
        assert!(manifest["platforms"]["darwin-aarch64"].is_object());
    }

    #[test]
    fn generates_sparkle_appcast() {
        let xml = generate_sparkle_appcast_xml(&desktop_input()).expect("appcast");
        assert!(xml.contains("<rss version=\"2.0\""));
        assert!(xml.contains("sparkle:version>2.1.0"));
        assert!(xml.contains("<enclosure url=\"https://cdn.sdkwork.com"));
        assert!(xml.contains("length=\"8192\""));
    }

    #[test]
    fn desktop_manifest_inputs_fail_closed() {
        let mut bad = desktop_input();
        bad.checksum_sha512_base64 = "short".to_owned();
        assert!(generate_electron_latest_yml(&bad).is_err());
        bad = desktop_input();
        bad.cpu_arch = "MIPS".to_owned();
        assert!(generate_tauri_latest_json(&bad).is_err());
        bad = desktop_input();
        bad.download_url = "http://cdn.sdkwork.com/x.exe".to_owned();
        assert!(generate_sparkle_appcast_xml(&bad).is_err());
        bad = desktop_input();
        bad.release_notes = Some("x".repeat(4001));
        assert!(generate_electron_latest_yml(&bad).is_err());
        // Escaping: bundle identity with XML-special characters is escaped.
        let mut special = desktop_input();
        special.bundle_identity = "com.sdkwork.a<b".to_owned();
        let xml = generate_sparkle_appcast_xml(&special).expect("appcast escaped");
        assert!(xml.contains("a&lt;b"));
    }
}
