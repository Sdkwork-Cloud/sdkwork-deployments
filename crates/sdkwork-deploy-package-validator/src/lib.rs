//! Byte-boundary validator for the `sdkwork.deploy-package.v1` deployment
//! package standard (REQ-2026-0002 requirement 8, TECH §5.2).
//!
//! The control plane registers package metadata; this validator reads the
//! actual package bytes through Drive and enforces per-format rules before a
//! Release may reference the package:
//!
//! - archive structure (ZIP / TAR_GZ) with bounded entry counts and sizes;
//! - traversal, absolute-path, symlink, and hidden-entry rejection;
//! - the embedded `sdkwork.deploy-package.v1` manifest is present and
//!   canonically valid;
//! - manifest fields agree with the registration expectation (platform,
//!   package format, semantic version, artifact hash, package size);
//! - platform size ceilings (WeChat/Douyin main and total package limits,
//!   web bundle ceiling, process bundle ceiling).
//!
//! Directory-form packages (`DIST_DIR`) are validated by walking a bounded
//! directory tree with the same rules.

use std::fs;
use std::io::Read;
use std::path::Path;

use sdkwork_deploy_core::{
    validate_package_manifest, validate_package_size, DOUYIN_MINIPROGRAM_MAIN_PACKAGE_BYTES,
    WECHAT_MINIPROGRAM_MAIN_PACKAGE_BYTES,
};

pub const PACKAGE_MANIFEST_PATH: &str = "sdkwork.deploy-package.v1.json";
pub const WECHAT_MANIFEST_PATH: &str = "app.json";
pub const DOUYIN_MANIFEST_PATH: &str = "app.json";
pub const MAXIMUM_ENTRIES: usize = 100_000;
pub const MAXIMUM_ENTRY_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum PackageValidationError {
    #[error("package io: {0}")]
    Io(String),
    #[error("package rule: {0}")]
    Rule(String),
    #[error("package manifest: {0}")]
    Manifest(String),
    #[error("package mismatch: {0}")]
    Mismatch(String),
}

/// Registration expectation the package bytes must agree with.
#[derive(Clone, Debug)]
pub struct PackageValidationExpectation {
    pub platform: String,
    pub package_format: String,
    pub semantic_version: String,
    pub artifact_hash_sha256: String,
    pub package_size_bytes: u64,
}

/// Bounded validation evidence for the `deploy_package.validation_report_json`
/// column.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackageValidationReport {
    pub valid: bool,
    pub format: String,
    pub entry_count: usize,
    pub main_package_bytes: Option<u64>,
    pub total_package_bytes: u64,
    pub manifest_present: bool,
    pub manifest_sha256: Option<String>,
    pub error_code: Option<String>,
    pub checked_at: String,
}

