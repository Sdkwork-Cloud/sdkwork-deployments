//! Semantic versioning (SemVer 2.0.0 subset) for deploy releases and packages.
//!
//! Supports `MAJOR.MINOR.PATCH[-prerelease][+build]` with bounded lengths, no
//! leading zeros, and SemVer precedence ordering. Build metadata participates
//! in identity but not in precedence.

use std::cmp::Ordering;
use std::fmt;

pub const MAXIMUM_VERSION_LENGTH: usize = 128;
pub const MAXIMUM_IDENTIFIER_LENGTH: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Vec<String>,
    pub build: Vec<String>,
}

impl SemanticVersion {
    /// Parses and validates a bounded SemVer 2.0.0 string.
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.is_empty() || value.len() > MAXIMUM_VERSION_LENGTH {
            return Err(format!(
                "semantic version must be 1..={MAXIMUM_VERSION_LENGTH} characters"
            ));
        }
        let (core, metadata) = match value.split_once('+') {
            Some((core, build)) => (core, Some(build)),
            None => (value, None),
        };
        let (core, prerelease) = match core.split_once('-') {
            Some((core, prerelease)) => (core, Some(prerelease)),
            None => (core, None),
        };

        let mut parts = core.split('.');
        let major = parse_number(parts.next(), "major")?;
        let minor = parse_number(parts.next(), "minor")?;
        let patch = parse_number(parts.next(), "patch")?;
        if parts.next().is_some() {
            return Err("semantic version core must have exactly three parts".into());
        }

        let prerelease = match prerelease {
            Some(value) => parse_identifiers(value, false)?,
            None => Vec::new(),
        };
        let build = match metadata {
            Some(value) => parse_identifiers(value, true)?,
            None => Vec::new(),
        };

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Renders the canonical string including build metadata.
    pub fn to_canonical_string(&self) -> String {
        let mut output = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if !self.prerelease.is_empty() {
            output.push('-');
            output.push_str(&self.prerelease.join("."));
        }
        if !self.build.is_empty() {
            output.push('+');
            output.push_str(&self.build.join("."));
        }
        output
    }

    pub fn is_prerelease(&self) -> bool {
        !self.prerelease.is_empty()
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_canonical_string())
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| compare_prerelease(&self.prerelease, &other.prerelease))
        // build metadata is ignored for precedence
    }
}

fn parse_number(part: Option<&str>, label: &str) -> Result<u64, String> {
    let Some(part) = part else {
        return Err(format!("semantic version is missing the {label} part"));
    };
    if part.is_empty() {
        return Err(format!("semantic version {label} part is empty"));
    }
    if part.len() > 1 && part.starts_with('0') {
        return Err(format!(
            "semantic version {label} part must not contain leading zeros"
        ));
    }
    part.parse::<u64>()
        .map_err(|_| format!("semantic version {label} part is not a number"))
}

fn parse_identifiers(value: &str, allow_leading_zeros: bool) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Err("semantic version identifier list is empty".into());
    }
    let mut identifiers = Vec::new();
    for identifier in value.split('.') {
        validate_identifier(identifier, allow_leading_zeros)?;
        identifiers.push(identifier.to_owned());
    }
    Ok(identifiers)
}

fn validate_identifier(identifier: &str, allow_leading_zeros: bool) -> Result<(), String> {
    if identifier.is_empty() || identifier.len() > MAXIMUM_IDENTIFIER_LENGTH {
        return Err(format!(
            "semantic version identifier must be 1..={MAXIMUM_IDENTIFIER_LENGTH} characters"
        ));
    }
    if !identifier
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("semantic version identifier contains invalid characters".into());
    }
    if identifier.len() > 1
        && !allow_leading_zeros
        && identifier.bytes().all(|byte| byte.is_ascii_digit())
        && identifier.starts_with('0')
    {
        return Err("semantic version numeric identifier must not contain leading zeros".into());
    }
    Ok(())
}

/// SemVer precedence for prerelease identifiers: identifiers are compared
/// dot-by-dot; numeric identifiers sort lower than alphanumeric ones; a
/// version without prerelease sorts higher than any prerelease of the same
/// core.
fn compare_prerelease(left: &[String], right: &[String]) -> Ordering {
    if left.is_empty() && right.is_empty() {
        return Ordering::Equal;
    }
    if left.is_empty() {
        return Ordering::Greater;
    }
    if right.is_empty() {
        return Ordering::Less;
    }
    for (left_id, right_id) in left.iter().zip(right.iter()) {
        let ordering = compare_identifier(left_id, right_id);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

fn compare_identifier(left: &str, right: &str) -> Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left_number), Ok(right_number)) => left_number.cmp(&right_number),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_semver() {
        let version = SemanticVersion::parse("1.4.2-alpha.1+build.7").expect("parse");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 4);
        assert_eq!(version.patch, 2);
        assert_eq!(version.prerelease, vec!["alpha", "1"]);
        assert_eq!(version.build, vec!["build", "7"]);
        assert_eq!(version.to_canonical_string(), "1.4.2-alpha.1+build.7");
    }

    #[test]
    fn rejects_invalid_versions() {
        for value in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "01.2.3",
            "1.02.3",
            "1.2.03",
            "1.2.3-",
            "1.2.3+",
            "1.2.3-01",
            "1.2.3-!bad",
        ] {
            assert!(
                SemanticVersion::parse(value).is_err(),
                "should reject {value}"
            );
        }
    }

    #[test]
    fn orders_by_semver_precedence() {
        let ordered = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
            "2.0.0",
        ];
        for pair in ordered.windows(2) {
            let left = SemanticVersion::parse(pair[0]).unwrap();
            let right = SemanticVersion::parse(pair[1]).unwrap();
            assert!(left < right, "{} should sort before {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn build_metadata_does_not_affect_precedence() {
        let base = SemanticVersion::parse("1.2.3").unwrap();
        let with_build = SemanticVersion::parse("1.2.3+build.1").unwrap();
        assert_eq!(base.cmp(&with_build), Ordering::Equal);
        assert_ne!(base.to_canonical_string(), with_build.to_canonical_string());
    }
}
