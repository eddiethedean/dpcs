//! Versioning helpers for DPCS artifacts (SPEC Ch 20).

use std::fmt;

/// Parsed SemVer-compatible version identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVersion {
    /// Major version component.
    pub major: u64,
    /// Minor version component.
    pub minor: u64,
    /// Patch version component.
    pub patch: u64,
    /// Optional pre-release identifier (for example `draft` or `alpha.1`).
    pub pre_release: Option<String>,
    /// Optional build metadata.
    pub build: Option<String>,
}

impl fmt::Display for ParsedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{pre}")?;
        }
        if let Some(build) = &self.build {
            write!(f, "+{build}")?;
        }
        Ok(())
    }
}

/// Returns `true` when `version` looks like a non-empty version string.
pub fn is_present_version(version: &str) -> bool {
    !version.trim().is_empty()
}

/// Returns `true` when `version` is a valid SemVer-compatible identifier.
pub fn is_valid_version(version: &str) -> bool {
    parse_version(version).is_ok()
}

/// Parse a SemVer-compatible version string (`MAJOR.MINOR.PATCH[-pre][+build]`).
///
/// Dot-separated numeric components must be non-empty. Pre-release and build
/// metadata follow SemVer 2.0 rules at a practical subset (alphanumeric and
/// `.` / `-` characters).
pub fn parse_version(version: &str) -> Result<ParsedVersion, String> {
    let version = version.trim();
    if version.is_empty() {
        return Err("version must not be empty".to_owned());
    }

    let (core_and_pre, build) = match version.split_once('+') {
        Some((left, right)) => {
            if right.is_empty() || !is_semver_ident(right, true) {
                return Err("invalid build metadata".to_owned());
            }
            (left, Some(right.to_owned()))
        }
        None => (version, None),
    };

    let (core, pre_release) = match core_and_pre.split_once('-') {
        Some((left, right)) => {
            if right.is_empty() || !is_semver_ident(right, false) {
                return Err("invalid pre-release identifier".to_owned());
            }
            (left, Some(right.to_owned()))
        }
        None => (core_and_pre, None),
    };

    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("version must be MAJOR.MINOR or MAJOR.MINOR.PATCH".to_owned());
    }

    let major = parse_numeric_component(parts[0], "major")?;
    let minor = parse_numeric_component(parts[1], "minor")?;
    let patch = if parts.len() == 3 {
        parse_numeric_component(parts[2], "patch")?
    } else {
        0
    };

    Ok(ParsedVersion {
        major,
        minor,
        patch,
        pre_release,
        build,
    })
}

/// Compare two version strings for toolkit/profile compatibility.
///
/// Equality is exact when both sides match. Otherwise compare normalized
/// `MAJOR.MINOR` prefixes after stripping a trailing `-draft` suffix.
pub fn versions_compatible(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    normalize_major_minor(left) == normalize_major_minor(right)
}

/// Normalize a version to `MAJOR.MINOR` for soft compatibility checks.
pub fn normalize_major_minor(value: &str) -> String {
    value
        .trim()
        .trim_end_matches("-draft")
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".")
}

fn parse_numeric_component(value: &str, label: &str) -> Result<u64, String> {
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("{label} component must be a non-negative integer"));
    }
    if value.len() > 1 && value.starts_with('0') {
        return Err(format!("{label} component must not have leading zeros"));
    }
    value
        .parse::<u64>()
        .map_err(|_| format!("{label} component is out of range"))
}

fn is_semver_ident(value: &str, allow_leading_zeros: bool) -> bool {
    value.split('.').all(|part| {
        if part.is_empty() {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return false;
        }
        if !allow_leading_zeros
            && part.chars().all(|c| c.is_ascii_digit())
            && part.len() > 1
            && part.starts_with('0')
        {
            return false;
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_semver_and_major_minor() {
        let full = parse_version("1.2.3-alpha.1+build.7").unwrap();
        assert_eq!(full.major, 1);
        assert_eq!(full.minor, 2);
        assert_eq!(full.patch, 3);
        assert_eq!(full.pre_release.as_deref(), Some("alpha.1"));
        assert_eq!(full.build.as_deref(), Some("build.7"));

        let short = parse_version("1.0").unwrap();
        assert_eq!(short.patch, 0);

        assert!(parse_version("1.0.0-draft").is_ok());
        assert!(parse_version("").is_err());
        assert!(parse_version("1").is_err());
        assert!(parse_version("01.0.0").is_err());
    }

    #[test]
    fn compatibility_uses_major_minor() {
        assert!(versions_compatible("1.0.0-draft", "1.0.0"));
        assert!(versions_compatible("1.0.0", "1.0.5"));
        assert!(!versions_compatible("1.0.0", "2.0.0"));
    }
}