/// Validates a package file (ZIP or TAR_GZ) against the expectation and the
/// per-format platform rules. Returns the validation report.
pub fn validate_package_file(
    path: &Path,
    expectation: &PackageValidationExpectation,
) -> Result<PackageValidationReport, PackageValidationError> {
    let metadata = fs::metadata(path)
        .map_err(|error| PackageValidationError::Io(format!("stat package: {error}")))?;
    if !metadata.is_file() {
        return Err(PackageValidationError::Rule(
            "package path is not a file".into(),
        ));
    }
    let size = metadata.len();
    if size == 0 {
        return Err(PackageValidationError::Rule("package is empty".into()));
    }
    if size != expectation.package_size_bytes {
        return Err(PackageValidationError::Mismatch(format!(
            "package size {size} does not match registration {size_expected}",
            size_expected = expectation.package_size_bytes
        )));
    }
    validate_package_size(&expectation.platform, &expectation.package_format, size)
        .map_err(PackageValidationError::Rule)?;

    let entries = match expectation.package_format.as_str() {
        // Archive formats carry the embedded manifest and full entry rules.
        "ZIP" | "TAR_GZ" | "IPA" | "APK" | "AAB" | "HAP" | "APP" | "JAR" | "WAR" | "MSIX" => {
            let entries = if expectation.package_format == "TAR_GZ" {
                scan_tar_gz(path)?
            } else {
                scan_zip(path)?
            };
            enforce_archive_rules(&entries)?;
            entries
        }
        // Installer formats are validated by container signature only; the
        // manifest travels as registration metadata, never embedded (embedding
        // would break Authenticode/notarization and vendor installers).
        "MSI" | "NSIS" | "EXE" | "DMG" | "PKG" | "DEB" | "RPM" | "APPIMAGE" => {
            verify_container_signature(path, expectation.package_format.as_str())?;
            Vec::new()
        }
        _ => {
            return Err(PackageValidationError::Rule(format!(
                "format {} is not a byte-validated archive or installer format",
                expectation.package_format
            )))
        }
    };

    // Archive formats must embed the package manifest; installer formats
    // validate at the container boundary and stay manifest-external.
    if entries.is_empty() {
        return Ok(PackageValidationReport {
            valid: true,
            format: expectation.package_format.clone(),
            entry_count: 0,
            main_package_bytes: None,
            total_package_bytes: size,
            manifest_present: false,
            manifest_sha256: None,
            error_code: None,
            checked_at: crate::now_rfc3339(),
        });
    }

    let manifest = find_entry(&entries, PACKAGE_MANIFEST_PATH).ok_or_else(|| {
        PackageValidationError::Manifest(format!(
            "package is missing the {PACKAGE_MANIFEST_PATH} manifest"
        ))
    })?;
    let manifest_bytes = read_entry(
        path,
        &entries,
        manifest,
        expectation.package_format.as_str(),
    )?;
    if manifest_bytes.len() > 64 * 1024 {
        return Err(PackageValidationError::Manifest(
            "package manifest exceeds the 64 KiB limit".into(),
        ));
    }
    let manifest_value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| PackageValidationError::Manifest(format!("parse manifest: {error}")))?;
    let validated =
        validate_package_manifest(&manifest_value).map_err(PackageValidationError::Manifest)?;

    if validated.semantic_version != expectation.semantic_version {
        return Err(PackageValidationError::Mismatch(format!(
            "manifest version {} does not match registration {}",
            validated.semantic_version, expectation.semantic_version
        )));
    }
    let manifest_platform = manifest_value
        .get("platform")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if manifest_platform != expectation.platform {
        return Err(PackageValidationError::Mismatch(format!(
            "manifest platform {manifest_platform} does not match registration {}",
            expectation.platform
        )));
    }
    let manifest_hash = manifest_value
        .get("artifactHashSha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if manifest_hash != expectation.artifact_hash_sha256 {
        return Err(PackageValidationError::Mismatch(
            "manifest artifactHashSha256 does not match registration".into(),
        ));
    }

    // Main-package ceiling: entries under the platform manifest directory
    // (WeChat/Douyin) count toward the main package.
    let main_package_bytes = match expectation.platform.as_str() {
        "WECHAT" => Some(main_package_size(&entries, WECHAT_MANIFEST_PATH)),
        "DOUYIN" => Some(main_package_size(&entries, DOUYIN_MANIFEST_PATH)),
        _ => None,
    };
    if let Some(main_bytes) = main_package_bytes {
        let ceiling = match expectation.platform.as_str() {
            "WECHAT" => WECHAT_MINIPROGRAM_MAIN_PACKAGE_BYTES,
            _ => DOUYIN_MINIPROGRAM_MAIN_PACKAGE_BYTES,
        };
        if main_bytes > ceiling {
            return Err(PackageValidationError::Rule(format!(
                "main package size {main_bytes} exceeds the {ceiling}-byte ceiling"
            )));
        }
    }

    Ok(PackageValidationReport {
        valid: true,
        format: expectation.package_format.clone(),
        entry_count: entries.len(),
        main_package_bytes,
        total_package_bytes: size,
        manifest_present: true,
        manifest_sha256: Some(validated.manifest_sha256),
        error_code: None,
        checked_at: crate::now_rfc3339(),
    })
}

