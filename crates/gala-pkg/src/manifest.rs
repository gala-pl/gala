//! `gala.toml` manifest parsing, validation, and in-place dependency editing.
//!
//! Dependency forms accepted under `[dependencies]`:
//!
//! ```toml
//! [package]
//! name = "app"
//! version = "0.1.0"
//!
//! [dependencies]
//! greeter = "^1.2"                                    # registry, semver requirement
//! util = { version = ">=0.3, <0.5" }                  # registry, table form
//! lib-git = { git = "https://example.org/lib.git", rev = "abc123" }
//! lib-local = { path = "../lib-local" }
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

use crate::error::{io_at, PkgError, Result};

/// File name of the manifest inside a project directory.
pub const MANIFEST_FILE: &str = "gala.toml";

/// A parsed `gala.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The `[package]` table.
    pub package: PackageInfo,
    /// The `[dependencies]` table, keyed by dependency name (sorted).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, DependencySpec>,
    /// The `[dev-dependencies]` table. Parsed and validated, not resolved in v1.
    #[serde(default, rename = "dev-dependencies", skip_serializing_if = "BTreeMap::is_empty")]
    pub dev_dependencies: BTreeMap<String, DependencySpec>,
}

/// The `[package]` table of a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name (ASCII letters, digits, `-`, `_`).
    pub name: String,
    /// Package version; must be valid semver.
    pub version: String,
    /// Language edition.
    #[serde(default = "default_edition")]
    pub edition: String,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional list of authors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    /// Optional SPDX license identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

fn default_edition() -> String {
    "2021".to_string()
}

/// A dependency as written in `gala.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Shorthand registry requirement, e.g. `greeter = "^1.2"`.
    Requirement(String),
    /// Table form with exactly one of `version`, `git` or `path`.
    Detailed(DetailedDependency),
}

/// Table form of a dependency: `{ version = ".."}`, `{ git = "..", rev = ".." }` or `{ path = ".." }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailedDependency {
    /// Semver requirement for registry dependencies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Git repository URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    /// Git revision (commit, tag or branch); only valid together with `git`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// Local path, relative to the depending project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// The validated source of a dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySource {
    /// Resolved from a registry index using a semver requirement.
    Registry(VersionReq),
    /// Git repository. Recorded in the lockfile; not fetched by v1.
    Git {
        /// Repository URL.
        url: String,
        /// Optional revision.
        rev: Option<String>,
    },
    /// Local directory containing another `gala.toml`, as written in the manifest.
    Path(String),
}

impl DependencySpec {
    /// Validate this spec and return its source.
    ///
    /// # Errors
    ///
    /// Returns an error if the requirement is not valid semver, if zero or several sources are
    /// given, or if `rev` is used without `git`.
    pub fn source(&self, name: &str) -> Result<DependencySource> {
        validate_name(name)?;
        match self {
            Self::Requirement(req) => Ok(DependencySource::Registry(parse_req(name, req)?)),
            Self::Detailed(d) => {
                let count = [d.version.is_some(), d.git.is_some(), d.path.is_some()]
                    .iter()
                    .filter(|set| **set)
                    .count();
                if count != 1 {
                    return Err(PkgError::Manifest(format!(
                        "dependency `{name}` must set exactly one of `version`, `git` or `path`"
                    )));
                }
                if d.rev.is_some() && d.git.is_none() {
                    return Err(PkgError::Manifest(format!(
                        "dependency `{name}` sets `rev` without `git`"
                    )));
                }
                if let Some(v) = &d.version {
                    return Ok(DependencySource::Registry(parse_req(name, v)?));
                }
                if let Some(url) = &d.git {
                    if url.trim().is_empty() {
                        return Err(PkgError::Manifest(format!(
                            "dependency `{name}` has an empty `git` URL"
                        )));
                    }
                    return Ok(DependencySource::Git { url: url.clone(), rev: d.rev.clone() });
                }
                match &d.path {
                    Some(p) if !p.trim().is_empty() => Ok(DependencySource::Path(p.clone())),
                    _ => {
                        Err(PkgError::Manifest(format!("dependency `{name}` has an empty `path`")))
                    }
                }
            }
        }
    }

    /// Convenience constructor for a registry requirement.
    pub fn requirement(req: impl Into<String>) -> Self {
        Self::Requirement(req.into())
    }

    /// Convenience constructor for a git dependency.
    pub fn git(url: impl Into<String>, rev: Option<String>) -> Self {
        Self::Detailed(DetailedDependency { version: None, git: Some(url.into()), rev, path: None })
    }

    /// Convenience constructor for a path dependency.
    pub fn path(path: impl Into<String>) -> Self {
        Self::Detailed(DetailedDependency {
            version: None,
            git: None,
            rev: None,
            path: Some(path.into()),
        })
    }
}

