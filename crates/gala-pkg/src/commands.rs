//! Project-level commands: `init`, `add`, `remove`, `update`, `install` and `publish`.
//!
//! Every mutating command returns a [`serde_json::Value`] with the stable `gala-pkg.v1` schema so
//! the CLI can print it with `--json`. Commands validate everything before writing, so a failed
//! resolution leaves `gala.toml` and `gala.lock` untouched.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::error::{io_at, PkgError, Result};
use crate::lockfile::{LockedPackage, Lockfile, LOCKFILE_NAME};
use crate::manifest::{
    edit_dependencies, manifest_path, parse_version, validate_name, DependencySpec, Manifest,
    PackageInfo,
};
use crate::registry::{
    checksum_dir, collect_files, copy_file, copy_tree, is_remote_registry, IndexEntry,
    LocalRegistry,
};
use crate::resolver::{resolve, ResolveOptions};

/// Schema identifier carried by every JSON output.
pub const JSON_SCHEMA: &str = "gala-pkg.v1";

/// Lockfile source string for packages taken from a registry.
const REGISTRY_SOURCE: &str = "registry";

/// Create a new package directory `parent/name` with a manifest and a starter source file.
///
/// # Errors
///
/// Returns [`PkgError::InvalidName`] for invalid names, and an error if the directory already
/// contains a manifest.
pub fn init_project(parent: &Path, name: &str, is_lib: bool) -> Result<Value> {
    validate_name(name)?;
    let dir = parent.join(name);
    let manifest_file = manifest_path(&dir);
    if manifest_file.exists() {
        return Err(PkgError::Unsupported(format!("`{}` already exists", manifest_file.display())));
    }
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(io_at(&src_dir))?;

    let manifest = Manifest {
        package: PackageInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            description: None,
            authors: None,
            license: Some("Apache-2.0".to_string()),
        },
        dependencies: BTreeMap::new(),
        dev_dependencies: BTreeMap::new(),
    };
    write_text(&manifest_file, &manifest.to_toml_string()?)?;

    // The application runs a Bell pair on the built-in simulator; the library exposes it.
    let (file, source) = if is_lib {
        (
            "lib.gala",
            format!(
                "// {name}\n\nfn bell() -> Measured<Bool> prob {{\n    let a = qubit()\n    let b = qubit()\n    h(a)\n    let (a, b) = cx(a, b)\n    drop(b)\n    measure(a)\n}}\n"
            ),
        )
    } else {
        (
            "main.gala",
            format!(
                "// {name}: a Bell pair on the built-in simulator\n\nfn bell() -> Measured<Bool> prob {{\n    let a = qubit()\n    let b = qubit()\n    h(a)\n    let (a, b) = cx(a, b)\n    drop(b)\n    measure(a)\n}}\n\nfn main() -> Int prob {{\n    print(bell())\n    0\n}}\n"
            ),
        )
    };
    write_text(&src_dir.join(file), &source)?;

    Ok(json!({
        "schema": JSON_SCHEMA,
        "action": "init",
        "package": name,
        "path": dir.display().to_string(),
        "kind": if is_lib { "lib" } else { "bin" },
    }))
}

/// Add or replace dependency `name` in `project_dir/gala.toml`, then update `gala.lock`.
///
/// Locked versions of other packages are preferred, so unrelated packages do not move.
///
/// # Errors
///
/// Returns manifest, registry or resolution errors. Nothing is written when resolution fails.
pub fn add(
    project_dir: &Path,
    name: &str,
    spec: &DependencySpec,
    registry: Option<&Path>,
) -> Result<Value> {
    let path = manifest_path(project_dir);
    let content = read_text(&path)?;
    let (new_text, manifest_changed) = edit_dependencies(&content, name, Some(spec))?;
    let manifest = Manifest::parse(&new_text)?;
    let old = load_lock(project_dir)?;
    let new = resolve_with(&manifest, project_dir, registry, &preferred_from(old.as_ref(), None))?;
    let lock_changed = write_lock_if_changed(project_dir, old.as_ref(), &new)?;
    if manifest_changed {
        write_text(&path, &new_text)?;
    }
    Ok(mutation_json("add", Some(name), manifest_changed, lock_changed, old.as_ref(), &new))
}

/// Remove dependency `name` from `project_dir/gala.toml`, then update `gala.lock`.
///
/// # Errors
///
/// Returns [`PkgError::DependencyNotFound`] if `name` is not declared, or resolution errors.
pub fn remove(project_dir: &Path, name: &str, registry: Option<&Path>) -> Result<Value> {
    let path = manifest_path(project_dir);
    let content = read_text(&path)?;
    let (new_text, manifest_changed) = edit_dependencies(&content, name, None)?;
    let manifest = Manifest::parse(&new_text)?;
    let old = load_lock(project_dir)?;
    let new = resolve_with(&manifest, project_dir, registry, &preferred_from(old.as_ref(), None))?;
    let lock_changed = write_lock_if_changed(project_dir, old.as_ref(), &new)?;
    if manifest_changed {
        write_text(&path, &new_text)?;
    }
    Ok(mutation_json("remove", Some(name), manifest_changed, lock_changed, old.as_ref(), &new))
}

