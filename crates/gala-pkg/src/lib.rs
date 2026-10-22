//! Package manager for Gala.
//!
//! - [`manifest`]: `gala.toml` parsing, validation and in-place dependency editing.
//! - [`resolver`]: deterministic semver resolution with backtracking.
//! - [`lockfile`]: canonical `gala.lock` serialization.
//! - [`registry`]: local directory registry (v1). Remote registries are refused.
//! - [`commands`]: `init`, `add`, `remove`, `update`, `install` and `publish`, each returning a
//!   `gala-pkg.v1` JSON value.

pub mod commands;
pub mod error;
pub mod lockfile;
pub mod manifest;
pub mod registry;
pub mod resolver;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub use commands::JSON_SCHEMA;
pub use error::{PkgError, Result};
pub use lockfile::{LockedPackage, Lockfile, LOCKFILE_NAME};
pub use manifest::{DependencySource, DependencySpec, Manifest, MANIFEST_FILE};
pub use registry::LocalRegistry;
pub use resolver::{resolve, ResolveOptions};

/// Create a new package in `./<name>` and print a one-line summary.
///
/// Compatibility wrapper for the `gala new` command; prefer [`commands::init_project`].
///
/// # Errors
///
/// Returns the error message of the underlying [`PkgError`].
pub fn init_project(name: &str, is_lib: bool) -> std::result::Result<(), String> {
    commands::init_project(Path::new("."), name, is_lib).map_err(|e| e.to_string())?;
    println!("created package '{name}' at ./{name}/");
    Ok(())
}

/// Add a dependency to `./gala.toml` in place, without resolving or writing `gala.lock`.
///
/// Compatibility wrapper for the `gala add` command. `git` takes precedence over `path`, and
/// `version` (default `*`) applies when neither is given. Use [`commands::add`] for the full
/// resolve-and-lock flow.
///
/// # Errors
///
/// Returns the error message of the underlying [`PkgError`].
pub fn add_dependency(
    name: &str,
    version: Option<String>,
    git: Option<String>,
    path: Option<PathBuf>,
) -> std::result::Result<(), String> {
    let spec = if let Some(url) = git {
        DependencySpec::git(url, None)
    } else if let Some(p) = path {
        DependencySpec::path(p.display().to_string())
    } else {
        DependencySpec::requirement(version.unwrap_or_else(|| "*".to_string()))
    };
    manifest::add_dependency(Path::new(MANIFEST_FILE), name, &spec).map_err(|e| e.to_string())?;
    println!("added '{name}' to dependencies");
    Ok(())
}
