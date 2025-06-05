# nixman

A Nix-inspired package manager for Arch Linux, designed to make system package management reproducible, auditable, and scriptable—without overstepping into areas you want to control yourself.

## Features

- **Track and manage installed packages** using a simple YAML file (`packages.yml`)
- **Install, remove, and list packages** with `pacman` or `paru` from the command line
- **Freeze** your current package state to YAML (optionally with versions)
- **Apply** a YAML configuration to synchronize installed packages
- **Update** all packages and update the YAML
- **Library API**: Use as a Rust library to programmatically manipulate pacman packages

## Why?

Arch Linux is powerful, but keeping your package list in sync across machines is tedious. `nixman` lets you:

- Reproduce your package set on a new install by copying your YAML file and running `nixman apply --pacstrap`
- Track your package state in version control
- Share your package list with others
- Script package management tasks in Rust

Unlike NixOS, `nixman` does **not** automate partitioning, timezones, or other system setup—just package management. You keep full control over your install process.

## Quickstart

### 1. Install

You can install `nixman` using Cargo, either from crates.io or from your local clone:

**From crates.io:**

```sh
cargo install nixman
```

**From a local clone:**

```sh
git clone https://github.com/ofluffydev/nixman.git
cd nixman
cargo install --path .
```

### 2. Track Your Packages

Freeze your current package list to YAML:

```sh
nixman freeze
```

This creates (or updates) `~/.config/nixman/packages.yml`.

### 3. Apply on a New System

Copy your `packages.yml` to the new system, then run:

```sh
nixman apply --pacstrap
```

This will install all packages listed in the YAML using `pacstrap` (for initial installs) or `pacman`/`paru`.

### 4. Install/Remove Packages

Install a package and update your YAML:

```sh
nixman -S htop
```

Remove a package and update your YAML:

```sh
nixman -R htop
```

### 5. Update All Packages

```sh
nixman update
```

## YAML Format

The YAML file is simple and versioned:

```yaml
packages:
  - name: htop
    version: 3.2.2-1
  - name: neovim
    version: 0.9.5-2
  - git
```

You can edit this file by hand and/or keep it in git.

## Paru and AUR Packages

If you want to use the `--paru` flag (for installing/removing AUR packages), **paru must be installed** on your system. If your YAML file contains AUR packages and you do not have paru installed, `nixman` will fail to install or remove those packages.

**Note:** Bootstrapping a new system (e.g., with `--pacstrap`) that includes AUR packages in your YAML is currently unsupported. You can use the `--continue` flag to ignore failed packages and circumvent this limitation, but you will need to manually install AUR packages or rerun with `--paru` after the initial bootstrap.

## Library Usage

You can use `nixman` as a Rust library to programmatically manage packages:

```rust
use nixman::{ensure_yml, write_package_list_to_yaml, parse_explicit_packages};
let yml_path = ensure_yml().unwrap();
let pkgs = parse_explicit_packages("htop 1.0.0-1", true);
write_package_list_to_yaml(&pkgs, &yml_path).unwrap();
```

## Philosophy

- **Reproducibility**: Track your package state in a single YAML file
- **Transparency**: No magic—just a thin wrapper over pacman/paru
- **Manual is good**: You still handle partitioning, timezones, and dotfiles
- **Scriptable**: Use as a CLI or as a Rust library

## FAQ

**Q: Does this replace Nix or NixOS?**

A: No. This is for Arch users who want reproducible package management, not full system declarativity.

**Q: Can I use this with AUR packages?**

A: Yes! Use the `--paru` flag to install/remove AUR packages with `paru`. **Note:** If your YAML contains AUR packages, you must have paru installed, or those packages will not be handled.

**Q: What about dotfiles, system config, etc?**

A: `nixman` only manages packages. Use your favorite dotfile manager for the rest, optionally including your `packages.yml` in version control.

## Contributing

PRs and issues welcome! See the issues tab for more.

## License

MIT OR Apache-2.0
