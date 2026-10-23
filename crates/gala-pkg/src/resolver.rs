//! Semver dependency resolver.
//!
//! The resolver is a deterministic depth-first search with backtracking over the candidate
//! versions of each package. Candidates are tried highest version first, except that a
//! preferred version (usually the one already in `gala.lock`) is tried first. Packages are
//! visited in name order, so the same inputs always produce the same lockfile.
//!
//! Root `path` dependencies are read from disk and treated as fixed candidates; their registry
//! dependencies are resolved like any other. Root `git` dependencies are recorded in the
//! lockfile as fixed leaves and are not fetched or resolved in v1.

use std::collections::BTreeMap;
use std::path::Path;

use semver::{Version, VersionReq};

use crate::error::{PkgError, Result};
use crate::lockfile::{LockedPackage, Lockfile};
use crate::manifest::{
    manifest_path, parse_req, parse_version, DependencySource, Manifest, MANIFEST_FILE,
};
use crate::registry::LocalRegistry;

/// Options controlling a resolution.
#[derive(Debug, Default, Clone)]
pub struct ResolveOptions<'a> {
    /// Registry used for registry dependencies. Required when any exist.
    pub registry: Option<&'a LocalRegistry>,
    /// Versions to try first, keyed by package name (typically taken from `gala.lock`).
    pub preferred: BTreeMap<String, String>,
}

/// A concrete version of a package that the solver may select.
#[derive(Debug, Clone)]
struct Candidate {
    version: Version,
    source: String,
    checksum: Option<String>,
    deps: BTreeMap<String, VersionReq>,
}

/// A requirement on a package, with the requirer for error messages.
#[derive(Debug, Clone)]
struct Requirement {
    by: String,
    req: VersionReq,
}

/// Search state: requirements gathered so far and the versions chosen for them.
#[derive(Debug, Clone, Default)]
struct State {
    reqs: BTreeMap<String, Vec<Requirement>>,
    chosen: BTreeMap<String, Candidate>,
}

/// Source of candidates for a resolution.
struct Universe<'a> {
    fixed: BTreeMap<String, Candidate>,
    registry: Option<&'a LocalRegistry>,
    preferred: &'a BTreeMap<String, String>,
}

/// Resolve `manifest` (located in `project_dir`) to a lockfile.
///
/// # Errors
///
/// Returns [`PkgError::PackageNotFound`] for unknown packages, [`PkgError::NoMatchingVersion`]
/// when no version satisfies a requirement, [`PkgError::Conflict`] when two requirements on the
/// same package cannot both hold (naming both), and [`PkgError::NoRegistry`] when a registry
/// dependency exists but no registry was given.
pub fn resolve(manifest: &Manifest, project_dir: &Path, opts: &ResolveOptions) -> Result<Lockfile> {
    let mut fixed = BTreeMap::new();
    let mut state = State::default();

    for (name, spec) in &manifest.dependencies {
        if name == &manifest.package.name {
            return Err(PkgError::Manifest(format!("package `{name}` depends on itself")));
        }
        match spec.source(name)? {
            DependencySource::Registry(req) => {
                push_req(&mut state, name, "root", req);
            }
            DependencySource::Path(p) => {
                let candidate = path_candidate(project_dir, name, &p)?;
                fixed.insert(name.clone(), candidate);
                push_req(&mut state, name, "root", VersionReq::STAR);
            }
            DependencySource::Git { url, rev } => {
                let source = match &rev {
                    Some(r) => format!("git+{url}#{r}"),
                    None => format!("git+{url}"),
                };
                let candidate = Candidate {
                    version: Version::new(0, 0, 0),
                    source,
                    checksum: None,
                    deps: BTreeMap::new(),
                };
                fixed.insert(name.clone(), candidate);
                push_req(&mut state, name, "root", VersionReq::STAR);
            }
        }
    }

    let universe = Universe { fixed, registry: opts.registry, preferred: &opts.preferred };
    let solved = search(&universe, state)?;

    let packages = solved
        .chosen
        .into_iter()
        .map(|(name, c)| LockedPackage {
            name,
            version: c.version.to_string(),
            source: c.source,
            checksum: c.checksum,
            dependencies: c.deps.keys().cloned().collect(),
        })
        .collect();
    Ok(Lockfile::new(packages))
}

/// Read the manifest of a root path dependency and turn it into a fixed candidate.
fn path_candidate(project_dir: &Path, name: &str, path: &str) -> Result<Candidate> {
    let dir = project_dir.join(path);
    let manifest = Manifest::load(&manifest_path(&dir))
        .map_err(|e| PkgError::Manifest(format!("path dependency `{name}` at `{path}`: {e}")))?;
    if manifest.package.name != name {
        return Err(PkgError::Manifest(format!(
            "path dependency `{name}` at `{path}` is package `{}`",
            manifest.package.name
        )));
    }
    let version = parse_version(&manifest.package.version)?;
    let deps = registry_deps(&manifest, MANIFEST_FILE)?;
    Ok(Candidate { version, source: format!("path+{path}"), checksum: None, deps })
}