/// Re-resolve dependencies and rewrite `gala.lock`.
///
/// With `package`, only that dependency may move to a newer version; everything else keeps its
/// locked version when still valid. Without it, every dependency is resolved to its highest
/// matching version.
///
/// # Errors
///
/// Returns [`PkgError::DependencyNotFound`] if `package` is not a declared dependency.
pub fn update(project_dir: &Path, package: Option<&str>, registry: Option<&Path>) -> Result<Value> {
    let manifest = Manifest::load(&manifest_path(project_dir))?;
    if let Some(p) = package {
        validate_name(p)?;
        if !manifest.dependencies.contains_key(p) {
            return Err(PkgError::DependencyNotFound(p.to_string()));
        }
    }
    let old = load_lock(project_dir)?;
    let new =
        resolve_with(&manifest, project_dir, registry, &preferred_from(old.as_ref(), package))?;
    let lock_changed = write_lock_if_changed(project_dir, old.as_ref(), &new)?;
    Ok(mutation_json("update", package, false, lock_changed, old.as_ref(), &new))
}

/// Install the dependencies recorded in `gala.lock`.
///
/// Without `locked`, the lockfile is regenerated if the manifest changed. With `locked`, the
/// lockfile must already match the manifest exactly, otherwise the call fails and nothing is
/// written. Registry packages are verified against their checksums and copied to
/// `.gala/packages/<name>/<version>` inside the project.
///
/// # Errors
///
/// Returns [`PkgError::LockedMismatch`] or [`PkgError::LockfileMissing`] for `--locked` failures,
/// [`PkgError::Checksum`] for corrupted registry packages, and resolution errors.
pub fn install(project_dir: &Path, registry: Option<&Path>, locked: bool) -> Result<Value> {
    let manifest = Manifest::load(&manifest_path(project_dir))?;
    let old = load_lock(project_dir)?;
    let (lock, lock_changed) = if locked {
        let existing = old.ok_or(PkgError::LockfileMissing)?;
        let resolved =
            resolve_with(&manifest, project_dir, registry, &preferred_from(Some(&existing), None))?;
        if resolved != existing {
            return Err(PkgError::LockedMismatch);
        }
        (existing, false)
    } else {
        let new =
            resolve_with(&manifest, project_dir, registry, &preferred_from(old.as_ref(), None))?;
        let changed = write_lock_if_changed(project_dir, old.as_ref(), &new)?;
        (new, changed)
    };

    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    for pkg in &lock.packages {
        if pkg.source == REGISTRY_SOURCE {
            installed.push(fetch_registry_package(project_dir, pkg, registry)?);
        } else {
            skipped.push(json!({ "name": pkg.name, "source": pkg.source }));
        }
    }
    Ok(json!({
        "schema": JSON_SCHEMA,
        "action": "install",
        "locked": locked,
        "manifest_changed": false,
        "lock_changed": lock_changed,
        "installed": installed,
        "skipped": skipped,
    }))
}

/// Publish the project in `project_dir` to a local registry directory.
///
/// The package is copied (`gala.toml` and `src/**`) to `packages/<name>/<version>` and appended
/// to `index/<name>.json`. Published versions are immutable. Remote registries are refused.
///
/// # Errors
///
/// Returns [`PkgError::Unsupported`] for URL registries and for packages with path or git
/// dependencies, [`PkgError::AlreadyPublished`] for an existing version, and I/O errors.
pub fn publish(project_dir: &Path, registry: &str) -> Result<Value> {
    if is_remote_registry(registry) {
        return Err(PkgError::Unsupported(format!(
            "publishing to remote registry `{registry}` is not supported in gala-pkg v1; \
             publish to a local registry directory instead"
        )));
    }
    let manifest = Manifest::load(&manifest_path(project_dir))?;
    let mut deps = BTreeMap::new();
    for (name, spec) in &manifest.dependencies {
        match spec.source(name)? {
            crate::manifest::DependencySource::Registry(req) => {
                deps.insert(name.clone(), req.to_string());
            }
            _ => {
                return Err(PkgError::Unsupported(format!(
                    "cannot publish `{}`: dependency `{name}` is a path or git dependency",
                    manifest.package.name
                )))
            }
        }
    }
    let src_dir = project_dir.join("src");
    if !src_dir.is_dir() {
        return Err(PkgError::Manifest(format!(
            "`{}` has no src/ directory",
            project_dir.display()
        )));
    }

    let name = manifest.package.name.clone();
    let version = parse_version(&manifest.package.version)?.to_string();
    let root = Path::new(registry);
    std::fs::create_dir_all(root).map_err(io_at(root))?;
    let reg = LocalRegistry::open(root)?;
    if reg.entries(&name)?.iter().any(|e| e.version == version) {
        return Err(PkgError::AlreadyPublished(format!("{name}@{version}")));
    }
    let dest = reg.package_dir(&name, &version)?;
    if dest.exists() {
        return Err(PkgError::AlreadyPublished(format!("{name}@{version}")));
    }

    // Copy gala.toml plus everything under src/.
    std::fs::create_dir_all(&dest).map_err(io_at(&dest))?;
    copy_file(&manifest_path(project_dir), &dest.join(crate::manifest::MANIFEST_FILE))?;
    copy_tree(&src_dir, &dest.join("src"))?;
    let files = collect_files(&dest, &dest)?;
    let checksum = checksum_dir(&dest)?;

    reg.append_entry(
        &name,
        IndexEntry { version: version.clone(), checksum: checksum.clone(), dependencies: deps },
    )?;
    Ok(json!({
        "schema": JSON_SCHEMA,
        "action": "publish",
        "package": name,
        "version": version,
        "checksum": checksum,
        "registry": registry,
        "files": files,
    }))
}

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(io_at(path))
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content).map_err(io_at(path))
}

