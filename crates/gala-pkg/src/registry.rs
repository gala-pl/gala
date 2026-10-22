//! Local directory registry (v1).
//!
//! Layout of a registry directory:
//!
//! ```text
//! <registry>/
//!   index/<name>.json                 # {"name": ..., "versions": [IndexEntry, ...]}
//!   packages/<name>/<version>/        # immutable copy of gala.toml and src/**
//! ```
//!
//! A package's `checksum` is the SHA-256 over its files, sorted by relative path, hashing each
//! path, a separator, the byte length and the contents. Remote (URL) registries are not
//! supported in v1 and are refused.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{io_at, PkgError, Result};
use crate::manifest::{parse_version, validate_name};

/// One published version as recorded in a registry index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Published semver version.
    pub version: String,
    /// SHA-256 of the package directory, hex encoded.
    pub checksum: String,
    /// Registry dependencies: name to semver requirement text.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexFile {
    name: String,
    #[serde(default)]
    versions: Vec<IndexEntry>,
}

/// A registry stored in a local directory.
#[derive(Debug, Clone)]
pub struct LocalRegistry {
    root: PathBuf,
}

/// Returns `true` if `location` names a remote registry (anything with a URL scheme).
pub fn is_remote_registry(location: &str) -> bool {
    location.contains("://")
}

impl LocalRegistry {
    /// Open a registry rooted at a local directory.
    ///
    /// # Errors
    ///
    /// Returns [`PkgError::Unsupported`] for remote URLs and [`PkgError::Registry`] if the path
    /// is not an existing directory.
    pub fn open(root: &Path) -> Result<Self> {
        if is_remote_registry(&root.to_string_lossy()) {
            return Err(PkgError::Unsupported(format!(
                "remote registry `{}` is not supported in gala-pkg v1; use a local directory",
                root.display()
            )));
        }
        if !root.is_dir() {
            return Err(PkgError::Registry(format!("`{}` is not a directory", root.display())));
        }
        Ok(Self { root: root.to_path_buf() })
    }

    /// Root directory of the registry.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Index entries for `name`, in ascending version order. Unknown packages yield an empty
    /// list.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid names, unreadable or malformed index files.
    pub fn entries(&self, name: &str) -> Result<Vec<IndexEntry>> {
        validate_name(name)?;
        let path = self.index_path(name);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io_at(&path)(e)),
        };
        let file: IndexFile = serde_json::from_str(&content)
            .map_err(|e| PkgError::Registry(format!("malformed index for `{name}`: {e}")))?;
        if file.name != name {
            return Err(PkgError::Registry(format!(
                "index file for `{name}` declares package `{}`",
                file.name
            )));
        }
        Ok(file.versions)
    }

    /// Directory holding the published files of `name@version`.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid names or versions.
    pub fn package_dir(&self, name: &str, version: &str) -> Result<PathBuf> {
        validate_name(name)?;
        parse_version(version)?;
        Ok(self.root.join("packages").join(name).join(version))
    }

    fn index_path(&self, name: &str) -> PathBuf {
        self.root.join("index").join(format!("{name}.json"))
    }

    /// Record a new version in the index for `name`, keeping versions sorted.
    pub(crate) fn append_entry(&self, name: &str, entry: IndexEntry) -> Result<()> {
        let mut versions = self.entries(name)?;
        versions.push(entry);
        versions.sort_by(|a, b| {
            let va = parse_version(&a.version).ok();
            let vb = parse_version(&b.version).ok();
            va.cmp(&vb)
        });
        let file = IndexFile { name: name.to_string(), versions };
        let text = serde_json::to_string_pretty(&file)
            .map_err(|e| PkgError::Registry(format!("cannot serialize index: {e}")))?;
        let dir = self.root.join("index");
        std::fs::create_dir_all(&dir).map_err(io_at(&dir))?;
        let path = self.index_path(name);
        std::fs::write(&path, format!("{text}\n")).map_err(io_at(&path))
    }
}

/// Collect the regular files under `dir`, as `/`-separated paths relative to `base`, sorted.
///
/// Symbolic links and other special files are skipped.
pub(crate) fn collect_files(base: &Path, dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    collect_into(base, dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_into(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(io_at(dir))?;
    for entry in entries {
        let entry = entry.map_err(io_at(dir))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(io_at(&path))?;
        if file_type.is_dir() {
            collect_into(base, &path, out)?;
        } else if file_type.is_file() {
            let rel = path.strip_prefix(base).map_err(|_| {
                PkgError::Registry(format!("`{}` is outside `{}`", path.display(), base.display()))
            })?;
            let parts: Vec<String> =
                rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
            out.push(parts.join("/"));
        }
    }
    Ok(())
}

/// SHA-256 checksum (hex) of the files under `dir`.
///
/// # Errors
///
/// Returns I/O errors while reading the files.
pub fn checksum_dir(dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    for rel in collect_files(dir, dir)? {
        let path = dir.join(&rel);
        let bytes = std::fs::read(&path).map_err(io_at(&path))?;
        hasher.update(rel.as_bytes());
        hasher.update([0u8]);
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Copy every regular file under `src` into `dst`, preserving relative paths.
pub(crate) fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    for rel in collect_files(src, src)? {
        copy_file(&src.join(&rel), &dst.join(&rel))?;
    }
    Ok(())
}

/// Copy a single file, creating parent directories as needed.
pub(crate) fn copy_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(io_at(parent))?;
    }
    std::fs::copy(from, to).map(|_| ()).map_err(io_at(from))
}