/// Extract the registry requirements of a path package. Nested path or git dependencies are
/// not supported in v1.
fn registry_deps(manifest: &Manifest, origin: &str) -> Result<BTreeMap<String, VersionReq>> {
    let mut deps = BTreeMap::new();
    for (dep, spec) in &manifest.dependencies {
        match spec.source(dep)? {
            DependencySource::Registry(req) => {
                deps.insert(dep.clone(), req);
            }
            _ => {
                return Err(PkgError::Unsupported(format!(
                    "`{origin}`: nested path or git dependency `{dep}` is not supported in v1"
                )))
            }
        }
    }
    Ok(deps)
}

fn push_req(state: &mut State, name: &str, by: &str, req: VersionReq) {
    state.reqs.entry(name.to_string()).or_default().push(Requirement { by: by.to_string(), req });
}

fn describe(r: &Requirement) -> String {
    format!("`{}` (required by {})", r.req, r.by)
}

/// Build the error for a package whose requirements cannot all be met by `candidates`.
fn conflict_error(name: &str, reqs: &[Requirement], candidates: &[Candidate]) -> PkgError {
    for (i, a) in reqs.iter().enumerate() {
        for b in reqs.iter().skip(i + 1) {
            let joint =
                candidates.iter().any(|c| a.req.matches(&c.version) && b.req.matches(&c.version));
            if !joint {
                return PkgError::Conflict {
                    name: name.to_string(),
                    first: describe(a),
                    second: describe(b),
                };
            }
        }
    }
    match (reqs.first(), reqs.last()) {
        (Some(first), Some(last)) if reqs.len() > 1 => PkgError::Conflict {
            name: name.to_string(),
            first: describe(first),
            second: describe(last),
        },
        (Some(only), _) => PkgError::NoMatchingVersion {
            name: name.to_string(),
            requirement: only.req.to_string(),
        },
        (None, _) => PkgError::PackageNotFound(name.to_string()),
    }
}

impl Universe<'_> {
    /// Candidates for `name`, best first.
    fn candidates(&self, name: &str) -> Result<Vec<Candidate>> {
        if let Some(fixed) = self.fixed.get(name) {
            return Ok(vec![fixed.clone()]);
        }
        let registry = self.registry.ok_or_else(|| PkgError::NoRegistry(name.to_string()))?;
        let mut out = Vec::new();
        for entry in registry.entries(name)? {
            let version = parse_version(&entry.version)?;
            let mut deps = BTreeMap::new();
            for (dep, req) in &entry.dependencies {
                deps.insert(dep.clone(), parse_req(dep, req)?);
            }
            out.push(Candidate {
                version,
                source: "registry".to_string(),
                checksum: Some(entry.checksum),
                deps,
            });
        }
        out.sort_by(|a, b| b.version.cmp(&a.version));
        if let Some(pref) = self.preferred.get(name) {
            if let Some(pos) = out.iter().position(|c| c.version.to_string() == *pref) {
                let chosen = out.remove(pos);
                out.insert(0, chosen);
            }
        }
        Ok(out)
    }
}

/// Depth-first search with backtracking. Returns the first consistent full assignment.
fn search(universe: &Universe, state: State) -> Result<State> {
    let Some(name) = state.reqs.keys().find(|n| !state.chosen.contains_key(*n)).cloned() else {
        return Ok(state);
    };
    let reqs = state.reqs.get(&name).cloned().unwrap_or_default();
    let candidates = universe.candidates(&name)?;
    if candidates.is_empty() {
        return Err(PkgError::PackageNotFound(name));
    }

    let matching: Vec<&Candidate> =
        candidates.iter().filter(|c| reqs.iter().all(|r| r.req.matches(&c.version))).collect();
    if matching.is_empty() {
        return Err(conflict_error(&name, &reqs, &candidates));
    }

    let mut last_err = None;
    for candidate in matching {
        match choose(universe, &state, &name, candidate) {
            Ok(next) => match search(universe, next) {
                Ok(solved) => return Ok(solved),
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| conflict_error(&name, &reqs, &candidates)))
}

/// Select `candidate` for `name`, adding its dependency requirements and checking any
/// already-chosen dependency against them.
fn choose(universe: &Universe, state: &State, name: &str, candidate: &Candidate) -> Result<State> {
    let mut next = state.clone();
    next.chosen.insert(name.to_string(), candidate.clone());
    let by = format!("{name} {}", candidate.version);
    for (dep, req) in &candidate.deps {
        if dep == name {
            return Err(PkgError::Manifest(format!("package `{name}` depends on itself")));
        }
        push_req(&mut next, dep, &by, req.clone());
        if let Some(existing) = next.chosen.get(dep) {
            if !req.matches(&existing.version) {
                let reqs = next.reqs.get(dep).cloned().unwrap_or_default();
                let candidates = universe.candidates(dep)?;
                return Err(conflict_error(dep, &reqs, &candidates));
            }
        }
    }
    Ok(next)
}