impl Manifest {
    /// Parse and validate a manifest from TOML text.
    ///
    /// # Errors
    ///
    /// Returns [`PkgError::Manifest`] for malformed TOML, unknown keys, or invalid
    /// names/versions/requirements. Never panics on user input.
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: Self =
            toml::from_str(content).map_err(|e| PkgError::Manifest(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Read and validate the manifest at `path`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the file cannot be read, or the same errors as [`Self::parse`].
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(io_at(path))?;
        Self::parse(&content)
    }

    /// Check names, versions and dependency specs.
    fn validate(&self) -> Result<()> {
        validate_name(&self.package.name)?;
        parse_version(&self.package.version)?;
        for (name, spec) in self.dependencies.iter().chain(self.dev_dependencies.iter()) {
            spec.source(name)?;
        }
        Ok(())
    }

    /// Serialize the manifest as pretty TOML.
    ///
    /// # Errors
    ///
    /// Returns [`PkgError::Manifest`] if serialization fails.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| PkgError::Manifest(e.to_string()))
    }
}

/// Validate a package or dependency name.
///
/// Names are used as directory names inside registries and project caches, so only ASCII
/// letters, digits, `-` and `_` are allowed.
///
/// # Errors
///
/// Returns [`PkgError::InvalidName`] otherwise.
pub fn validate_name(name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && !name.starts_with(['-', '_'])
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(PkgError::InvalidName(name.to_string()))
    }
}

/// Parse a semver version string.
///
/// # Errors
///
/// Returns [`PkgError::InvalidVersion`] if the text is not valid semver.
pub fn parse_version(value: &str) -> Result<Version> {
    Version::parse(value)
        .map_err(|e| PkgError::InvalidVersion { value: value.to_string(), message: e.to_string() })
}

/// Parse a semver requirement for dependency `name`.
///
/// # Errors
///
/// Returns [`PkgError::InvalidRequirement`] if the text is not a valid requirement.
pub fn parse_req(name: &str, req: &str) -> Result<VersionReq> {
    VersionReq::parse(req).map_err(|e| PkgError::InvalidRequirement {
        name: name.to_string(),
        requirement: req.to_string(),
        message: e.to_string(),
    })
}

/// Edit the `[dependencies]` table of manifest text, preserving everything else.
///
/// `Some(spec)` inserts or replaces `name`; `None` removes it. Comments, ordering and formatting
/// of unrelated content are kept by `toml_edit`. Returns the new text and whether it changed.
///
/// # Errors
///
/// Returns [`PkgError::Manifest`] if the text is not valid TOML or `dependencies` is not a
/// table, and [`PkgError::DependencyNotFound`] when removing a missing dependency.
pub fn edit_dependencies(
    content: &str,
    name: &str,
    spec: Option<&DependencySpec>,
) -> Result<(String, bool)> {
    validate_name(name)?;
    let mut doc: DocumentMut =
        content.parse().map_err(|e: toml_edit::TomlError| PkgError::Manifest(e.to_string()))?;
    if doc.get("dependencies").is_none() {
        if spec.is_none() {
            return Err(PkgError::DependencyNotFound(name.to_string()));
        }
        doc.insert("dependencies", Item::Table(Table::new()));
    }
    let table = doc["dependencies"]
        .as_table_mut()
        .ok_or_else(|| PkgError::Manifest("`dependencies` must be a table".to_string()))?;

    let changed = match spec {
        Some(spec) => {
            let item = spec_to_item(spec);
            let old = table.get(name).map(|i| i.to_string().trim().to_string());
            let new = item.to_string().trim().to_string();
            let changed = old.as_deref() != Some(new.as_str());
            table.insert(name, item);
            changed
        }
        None => {
            if table.remove(name).is_none() {
                return Err(PkgError::DependencyNotFound(name.to_string()));
            }
            true
        }
    };
    Ok((doc.to_string(), changed))
}

/// Add or replace a dependency in the manifest file at `manifest_path`, in place.
///
/// Returns `true` if the file was changed.
///
/// # Errors
///
/// Returns I/O errors, or the errors of [`edit_dependencies`].
pub fn add_dependency(manifest_path: &Path, name: &str, spec: &DependencySpec) -> Result<bool> {
    let content = std::fs::read_to_string(manifest_path).map_err(io_at(manifest_path))?;
    let (new, changed) = edit_dependencies(&content, name, Some(spec))?;
    if changed {
        std::fs::write(manifest_path, new).map_err(io_at(manifest_path))?;
    }
    Ok(changed)
}

/// Remove a dependency from the manifest file at `manifest_path`, in place.
///
/// # Errors
///
/// Returns [`PkgError::DependencyNotFound`] if the dependency is not declared, or I/O errors.
pub fn remove_dependency(manifest_path: &Path, name: &str) -> Result<()> {
    let content = std::fs::read_to_string(manifest_path).map_err(io_at(manifest_path))?;
    let (new, _) = edit_dependencies(&content, name, None)?;
    std::fs::write(manifest_path, new).map_err(io_at(manifest_path))
}

/// Path of the manifest inside a project directory.
pub fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join(MANIFEST_FILE)
}

fn spec_to_item(spec: &DependencySpec) -> Item {
    match spec {
        DependencySpec::Requirement(req) => Item::Value(Value::from(req.as_str())),
        DependencySpec::Detailed(d) => {
            let mut inline = InlineTable::new();
            let fields =
                [("version", &d.version), ("git", &d.git), ("rev", &d.rev), ("path", &d.path)];
            for (key, value) in fields {
                if let Some(v) = value {
                    inline.insert(key, Value::from(v.as_str()));
                }
            }
            Item::Value(Value::InlineTable(inline))
        }
    }
}