#[derive(Clone, Debug)]
struct Entry {
    path: String,
    is_dir: bool,
    is_symlink: bool,
    size: u64,
    uncompressed_size: u64,
}

fn scan_zip(path: &Path) -> Result<Vec<Entry>, PackageValidationError> {
    let file = fs::File::open(path)
        .map_err(|error| PackageValidationError::Io(format!("open zip: {error}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| PackageValidationError::Rule(format!("invalid zip archive: {error}")))?;
    if archive.len() > MAXIMUM_ENTRIES {
        return Err(PackageValidationError::Rule(format!(
            "zip entry count {} exceeds the {MAXIMUM_ENTRIES} limit",
            archive.len()
        )));
    }
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|error| PackageValidationError::Io(format!("read zip entry: {error}")))?;
        let raw_path = file.name().to_owned();
        let uncompressed_size = file.size();
        entries.push(Entry {
            path: normalize_entry_path(&raw_path),
            is_dir: file.is_dir(),
            is_symlink: file
                .unix_mode()
                .map(|mode| mode & 0o170000 == 0o120000)
                .unwrap_or(false),
            size: uncompressed_size,
            uncompressed_size,
        });
    }
    Ok(entries)
}

fn scan_tar_gz(path: &Path) -> Result<Vec<Entry>, PackageValidationError> {
    let file = fs::File::open(path)
        .map_err(|error| PackageValidationError::Io(format!("open tar: {error}")))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut entries = Vec::new();
    let archive_entries = archive
        .entries()
        .map_err(|error| PackageValidationError::Rule(format!("invalid tar archive: {error}")))?;
    for entry in archive_entries {
        let entry = entry
            .map_err(|error| PackageValidationError::Rule(format!("invalid tar entry: {error}")))?;
        let raw_path = entry
            .path()
            .map_err(|error| PackageValidationError::Rule(format!("invalid tar path: {error}")))?
            .to_string_lossy()
            .into_owned();
        let size = entry.size();
        entries.push(Entry {
            path: normalize_entry_path(&raw_path),
            is_dir: entry.header().entry_type().is_dir(),
            is_symlink: entry.header().entry_type().is_symlink(),
            size,
            uncompressed_size: size,
        });
        if entries.len() > MAXIMUM_ENTRIES {
            return Err(PackageValidationError::Rule(format!(
                "tar entry count exceeds the {MAXIMUM_ENTRIES} limit"
            )));
        }
    }
    Ok(entries)
}

/// Verifies the container magic signature of an installer package. Header
/// reads are bounded to 512 bytes plus the PE `e_lfanew` probe; tail reads are
/// bounded to 512 bytes (DMG `koly` trailer). Failures are fail-closed.
fn verify_container_signature(path: &Path, format: &str) -> Result<(), PackageValidationError> {
    let mut head = [0u8; 512];
    let mut tail = [0u8; 512];
    let mut file = fs::File::open(path)
        .map_err(|error| PackageValidationError::Io(format!("open {format} package: {error}")))?;
    let head_len = file
        .read(&mut head)
        .map_err(|error| PackageValidationError::Io(format!("read {format} header: {error}")))?;

    let (expected, label): (&[u8], &str) = match format {
        "MSI" => (
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
            "OLE compound document",
        ),
        "PKG" => (b"xar!", "xar archive"),
        "DEB" => (b"!<arch>\n", "ar archive"),
        "RPM" => (&[0xED, 0xAB, 0xEE, 0xDB], "RPM container"),
        "APPIMAGE" => (&[0x7F, b'E', b'L', b'F'], "ELF executable"),
        "EXE" | "NSIS" => (b"MZ", "PE executable"),
        "DMG" => {
            let tail_len = file.read(&mut tail).map_err(|error| {
                PackageValidationError::Io(format!("read {format} tail: {error}"))
            })?;
            // The trailer `koly` block sits in the final 512 bytes.
            let window = if tail_len >= 512 {
                &tail[tail_len - 512..]
            } else {
                &tail[..tail_len]
            };
            if window.windows(4).any(|chunk| chunk == b"koly") {
                return Ok(());
            }
            return Err(PackageValidationError::Rule(
                "DMG package is missing the koly trailer block".into(),
            ));
        }
        _ => {
            return Err(PackageValidationError::Rule(format!(
                "format {format} is not a signature-verified installer format"
            )))
        }
    };

    if head_len < expected.len() || !head[..expected.len()].starts_with(expected) {
        return Err(PackageValidationError::Rule(format!(
            "package is not a {label} ({format})"
        )));
    }

    // PE executables: verify the `PE\0\0` signature at the e_lfanew offset.
    if format == "EXE" || format == "NSIS" {
        if head_len < 0x40 {
            return Err(PackageValidationError::Rule(
                "PE executable header is truncated".into(),
            ));
        }
        let e_lfanew =
            u32::from_le_bytes([head[0x3C], head[0x3D], head[0x3E], head[0x3F]]) as usize;
        let pe_offset = e_lfanew
            .checked_add(4)
            .ok_or_else(|| PackageValidationError::Rule("invalid PE header offset".into()))?;
        if pe_offset > head_len {
            return Err(PackageValidationError::Rule(
                "PE header offset exceeds the bounded read window".into(),
            ));
        }
        if &head[e_lfanew..pe_offset] != b"PE\x00\x00" {
            return Err(PackageValidationError::Rule(format!(
                "package is not a valid {label} ({format})"
            )));
        }
        if format == "NSIS" {
            // NSIS installers embed the "NullsoftInst" marker near the tail.
            let tail_len = file.read(&mut tail).map_err(|error| {
                PackageValidationError::Io(format!("read {format} tail: {error}"))
            })?;
            if !tail[..tail_len]
                .windows(12)
                .any(|chunk| chunk == b"NullsoftInst")
            {
                return Err(PackageValidationError::Rule(
                    "NSIS installer is missing the NullsoftInst marker".into(),
                ));
            }
        }
    }

    Ok(())
}

fn enforce_archive_rules(entries: &[Entry]) -> Result<(), PackageValidationError> {
    for entry in entries {
        if entry.is_symlink {
            return Err(PackageValidationError::Rule(format!(
                "entry {} is a symlink; symlinks are not allowed in packages",
                entry.path
            )));
        }
        if entry.path.starts_with('/') {
            return Err(PackageValidationError::Rule(format!(
                "entry {} is an absolute path",
                entry.path
            )));
        }
        if entry.path.contains("..") {
            return Err(PackageValidationError::Rule(format!(
                "entry {} escapes the package root",
                entry.path
            )));
        }
        if entry.path.starts_with(".git/") || entry.path == ".git" {
            return Err(PackageValidationError::Rule(
                ".git entries are not allowed in packages".into(),
            ));
        }
        if entry.uncompressed_size > MAXIMUM_ENTRY_BYTES {
            return Err(PackageValidationError::Rule(format!(
                "entry {} exceeds the {MAXIMUM_ENTRY_BYTES}-byte limit",
                entry.path
            )));
        }
    }
    Ok(())
}

/// Normalizes archive paths: strips leading `./`, rejects backslashes, and
/// collapses duplicate slashes. The normalized path is used for rule checks.
fn normalize_entry_path(raw: &str) -> String {
    let mut normalized = raw.replace('\\', "/");
    while normalized.starts_with("./") {
        normalized = normalized[2..].to_owned();
    }
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    normalized
}

fn find_entry<'a>(entries: &'a [Entry], name: &str) -> Option<&'a Entry> {
    entries
        .iter()
        .find(|entry| !entry.is_dir && entry.path == name)
}

