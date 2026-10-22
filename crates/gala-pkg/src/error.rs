//! Error type for the package manager. Every fallible operation returns a [`PkgError`] value.

use std::path::{Path, PathBuf};

/// Errors produced by the Gala package manager.
#[derive(Debug, thiserror::Error)]
pub enum PkgError {
    /// A file or directory could not be read or written.
    #[error("cannot access `{}`: {source}", path.display())]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// `gala.toml` is malformed or semantically invalid.
    #[error("invalid manifest: {0}")]
    Manifest(String),
    /// `gala.lock` is malformed.
    #[error("invalid lockfile: {0}")]
    Lockfile(String),
    /// A package or dependency name is not a valid identifier.
    #[error(
        "invalid package name `{0}`: names may only contain ASCII letters, digits, `-` and `_`"
    )]
    InvalidName(String),
    /// A version string is not valid semver.
    #[error("invalid version `{value}`: {message}")]
    InvalidVersion {
        /// The offending version text.
        value: String,
        /// Parser message.
        message: String,
    },
    /// A version requirement is not valid semver.
    #[error("invalid requirement `{requirement}` for `{name}`: {message}")]
    InvalidRequirement {
        /// Dependency name.
        name: String,
        /// The offending requirement text.
        requirement: String,
        /// Parser message.
        message: String,
    },
    /// The package is not known to the registry.
    #[error("package `{0}` was not found in the registry")]
    PackageNotFound(String),
    /// No published version satisfies the requirement.
    #[error("no version of `{name}` matches requirement `{requirement}`")]
    NoMatchingVersion {
        /// Package name.
        name: String,
        /// The requirement that matched nothing.
        requirement: String,
    },
    /// Two requirements on the same package cannot be satisfied together.
    #[error(
        "conflicting requirements for `{name}`: {first} and {second} cannot both be satisfied"
    )]
    Conflict {
        /// Package name.
        name: String,
        /// First conflicting requirement, with its requirer.
        first: String,
        /// Second conflicting requirement, with its requirer.
        second: String,
    },
    /// A dependency named in a command is not declared in `gala.toml`.
    #[error("dependency `{0}` is not declared in gala.toml")]
    DependencyNotFound(String),
    /// `gala.lock` does not match what `gala.toml` resolves to.
    #[error("gala.lock is out of date with gala.toml; run `gala-pkg update` to regenerate it")]
    LockedMismatch,
    /// `gala.lock` is missing when `--locked` was requested.
    #[error("gala.lock is missing; run `gala-pkg update` to create it")]
    LockfileMissing,
    /// A registry package does not match the checksum recorded in the lockfile.
    #[error("checksum mismatch for `{name}@{version}`: expected {expected}, found {actual}")]
    Checksum {
        /// Package name.
        name: String,
        /// Package version.
        version: String,
        /// Checksum recorded in `gala.lock`.
        expected: String,
        /// Checksum computed from the registry contents.
        actual: String,
    },
    /// A registry dependency was declared but no `--registry` was given.
    #[error("no registry configured; pass --registry <dir> to resolve registry dependency `{0}`")]
    NoRegistry(String),
    /// The registry is malformed or the operation is not supported for it.
    #[error("registry error: {0}")]
    Registry(String),
    /// The requested operation is intentionally not implemented.
    #[error("{0}")]
    Unsupported(String),
    /// The package version already exists in the registry (versions are immutable).
    #[error("`{0}` is already published; published versions are immutable")]
    AlreadyPublished(String),
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, PkgError>;

/// Build a closure that maps an [`std::io::Error`] into [`PkgError::Io`] for `path`.
pub(crate) fn io_at(path: &Path) -> impl FnOnce(std::io::Error) -> PkgError + '_ {
    move |source| PkgError::Io { path: path.to_path_buf(), source }
}
