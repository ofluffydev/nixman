//! Simple wrapper for pacman command invocations
//!
//! # Overview
//!
//! This module provides simple Rust wrappers for common `pacman` and `paru` package manager operations on Arch Linux systems.
//! It allows programmatic installation, removal, and listing of packages, with optional sudo support for privileged operations.
//!
//! # Functions
//!
//! - [`pacman_install`]: Install packages using pacman, optionally with sudo.
//! - [`pacman_list_explicit`]: List explicitly installed packages.
//! - [`paru_install`]: Install packages using paru (AUR helper).
//! - [`pacman_remove`]: Remove packages using pacman, optionally with sudo.
//! - [`paru_remove`]: Remove packages using paru.
//!
//! # Example
//!
//! ```rust
//! use nixman::pacman::{pacman_install, pacman_list_explicit};
//! let status = pacman_install(&["htop".to_string()], true)?;
//! let output = pacman_list_explicit()?;
//! ```

use std::process::{Command, ExitStatus, Output};

/// Installs the given packages using pacman.
///
/// # Arguments
/// * `packages` - A slice of package names to install.
/// * `use_sudo` - Whether to run pacman with sudo.
///
/// # Returns
/// * `std::io::Result<ExitStatus>` - The exit status of the pacman command.
///
/// # Errors
/// Returns an error if the pacman command could not be executed.
pub fn pacman_install(packages: &[String], use_sudo: bool) -> std::io::Result<ExitStatus> {
    let mut cmd = if use_sudo {
        let mut c = Command::new("sudo");
        c.arg("pacman");
        c
    } else {
        Command::new("pacman")
    };
    cmd.arg("-S").args(packages);
    cmd.status()
}

/// Lists explicitly installed packages using `pacman -Qe`.
///
/// # Returns
/// * `std::io::Result<Output>` - The output of the pacman command.
///
/// # Errors
/// Returns an error if the pacman command could not be executed.
pub fn pacman_list_explicit() -> std::io::Result<Output> {
    Command::new("pacman").arg("-Qe").output()
}

/// Installs the given packages using paru (AUR helper).
///
/// # Arguments
/// * `packages` - A slice of package names to install.
///
/// # Returns
/// * `std::io::Result<ExitStatus>` - The exit status of the paru command.
///
/// # Errors
/// Returns an error if the paru command could not be executed.
pub fn paru_install(packages: &[String]) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("paru");
    cmd.arg("-S").args(packages);
    cmd.status()
}

/// Removes the given packages using pacman.
///
/// # Arguments
/// * `packages` - A slice of package names to remove.
/// * `use_sudo` - Whether to run pacman with sudo.
///
/// # Returns
/// * `std::io::Result<ExitStatus>` - The exit status of the pacman command.
///
/// # Errors
/// Returns an error if the pacman command could not be executed.
pub fn pacman_remove(packages: &[String], use_sudo: bool) -> std::io::Result<ExitStatus> {
    let mut cmd = if use_sudo {
        let mut c = Command::new("sudo");
        c.arg("pacman");
        c
    } else {
        Command::new("pacman")
    };
    cmd.arg("-Rns").args(packages);
    cmd.status()
}

/// Removes the given packages using paru.
///
/// # Arguments
/// * `packages` - A slice of package names to remove.
///
/// # Returns
/// * `std::io::Result<ExitStatus>` - The exit status of the paru command.
///
/// # Errors
/// Returns an error if the paru command could not be executed.
pub fn paru_remove(packages: &[String]) -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("paru");
    cmd.arg("-Rns").args(packages);
    cmd.status()
}
