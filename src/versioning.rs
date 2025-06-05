//! Pacman Versioning Utilities
//!
//! This module provides types and utilities for parsing, serializing, and working with Pacman package version strings in Rust. Pacman, the package manager for Arch Linux, uses a versioning system with three main components: `epoch`, `version`, and `release`, formatted as `epoch:version-release`.
//!
//! - **Epoch**: An optional integer that, if present, overrides normal version comparisons. For example, `2:1.0-1` is always considered newer than `1:3.6-1`, regardless of the version numbers.
//! - **Version**: The upstream software version, typically in the form `major.minor.patch` (e.g., `3.0.16`).
//! - **Release**: The number of times the Arch package has been (re)built or modified (e.g., the `2` in `3.0.16-2`).
//!
//! # Example
//!
//! ```rust
//! use nixman::versioning::FullVersion;
//! let v = FullVersion::from("1:2.3.4-5");
//! assert_eq!(v.epoch.0, Some(1));
//! assert_eq!(v.version.major, 2);
//! assert_eq!(v.version.minor, 3);
//! assert_eq!(v.version.patch, 4);
//! assert_eq!(v.release.0, 5);
//! ```

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Write as FmtWrite;
use std::fmt::{self, Display, Formatter};

/// The optional epoch component in Pacman versioning.
///
/// The epoch is an integer that, if present, takes precedence in version comparisons.
/// For example, `2:1.0-1` is newer than `1:3.6-1`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Epoch(pub Option<u32>);

/// The upstream version (e.g., 3.0.16).
///
/// This struct splits the version into major, minor, and patch components.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// The release number (e.g., the "2" in 3.0.16-2).
///
/// The release number is incremented when the Arch package is rebuilt or modified.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Release(pub u32);

/// The full Pacman version: epoch:version-release.
///
/// This struct combines the epoch, version, and release components.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FullVersion {
    pub epoch: Epoch,
    pub version: Version,
    pub release: Release,
}

/// Conversion from a string to the `Epoch` struct.
///
/// Accepts an empty string for `None`, or a stringified integer for `Some`.
impl From<&str> for Epoch {
    /// Converts a string to an `Epoch`. If the string is empty, it returns `None`.
    fn from(s: &str) -> Self {
        if s.is_empty() {
            Self(None)
        } else {
            Self(s.parse().ok())
        }
    }
}

/// Conversion from a string to the `Version` struct.
///
/// Accepts strings in the format `major.minor.patch`. Missing components default to 0.
impl From<&str> for Version {
    /// Converts a string to a `Version`. If the string is malformed, it defaults to 0.0.0.
    fn from(s: &str) -> Self {
        let mut parts = s.split('.');
        let major = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Conversion from a string to the `Release` struct.
///
/// Accepts a string representing the release number. Malformed or empty strings default to 0.
impl From<&str> for Release {
    /// Converts a string to a `Release`. If the string is empty or malformed, it defaults to 0.
    fn from(s: &str) -> Self {
        Self(s.parse().unwrap_or(0))
    }
}

/// Conversion from a string to the `FullVersion` struct.
///
/// Accepts strings in the format `epoch:version-release`, `version-release`, or just `version`.
impl From<&str> for FullVersion {
    /// Converts a string to a `FullVersion`. The string should be in the format `epoch:version-release`.
    fn from(s: &str) -> Self {
        let (epoch_str, rest) = s
            .find(':')
            .map_or(("", s), |idx| (&s[..idx], &s[idx + 1..]));
        let (version_str, release_str) = rest
            .rfind('-')
            .map_or((rest, "0"), |idx| (&rest[..idx], &rest[idx + 1..]));
        Self {
            epoch: Epoch::from(epoch_str),
            version: Version::from(version_str),
            release: Release::from(release_str),
        }
    }
}

/// Serialization for the `FullVersion` struct.
impl Serialize for FullVersion {
    /// Serializes the `FullVersion` struct into a string format `epoch:version-release`.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut version_str = String::new();
        if let Some(epoch) = self.epoch.0 {
            write!(&mut version_str, "{epoch}:").unwrap();
        }
        write!(
            &mut version_str,
            "{}.{}.{}-{}",
            self.version.major, self.version.minor, self.version.patch, self.release.0
        )
        .unwrap();
        serializer.serialize_str(&version_str)
    }
}

/// Deserialization for the `FullVersion` struct.
impl<'_de> Deserialize<'_de> for FullVersion {
    /// Deserializes a string into a `FullVersion` struct. The string should be in the format `epoch:version-release`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'_de>,
    {
        struct FullVersionVisitor;
        impl Visitor<'_> for FullVersionVisitor {
            type Value = FullVersion;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a string like '1:2.3.4-5' or '2.3.4-5'")
            }
            fn visit_str<E>(self, v: &str) -> Result<FullVersion, E>
            where
                E: de::Error,
            {
                Ok(FullVersion::from(v))
            }
        }
        deserializer.deserialize_str(FullVersionVisitor)
    }
}

/// Formats the `FullVersion` struct as a string in the format `epoch:version-release`.
impl Display for FullVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(epoch) = self.epoch.0 {
            write!(f, "{epoch}:")?;
        }
        write!(
            f,
            "{}.{}.{}-{}",
            self.version.major, self.version.minor, self.version.patch, self.release.0
        )
    }
}