fn load_lock(project_dir: &Path) -> Result<Option<Lockfile>> {
    Lockfile::load(&project_dir.join(LOCKFILE_NAME))
}

/// Pin versions from an existing lockfile, optionally excluding one package from the pins.
fn preferred_from(lock: Option<&Lockfile>, exclude: Option<&str>) -> BTreeMap<String, String> {
    lock.map(|l| {
        l.packages
            .iter()
            .filter(|p| Some(p.name.as_str()) != exclude)
            .map(|p| (p.name.clone(), p.version.clone()))
            .collect()
    })
    .unwrap_or_default()
}

fn resolve_with(
    manifest: &Manifest,
    project_dir: &Path,
    registry: Option<&Path>,
    preferred: &BTreeMap<String, String>,
) -> Result<Lockfile> {
    let reg = registry.map(LocalRegistry::open).transpose()?;
    let opts = ResolveOptions { registry: reg.as_ref(), preferred: preferred.clone() };
    resolve(manifest, project_dir, &opts)
}

/// Write `new` to `gala.lock` unless it equals `old`. Returns whether the file was written.
fn write_lock_if_changed(
    project_dir: &Path,
    old: Option<&Lockfile>,
    new: &Lockfile,
) -> Result<bool> {
    if old == Some(new) {
        return Ok(false);
    }
    new.write(&project_dir.join(LOCKFILE_NAME))?;
    Ok(true)
}

fn fetch_registry_package(
    project_dir: &Path,
    pkg: &LockedPackage,
    registry: Option<&Path>,
) -> Result<Value> {
    validate_name(&pkg.name)?;
    parse_version(&pkg.version)?;
    let registry_path = registry.ok_or_else(|| PkgError::NoRegistry(pkg.name.clone()))?;
    let reg = LocalRegistry::open(registry_path)?;
    let src = reg.package_dir(&pkg.name, &pkg.version)?;
    if !src.is_dir() {
        return Err(PkgError::Registry(format!(
            "`{}@{}` is locked but missing from the registry",
            pkg.name, pkg.version
        )));
    }
    let expected = pkg
        .checksum
        .clone()
        .ok_or_else(|| PkgError::Lockfile(format!("`{}` has no checksum", pkg.name)))?;
    let actual = checksum_dir(&src)?;
    if actual != expected {
        return Err(PkgError::Checksum {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            expected,
            actual,
        });
    }
    let rel = Path::new(".gala").join("packages").join(&pkg.name).join(&pkg.version);
    let dest = project_dir.join(&rel);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(io_at(&dest))?;
    }
    copy_tree(&src, &dest)?;
    Ok(json!({
        "name": pkg.name,
        "version": pkg.version,
        "source": pkg.source,
        "path": rel.display().to_string(),
    }))
}

/// Compute `(added, removed, updated)` arrays describing the change from `old` to `new`.
fn lock_diff(old: Option<&Lockfile>, new: &Lockfile) -> (Value, Value, Value) {
    let index = |l: &Lockfile| -> BTreeMap<String, String> {
        l.packages.iter().map(|p| (p.name.clone(), p.version.clone())).collect()
    };
    let old_map = old.map(index).unwrap_or_default();
    let new_map = index(new);
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut updated = Vec::new();
    for (name, version) in &new_map {
        match old_map.get(name) {
            None => added.push(json!({ "name": name, "version": version })),
            Some(prev) if prev != version => {
                updated.push(json!({ "name": name, "from": prev, "to": version }))
            }
            Some(_) => {}
        }
    }
    for (name, version) in &old_map {
        if !new_map.contains_key(name) {
            removed.push(json!({ "name": name, "version": version }));
        }
    }
    (Value::Array(added), Value::Array(removed), Value::Array(updated))
}

fn mutation_json(
    action: &str,
    package: Option<&str>,
    manifest_changed: bool,
    lock_changed: bool,
    old: Option<&Lockfile>,
    new: &Lockfile,
) -> Value {
    let (added, removed, updated) = lock_diff(old, new);
    json!({
        "schema": JSON_SCHEMA,
        "action": action,
        "package": package,
        "manifest_changed": manifest_changed,
        "lock_changed": lock_changed,
        "added": added,
        "removed": removed,
        "updated": updated,
    })
}