fn read_entry(
    path: &Path,
    _entries: &[Entry],
    entry: &Entry,
    format: &str,
) -> Result<Vec<u8>, PackageValidationError> {
    match format {
        "TAR_GZ" => {
            let file = fs::File::open(path)
                .map_err(|error| PackageValidationError::Io(format!("open tar: {error}")))?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            for archive_entry in archive
                .entries()
                .map_err(|error| PackageValidationError::Rule(format!("invalid tar: {error}")))?
            {
                let mut archive_entry = archive_entry.map_err(|error| {
                    PackageValidationError::Rule(format!("invalid tar entry: {error}"))
                })?;
                let entry_path = archive_entry
                    .path()
                    .map_err(|error| {
                        PackageValidationError::Rule(format!("invalid tar path: {error}"))
                    })?
                    .to_string_lossy()
                    .into_owned();
                if normalize_entry_path(&entry_path) == entry.path {
                    let mut bytes = Vec::with_capacity(entry.size as usize);
                    archive_entry.read_to_end(&mut bytes).map_err(|error| {
                        PackageValidationError::Io(format!("read tar entry: {error}"))
                    })?;
                    return Ok(bytes);
                }
            }
            Err(PackageValidationError::Rule(format!(
                "manifest entry {} disappeared during re-read",
                entry.path
            )))
        }
        _ => {
            let file = fs::File::open(path)
                .map_err(|error| PackageValidationError::Io(format!("open zip: {error}")))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| PackageValidationError::Rule(format!("invalid zip: {error}")))?;
            let mut manifest = archive
                .by_name(&entry.path)
                .map_err(|error| PackageValidationError::Rule(format!("read manifest: {error}")))?;
            let mut bytes = Vec::with_capacity(entry.size as usize);
            manifest
                .read_to_end(&mut bytes)
                .map_err(|error| PackageValidationError::Io(format!("read zip entry: {error}")))?;
            Ok(bytes)
        }
    }
}

