//! Main entry point for the `nixman` CLI tool
//!
//! This binary provides a command-line interface for managing Arch Linux packages and synchronizing them with a YAML configuration file, inspired by the Nix OS approach.
//!
//! # Features
//!
//! - Install, remove, and list packages using `pacman` or `paru`
//! - Freeze the current package state to YAML (optionally with versions)
//! - Apply the YAML configuration to synchronize installed packages
//! - Update all packages and update the YAML
//!
//! # Example
//!
//! ```sh
//! nixman -S htop
//! nixman freeze --versioned
//! nixman apply
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

use clap::{Parser, Subcommand};
use nixman::{ensure_yml, parse_explicit_packages, write_package_list_to_yaml};
mod pacman;

#[derive(Subcommand)]
enum Commands {
    S {
        /// The package(s) to install
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Update,
    Freeze {
        /// Include package versions in the YAML
        #[arg(long)]
        versioned: bool,
    },
    Apply {
        /// Use paru instead of pacman
        #[arg(
            long,
            help = "Use paru instead of pacman for applying packages from YAML"
        )]
        paru: bool,
        /// Continue on errors (try all packages, don't abort on first failure)
        #[arg(long, help = "Continue on errors when removing/installing packages")]
        continue_on_error: bool,
    },
}

/// A simple CLI tool to list installed packages in Arch Linux and save them to
/// a YAML file, mimicking the config approach of nix os.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Run using pacstrap instead of pacman, useful for initial installs
    #[arg(
        long,
        help = "Run with pacstrap instead of pacman, useful for initial installs"
    )]
    pacstrap: bool,
    /// Install package(s) using pacman (like pacman -S)
    #[arg(short = 'S', value_name = "PACKAGE", num_args = 1.., help = "Install package(s) with pacman")]
    install: Option<Vec<String>>,
    /// Remove package(s) using pacman (like pacman -Rns)
    #[arg(short = 'R', value_name = "PACKAGE", num_args = 1.., help = "Remove package(s) with pacman")]
    remove: Option<Vec<String>>,
    /// Use paru instead of pacman for -S/--install
    #[arg(long, help = "Use paru instead of pacman for installing packages")]
    paru: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() {
    let yml_path = match ensure_yml() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to ensure config file: {e}");
            std::process::exit(1);
        }
    };
    println!("Using config file: {}", yml_path.display());

    let cli = Cli::parse();
    if let Some(packages) = cli.remove {
        let use_paru = cli.paru;
        let status = if use_paru {
            pacman::paru_remove(&packages).expect("Failed to execute paru -Rns")
        } else {
            pacman::pacman_remove(&packages, true).expect("Failed to execute sudo pacman -Rns")
        };
        std::process::exit(status.code().unwrap_or(1));
    }
    if let Some(packages) = cli.install {
        let use_paru = cli.paru;
        let status = if use_paru {
            pacman::paru_install(&packages).expect("Failed to execute paru -S")
        } else {
            pacman::pacman_install(&packages, true).expect("Failed to execute sudo pacman -S")
        };
        if status.success() {
            let output = pacman::pacman_list_explicit().expect("Failed to execute pacman -Qe");
            let output = String::from_utf8_lossy(&output.stdout);
            let package_list = parse_explicit_packages(&output, false); // no versions by default
            write_package_list_to_yaml(&package_list, &yml_path).expect("Failed to write to YAML");
            println!("Updated package list written to {}", yml_path.display());
        }
        std::process::exit(status.code().unwrap_or(1));
    }
    if let Some(Commands::S { packages }) = cli.command {
        let use_paru = cli.paru;
        let status = if use_paru {
            pacman::paru_install(&packages).expect("Failed to execute paru -S")
        } else {
            pacman::pacman_install(&packages, true).expect("Failed to execute sudo pacman -S")
        };
        std::process::exit(status.code().unwrap_or(1));
    } else if matches!(cli.command, Some(Commands::Update)) {
        let use_paru = cli.paru;
        let status = if use_paru {
            pacman::paru_update().expect("Failed to execute paru -Syyu")
        } else {
            pacman::pacman_update().expect("Failed to execute sudo pacman -Syyu")
        };
        if status.success() {
            let output = pacman::pacman_list_explicit().expect("Failed to execute pacman -Qe");
            let output = String::from_utf8_lossy(&output.stdout);
            let package_list = parse_explicit_packages(&output, true);
            write_package_list_to_yaml(&package_list, &yml_path).expect("Failed to write to YAML");
            println!("Updated package list written to {}", yml_path.display());
        }
        std::process::exit(status.code().unwrap_or(1));
    } else if let Some(Commands::Freeze { versioned }) = cli.command {
        let output = pacman::pacman_list_explicit().expect("Failed to execute pacman command");
        let output = String::from_utf8_lossy(&output.stdout);
        let package_list = parse_explicit_packages(&output, versioned);
        write_package_list_to_yaml(&package_list, &yml_path).expect("Failed to write to YAML");
        println!("Frozen package list written to {}", yml_path.display());
        std::process::exit(0);
    } else if let Some(Commands::Apply {
        paru,
        continue_on_error,
    }) = cli.command
    {
        match nixman::apply_packages_from_yaml(&yml_path, paru, continue_on_error) {
            Ok(()) => {
                println!("Apply completed successfully.");
                std::process::exit(0);
            }
            Err(msg) => {
                eprintln!("{msg}");
                std::process::exit(1);
            }
        }
    } else {
        let packages = pacman::pacman_list_explicit().expect("Failed to execute pacman command");
        let output = String::from_utf8_lossy(&packages.stdout);
        let package_list = parse_explicit_packages(&output, true);
        write_package_list_to_yaml(&package_list, "packages.yml").expect("Failed to write to YAML");
    }
}
