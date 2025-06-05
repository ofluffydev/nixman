//! Library for the `nixman` package manager
//!
//! Provides core types and functions for synchronizing Arch Linux packages with a YAML configuration file, parsing package lists, and handling versioning.
//!
//! # Modules
//!
//! - [`versioning`]: Pacman version string parsing and utilities
//!
//! # Example
//!
//! ```rust
//! use nixman::{ensure_yml, write_package_list_to_yaml, parse_explicit_packages};
//! let yml_path = ensure_yml().unwrap();
//! let pkgs = parse_explicit_packages("htop 1.0.0-1", true);
//! write_package_list_to_yaml(&pkgs, &yml_path).unwrap();
//! ```

// Be a perfectionist, no code is good enough!
#![deny(
    clippy::all,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery
)]

use crate::versioning::FullVersion;
use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeStruct, Serializer};
use std::fmt;
use std::io::Write;
use std::path::PathBuf;

pub mod pacman;
pub mod versioning;

#[derive(PartialEq, Eq, Debug)]
pub struct Package {
    pub name: String,
    pub version: Option<FullVersion>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PackageList {
    pub packages: Vec<Package>,
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        let mut parts = s.splitn(2, ' ');
        let name = parts.next().unwrap_or("").to_string();
        let version_str = parts.next().unwrap_or("");
        let version = if version_str.is_empty() {
            None
        } else {
            Some(FullVersion::from(version_str))
        };
        Self { name, version }
    }
}

impl serde::Serialize for Package {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.version.is_none() {
            serializer.serialize_str(&self.name)
        } else {
            let mut state = serializer.serialize_struct("Package", 2)?;
            state.serialize_field("name", &self.name)?;
            if let Some(ref v) = self.version {
                state.serialize_field("version", v)?;
            }
            state.end()
        }
    }
}

impl<'de> serde::Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PackageVisitor;
        impl<'de> Visitor<'de> for PackageVisitor {
            type Value = Package;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a map with name and optional version")
            }
            fn visit_str<E>(self, v: &str) -> Result<Package, E>
            where
                E: de::Error,
            {
                Ok(Package {
                    name: v.to_string(),
                    version: None,
                })
            }
            fn visit_map<M>(self, mut map: M) -> Result<Package, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut name = None;
                let mut version = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "version" => version = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                Ok(Package { name, version })
            }
        }
        deserializer.deserialize_any(PackageVisitor)
    }
}

impl serde::Serialize for PackageList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_struct("PackageList", 1)?;
        map.serialize_field("packages", &self.packages)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for PackageList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            packages: Vec<Package>,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            packages: helper.packages,
        })
    }
}

/// Ensures the XDG-compliant YML file exists. (~/.config/nixman/packages.yaml)
///
/// # Errors
/// Returns an error if the config directory or file cannot be created or written.
pub fn ensure_yml() -> std::io::Result<PathBuf> {
    let mut path = PathBuf::from(std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.config")
    }));
    path.push("nixman");
    std::fs::create_dir_all(&path)?;

    path.push("packages.yml");
    let existed = path.exists();
    if !existed {
        std::fs::write(&path, "")?;
        eprintln!("Warning: {} did not exist and was created.", path.display());
    }
    Ok(path)
}

/// Write a package list to a YAML file at the given path.
///
/// # Errors
/// Returns an error if the file cannot be created or written.
///
/// # Panics
/// Panics if serialization to YAML fails (should not happen for valid data).
pub fn write_package_list_to_yaml<P: AsRef<std::path::Path>>(
    package_list: &PackageList,
    path: P,
) -> std::io::Result<()> {
    let yml = serde_yml::to_string(package_list).expect("Failed to serialize to YAML");
    let mut file = std::fs::File::create(path)?;
    file.write_all(yml.as_bytes())?;
    Ok(())
}

/// Parse the output of `pacman -Qe` into a `PackageList`, optionally versioned.
pub fn parse_explicit_packages(output: &str, versioned: bool) -> PackageList {
    let packages: Vec<Package> = if versioned {
        output.lines().map(Package::from).collect()
    } else {
        output
            .lines()
            .map(|line| {
                let name = line.split_whitespace().next().unwrap_or("").to_string();
                Package {
                    name,
                    version: None,
                }
            })
            .collect()
    };
    PackageList { packages }
}

/// Synchronize installed packages with the list in the YAML file.
///
/// # Returns
/// `(to_install, to_remove)` as `Vec<String>` of package names.
///
/// # Errors
/// Returns an error if the YAML file cannot be read or parsed.
pub fn sync_packages_from_yaml<P: AsRef<std::path::Path>>(
    yml_path: P,
    installed_packages: &[String],
) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let yml_content = std::fs::read_to_string(&yml_path)?;
    let package_list: PackageList = serde_yml::from_str(&yml_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let installed: std::collections::HashSet<String> = installed_packages.iter().cloned().collect();
    let wanted: std::collections::HashSet<String> = package_list
        .packages
        .iter()
        .map(|pkg| pkg.name.clone())
        .collect();
    let to_install: Vec<String> = wanted.difference(&installed).cloned().collect();
    let to_remove: Vec<String> = installed.difference(&wanted).cloned().collect();
    Ok((to_install, to_remove))
}