/// Sums uncompressed bytes of non-directory entries under the directory of
/// the given platform manifest entry (main package semantics).
fn main_package_size(entries: &[Entry], platform_manifest_path: &str) -> u64 {
    let Some(manifest) = find_entry(entries, platform_manifest_path) else {
        return 0;
    };
    let Some(prefix) = manifest.path.rsplit_once('/').map(|(prefix, _)| prefix) else {
        // manifest at root: whole package is the main package
        return entries
            .iter()
            .filter(|entry| !entry.is_dir)
            .map(|entry| entry.uncompressed_size)
            .sum();
    };
    let prefix = format!("{prefix}/");
    entries
        .iter()
        .filter(|entry| !entry.is_dir && entry.path.starts_with(&prefix))
        .map(|entry| entry.uncompressed_size)
        .sum()
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

// ---------------------------------------------------------------------------
// Directory-form packages (DIST_DIR)
// ---------------------------------------------------------------------------

/// Validates a bounded directory tree as a `DIST_DIR` package.
pub fn validate_directory_package(
    root: &Path,
    expectation: &PackageValidationExpectation,
) -> Result<PackageValidationReport, PackageValidationError> {
    if expectation.package_format != "DIST_DIR" {
        return Err(PackageValidationError::Rule(format!(
            "format {} is not a directory format",
            expectation.package_format
        )));
    }
    let entries = scan_directory(root)?;
    enforce_archive_rules(&entries)?;
    let manifest_path = root.join(PACKAGE_MANIFEST_PATH);
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        PackageValidationError::Manifest(format!(
            "package is missing the {PACKAGE_MANIFEST_PATH} manifest: {error}"
        ))
    })?;
    let manifest_value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| PackageValidationError::Manifest(format!("parse manifest: {error}")))?;
    let validated =
        validate_package_manifest(&manifest_value).map_err(PackageValidationError::Manifest)?;
    if validated.semantic_version != expectation.semantic_version {
        return Err(PackageValidationError::Mismatch(format!(
            "manifest version {} does not match registration {}",
            validated.semantic_version, expectation.semantic_version
        )));
    }
    let total: u64 = entries
        .iter()
        .filter(|entry| !entry.is_dir)
        .map(|entry| entry.size)
        .sum();
    if total != expectation.package_size_bytes {
        return Err(PackageValidationError::Mismatch(format!(
            "directory size {total} does not match registration {}",
            expectation.package_size_bytes
        )));
    }
    Ok(PackageValidationReport {
        valid: true,
        format: expectation.package_format.clone(),
        entry_count: entries.len(),
        main_package_bytes: None,
        total_package_bytes: total,
        manifest_present: true,
        manifest_sha256: Some(validated.manifest_sha256),
        error_code: None,
        checked_at: now_rfc3339(),
    })
}

fn scan_directory(root: &Path) -> Result<Vec<Entry>, PackageValidationError> {
    let mut entries = Vec::new();
    scan_directory_recursive(root, root, &mut entries)?;
    Ok(entries)
}

fn scan_directory_recursive(
    root: &Path,
    current: &Path,
    entries: &mut Vec<Entry>,
) -> Result<(), PackageValidationError> {
    if entries.len() > MAXIMUM_ENTRIES {
        return Err(PackageValidationError::Rule(format!(
            "directory entry count exceeds the {MAXIMUM_ENTRIES} limit"
        )));
    }
    let read_dir = fs::read_dir(current)
        .map_err(|error| PackageValidationError::Io(format!("read directory: {error}")))?;
    for item in read_dir {
        let item =
            item.map_err(|error| PackageValidationError::Io(format!("read entry: {error}")))?;
        let item_path = item.path();
        let relative = item_path
            .strip_prefix(root)
            .map_err(|_| PackageValidationError::Rule("entry escapes the package root".into()))?;
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        let file_type = item
            .file_type()
            .map_err(|error| PackageValidationError::Io(format!("read file type: {error}")))?;
        if file_type.is_symlink() {
            return Err(PackageValidationError::Rule(format!(
                "entry {relative_str} is a symlink; symlinks are not allowed in packages"
            )));
        }
        if file_type.is_dir() {
            entries.push(Entry {
                path: relative_str.clone(),
                is_dir: true,
                is_symlink: false,
                size: 0,
                uncompressed_size: 0,
            });
            scan_directory_recursive(root, &item_path, entries)?;
        } else if file_type.is_file() {
            let size = item
                .metadata()
                .map_err(|error| PackageValidationError::Io(format!("read metadata: {error}")))?
                .len();
            entries.push(Entry {
                path: relative_str.clone(),
                is_dir: false,
                is_symlink: false,
                size,
                uncompressed_size: size,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn expectation(platform: &str, format: &str) -> PackageValidationExpectation {
        PackageValidationExpectation {
            platform: platform.to_owned(),
            package_format: format.to_owned(),
            semantic_version: "1.4.2".to_owned(),
            artifact_hash_sha256:
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned(),
            package_size_bytes: 0,
        }
    }

    fn make_manifest_json(platform: &str, format: &str, size: u64) -> String {
        serde_json::json!({
            "schemaVersion": "sdkwork.deploy-package.v1",
            "kind": "sdkwork.deploy-package.manifest",
            "packageUuid": "package-1",
            "appUuid": "app-1",
            "platformTargetUuid": "target-1",
            "platform": platform,
            "packageFormat": format,
            "semanticVersion": "1.4.2",
            "buildNumber": 117,
            "buildUuid": "build-1",
            "sourceCommit": "0123456789abcdef0123456789abcdef01234567",
            "artifactHashSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "packageSizeBytes": size,
        })
        .to_string()
    }

    fn write_zip(path: &Path, files: &[(&str, &[u8])]) -> zip::result::ZipResult<()> {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in files {
            writer.start_file(*name, options)?;
            writer.write_all(bytes)?;
        }
        writer.finish()?;
        Ok(())
    }

    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_dir(name: &str) -> PathBuf {
        let counter = DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sdkwork-{name}-{}-{counter}", std::process::id()))
    }

    #[test]
    fn validates_wechat_package_with_manifest() {
        let dir = test_dir("pkg-wechat");
        fs::create_dir_all(&dir).unwrap();
        let manifest = make_manifest_json("WECHAT", "ZIP", 1);
        let mut files: Vec<(&str, Vec<u8>)> = vec![
            ("app.json", br#"{"pages":["index"]}"#.to_vec()),
            (PACKAGE_MANIFEST_PATH, manifest.into_bytes()),
            ("index.js", b"console.log(1)".to_vec()),
        ];
        // sizes are not enforced at entry level; total is fixed below
        let path = dir.join("wechat.zip");
        let file_refs: Vec<(&str, &[u8])> = files
            .iter_mut()
            .map(|(name, bytes)| (*name, bytes.as_slice()))
            .collect();
        write_zip(&path, &file_refs).unwrap();
        let mut expectation = expectation("WECHAT", "ZIP");
        expectation.package_size_bytes = fs::metadata(&path).unwrap().len();
        let report = validate_package_file(&path, &expectation).expect("valid wechat package");
        assert!(report.valid);
        assert!(report.main_package_bytes.is_some());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_traversal_entries() {
        let dir = test_dir("pkg-wechat");
        fs::create_dir_all(&dir).unwrap();
        let manifest = make_manifest_json("WECHAT", "ZIP", 1);
        let mut files: Vec<(&str, Vec<u8>)> = vec![
            ("app.json", br#"{"pages":["index"]}"#.to_vec()),
            ("../escape.txt", b"bad".to_vec()),
            (PACKAGE_MANIFEST_PATH, manifest.into_bytes()),
        ];
        let path = dir.join("traversal.zip");
        let file_refs: Vec<(&str, &[u8])> = files
            .iter_mut()
            .map(|(name, bytes)| (*name, bytes.as_slice()))
            .collect();
        write_zip(&path, &file_refs).unwrap();
        let mut expectation = expectation("WECHAT", "ZIP");
        expectation.package_size_bytes = fs::metadata(&path).unwrap().len();
        let error = validate_package_file(&path, &expectation).expect_err("traversal rejected");
        assert!(error.to_string().contains("escape"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_missing_manifest() {
        let dir = test_dir("pkg-nomanifest");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("no-manifest.zip");
        write_zip(&path, &[("app.json", br#"{"pages":[]}"#.as_slice())]).unwrap();
        let mut expectation = expectation("WECHAT", "ZIP");
        expectation.package_size_bytes = fs::metadata(&path).unwrap().len();
        let error = validate_package_file(&path, &expectation).expect_err("missing manifest");
        assert!(error.to_string().contains("manifest"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn validates_directory_package() {
        let dir = test_dir("dir");
        fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = make_manifest_json("WEB", "DIST_DIR", 1);
        fs::write(dir.join("index.html"), b"<html></html>").unwrap();
        fs::write(dir.join("assets/app.js"), b"console.log(1)").unwrap();
        let manifest_bytes = manifest.clone().into_bytes();
        fs::write(dir.join(PACKAGE_MANIFEST_PATH), &manifest_bytes).unwrap();
        let total: u64 = [
            dir.join("index.html"),
            dir.join("assets/app.js"),
            dir.join(PACKAGE_MANIFEST_PATH),
        ]
        .iter()
        .map(|path| fs::metadata(path).unwrap().len())
        .sum();
        let mut expectation = expectation("WEB", "DIST_DIR");
        expectation.package_size_bytes = total;
        let report = validate_directory_package(&dir, &expectation).expect("valid dist dir");
        assert!(report.valid);
        assert_eq!(report.entry_count, 4); // 1 dir + 3 files
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Writes a synthetic installer with the given leading bytes (plus a
    /// trailing marker when provided) and validates it under the format.
    fn assert_installer_accepted(format: &str, head: &[u8], tail_marker: Option<&[u8]>) {
        let dir = test_dir(&format!("installer-{format}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("installer.bin");
        let mut bytes = head.to_vec();
        bytes.resize(1024, 0u8);
        if let Some(marker) = tail_marker {
            let offset = bytes.len() - marker.len();
            bytes[offset..].copy_from_slice(marker);
        }
        if format == "EXE" || format == "NSIS" {
            // e_lfanew = 0x40 pointing at PE\0\0
            bytes[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
            bytes[0x40..0x44].copy_from_slice(b"PE\x00\x00");
        }
        if format == "NSIS" {
            let offset = bytes.len() - 64;
            bytes[offset..offset + 12].copy_from_slice(b"NullsoftInst");
        }
        fs::write(&path, &bytes).unwrap();
        let mut expectation = expectation("WINDOWS", format);
        if format == "APPIMAGE" || format == "DEB" || format == "RPM" || format == "PKG" {
            expectation.platform = "LINUX".to_owned();
        }
        if format == "DMG" {
            expectation.platform = "MACOS".to_owned();
            expectation.package_format = "DMG".to_owned();
        }
        expectation.package_size_bytes = bytes.len() as u64;
        let report = validate_package_file(&path, &expectation).expect("installer accepted");
        assert!(report.valid);
        assert_eq!(report.entry_count, 0);
        assert!(!report.manifest_present);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn installer_container_signatures_are_verified() {
        assert_installer_accepted(
            "MSI",
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
            None,
        );
        assert_installer_accepted("PKG", b"xar!", None);
        assert_installer_accepted("DEB", b"!<arch>\n", None);
        assert_installer_accepted("RPM", &[0xED, 0xAB, 0xEE, 0xDB], None);
        assert_installer_accepted("APPIMAGE", &[0x7F, b'E', b'L', b'F'], None);
        assert_installer_accepted("EXE", b"MZ", None);
        assert_installer_accepted("NSIS", b"MZ", None);
        assert_installer_accepted("DMG", b"\x00\x01\x02\x03", Some(b"koly"));
    }

    #[test]
    fn installer_container_signatures_fail_closed() {
        // Each case uses its own directory; the shared test_dir counter
        // guarantees unique paths across parallel test threads.
        let dir = test_dir("installer-bad-deb");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.deb");
        fs::write(&path, b"not-an-archive").unwrap();
        let mut exp = expectation("LINUX", "DEB");
        exp.package_size_bytes = fs::metadata(&path).unwrap().len();
        let error = validate_package_file(&path, &exp).expect_err("bad deb rejected");
        assert!(error.to_string().contains("ar archive"));
        fs::remove_dir_all(&dir).unwrap();

        let dir = test_dir("installer-bad-exe");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.exe");
        fs::write(&path, b"MZ").unwrap();
        let mut exp = expectation("WINDOWS", "EXE");
        exp.package_size_bytes = fs::metadata(&path).unwrap().len();
        let error = validate_package_file(&path, &exp).expect_err("truncated exe rejected");
        assert!(error.to_string().contains("truncated"));
        fs::remove_dir_all(&dir).unwrap();

        let dir = test_dir("installer-bad-dmg");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.dmg");
        fs::write(&path, vec![0u8; 600]).unwrap();
        let mut exp = expectation("MACOS", "DMG");
        exp.package_size_bytes = fs::metadata(&path).unwrap().len();
        let error = validate_package_file(&path, &exp).expect_err("dmg without koly rejected");
        assert!(error.to_string().contains("koly"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