/// Apply the YAML configuration to synchronize installed packages.
///
/// - `yml_path`: Path to the YAML file
/// - `use_paru`: Use paru instead of pacman
/// - `continue_on_error`: Continue on errors (try all packages, don't abort on first failure)
///
/// # Errors
/// Returns `Err(String)` with a summary of failed packages or IO errors.
pub fn apply_packages_from_yaml<P: AsRef<std::path::Path>>(
    yml_path: P,
    use_paru: bool,
    continue_on_error: bool,
) -> Result<(), String> {
    let installed_output = crate::pacman::pacman_list_explicit().map_err(|e| e.to_string())?;
    let installed_str = String::from_utf8_lossy(&installed_output.stdout);
    let installed: Vec<String> = installed_str
        .lines()
        .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    let (to_install, to_remove) =
        crate::sync_packages_from_yaml(&yml_path, &installed).map_err(|e| e.to_string())?;
    if to_install.is_empty() && to_remove.is_empty() {
        return Ok(());
    }
    let mut failed_removals = Vec::new();
    let mut failed_installs = Vec::new();
    if !to_remove.is_empty() {
        if continue_on_error {
            for pkg in &to_remove {
                let status = if use_paru {
                    crate::pacman::paru_remove(&[pkg.clone()]).map_err(|e| e.to_string())?
                } else {
                    crate::pacman::pacman_remove(&[pkg.clone()], true).map_err(|e| e.to_string())?
                };
                if !status.success() {
                    failed_removals.push(pkg.clone());
                }
            }
        } else {
            let status = if use_paru {
                crate::pacman::paru_remove(&to_remove).map_err(|e| e.to_string())?
            } else {
                crate::pacman::pacman_remove(&to_remove, true).map_err(|e| e.to_string())?
            };
            if !status.success() {
                return Err("Failed to remove some packages".to_string());
            }
        }
    }
    if !to_install.is_empty() {
        if continue_on_error {
            for pkg in &to_install {
                let status = if use_paru {
                    crate::pacman::paru_install(&[pkg.clone()]).map_err(|e| e.to_string())?
                } else {
                    crate::pacman::pacman_install(&[pkg.clone()], true)
                        .map_err(|e| e.to_string())?
                };
                if !status.success() {
                    failed_installs.push(pkg.clone());
                }
            }
        } else {
            let status = if use_paru {
                crate::pacman::paru_install(&to_install).map_err(|e| e.to_string())?
            } else {
                crate::pacman::pacman_install(&to_install, true).map_err(|e| e.to_string())?
            };
            if !status.success() {
                return Err("Failed to install some packages".to_string());
            }
        }
    }
    if failed_removals.is_empty() && failed_installs.is_empty() {
        Ok(())
    } else {
        use std::fmt::Write as _;
        let mut msg = String::new();
        if !failed_removals.is_empty() {
            let _ = write!(
                msg,
                "Failed to remove packages: {}",
                failed_removals.join(", ")
            );
            msg.push('\n');
        }
        if !failed_installs.is_empty() {
            let _ = write!(
                msg,
                "Failed to install packages: {}",
                failed_installs.join(", ")
            );
            msg.push('\n');
        }
        Err(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::FullVersion;
    use std::fs;

    /// Tests the conversion of a package to a `Package` struct to and from YAML.
    #[test]
    fn yaml_roundtrip_package_list() {
        // Simulate some packages for testing
        let packages = vec![
            Package {
                name: "foo".to_string(),
                version: Some(FullVersion::from("1.0.0-1")),
            },
            Package {
                name: "bar".to_string(),
                version: Some(FullVersion::from("2.1.0-2")),
            },
        ];
        let package_list = PackageList { packages };

        let yml = serde_yml::to_string(&package_list).expect("Failed to serialize to YAML");
        let deserialized: PackageList =
            serde_yml::from_str(&yml).expect("Failed to deserialize YAML");
        assert_eq!(
            package_list, deserialized,
            "Packages before and after YAML roundtrip differ!"
        );
    }

    /// Tests the conversion of a package list to a YAML file and back.
    #[test]
    fn yaml_file_roundtrip_package_list() {
        let packages = vec![Package {
            name: "baz".to_string(),
            version: Some(FullVersion::from("3.2.1-3")),
        }];
        let package_list = PackageList { packages };
        let yml = serde_yml::to_string(&package_list).expect("Failed to serialize to YAML");
        fs::write("test_packages.yml", &yml).expect("Failed to write test YAML file");
        let yml_content =
            fs::read_to_string("test_packages.yml").expect("Failed to read test YAML file");
        let deserialized: PackageList =
            serde_yml::from_str(&yml_content).expect("Failed to deserialize YAML");
        assert_eq!(
            package_list, deserialized,
            "Packages before and after YAML file roundtrip differ!"
        );
        let _ = fs::remove_file("test_packages.yml");
    }
}
