//! Unit and end-to-end tests. Temporary directories live under `std::env::temp_dir()` and are
//! removed when the `TempDir` guard drops.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use semver::Version;
use serde_json::Value;

use crate::commands;
use crate::error::PkgError;
use crate::lockfile::{LockedPackage, Lockfile, LOCKFILE_NAME};
use crate::manifest::{
    add_dependency, remove_dependency, DependencySource, DependencySpec, Manifest, MANIFEST_FILE,
};
use crate::registry::LocalRegistry;
use crate::resolver::{resolve, ResolveOptions};

/// Temporary directory removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir()
            .join(format!("gala-pkg-test-{tag}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn manifest_text(name: &str, version: &str, deps: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n\n[dependencies]\n{deps}")
}

/// Create a package directory `parent/<name>-<version>` with a manifest and a source file.
fn make_pkg(parent: &Path, name: &str, version: &str, deps: &str) -> PathBuf {
    let dir = parent.join(format!("{name}-{version}"));
    write(&dir.join(MANIFEST_FILE), &manifest_text(name, version, deps));
    write(&dir.join("src").join("lib.gala"), &format!("// {name} {version}\npub fn f() {{}}\n"));
    dir
}

/// Create a package and publish it to `registry`.
fn publish_pkg(scratch: &Path, registry: &Path, name: &str, version: &str, deps: &str) {
    let dir = make_pkg(scratch, name, version, deps);
    commands::publish(&dir, registry.to_str().unwrap()).unwrap();
}

fn lock_at(project: &Path) -> Lockfile {
    Lockfile::load(&project.join(LOCKFILE_NAME)).unwrap().unwrap()
}

fn locked_version(lock: &Lockfile, name: &str) -> String {
    lock.get(name).map(|p| p.version.clone()).unwrap_or_default()
}

fn resolve_text(project: &Path, registry: &LocalRegistry) -> crate::Result<Lockfile> {
    let manifest = Manifest::load(&project.join(MANIFEST_FILE))?;
    let opts = ResolveOptions { registry: Some(registry), preferred: BTreeMap::new() };
    resolve(&manifest, project, &opts)
}

// ---------- AC1: manifest parsing ----------

#[test]
fn test_manifest_serialization() {
    let manifest = Manifest::parse(&manifest_text("app", "0.1.0", "dep1 = \"1.0.0\"\n")).unwrap();
    let text = manifest.to_toml_string().unwrap();
    assert!(text.contains("dep1"));
    assert_eq!(Manifest::parse(&text).unwrap(), manifest);
}

#[test]
fn manifest_parses_all_dependency_forms() {
    let text = r#"
[package]
name = "app"
version = "0.1.0"

[dependencies]
greeter = "^1.2"
util = { version = ">=0.3, <0.5" }
libgit = { git = "https://example.org/libgit.git", rev = "abc123" }
local = { path = "../local" }
"#;
    let m = Manifest::parse(text).unwrap();
    assert_eq!(m.package.edition, "2021");
    let src = |n: &str| m.dependencies[n].source(n).unwrap();
    match src("greeter") {
        DependencySource::Registry(req) => {
            assert!(req.matches(&Version::parse("1.5.0").unwrap()));
            assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
        }
        other => panic!("unexpected source {other:?}"),
    }
    assert!(matches!(src("util"), DependencySource::Registry(_)));
    assert_eq!(
        src("libgit"),
        DependencySource::Git {
            url: "https://example.org/libgit.git".to_string(),
            rev: Some("abc123".to_string())
        }
    );
    assert_eq!(src("local"), DependencySource::Path("../local".to_string()));
}

#[test]
fn invalid_manifests_return_errors_without_panicking() {
    let cases = [
        "[package".to_string(),
        "[dependencies]\n".to_string(),
        manifest_text("app", "not-a-version", ""),
        manifest_text("../evil", "0.1.0", ""),
        manifest_text("app", "0.1.0", "x = \"not a req\"\n"),
        manifest_text("app", "0.1.0", "x = { version = \"1\", git = \"u\" }\n"),
        manifest_text("app", "0.1.0", "x = { version = \"1\", rev = \"abc\" }\n"),
        manifest_text("app", "0.1.0", "x = { version = \"1\", features = [\"a\"] }\n"),
        manifest_text("app", "0.1.0", "x = { }\n"),
        manifest_text("app", "0.1.0", "x = { path = \"\" }\n"),
    ];
    for case in &cases {
        let result = Manifest::parse(case);
        assert!(result.is_err(), "expected error for manifest:\n{case}");
    }
}

#[test]
fn add_dependency_preserves_other_content() {
    let scratch = TempDir::new("edit");
    let path = scratch.path().join(MANIFEST_FILE);
    let original = "# App manifest, hand-written\n[package]\nname = \"app\"  # the name\n\
                    version = \"0.1.0\"\n\n# Keep this comment\n[dependencies]\n\
                    existing = \"1.0\"\n\n[profile]\nopt = 3\n";
    write(&path, original);

    assert!(add_dependency(&path, "greeter", &DependencySpec::requirement("^1.0")).unwrap());
    assert!(add_dependency(
        &path,
        "libgit",
        &DependencySpec::git("https://example.org/libgit.git", Some("v2".to_string()))
    )
    .unwrap());
    assert!(add_dependency(&path, "local", &DependencySpec::path("../local")).unwrap());
    // Re-adding the identical spec is a no-op.
    assert!(!add_dependency(&path, "greeter", &DependencySpec::requirement("^1.0")).unwrap());

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("# App manifest, hand-written"));
    assert!(text.contains("# the name"));
    assert!(text.contains("# Keep this comment"));
    assert!(text.contains("[profile]\nopt = 3"));
    assert!(text.contains("existing = \"1.0\""));
    assert!(text.contains("greeter = \"^1.0\""));
    let manifest = Manifest::load(&path).unwrap();
    assert_eq!(manifest.dependencies.len(), 4);

    remove_dependency(&path, "existing").unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(!text.contains("existing ="));
    assert!(text.contains("# Keep this comment"));
    assert!(text.contains("[profile]\nopt = 3"));
    assert!(matches!(remove_dependency(&path, "missing"), Err(PkgError::DependencyNotFound(_))));
}

// ---------- AC2: resolver ----------

#[test]
fn resolver_selects_highest_matching_version() {
    let scratch = TempDir::new("highest");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.4.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "2.0.0", "");
    let reg = LocalRegistry::open(&reg_dir).unwrap();

    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", "greeter = \"^1.0\"\n"));
    let lock = resolve_text(&project, &reg).unwrap();
    assert_eq!(locked_version(&lock, "greeter"), "1.4.0");
}

#[test]
fn resolver_conflict_names_both_requirements() {
    let scratch = TempDir::new("conflict");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "shared", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "shared", "1.5.0", "");
    publish_pkg(scratch.path(), &reg_dir, "other", "1.0.0", "shared = \"^1.5\"\n");
    let reg = LocalRegistry::open(&reg_dir).unwrap();

    let project = scratch.path().join("app");
    write(
        &project.join(MANIFEST_FILE),
        &manifest_text("app", "0.1.0", "shared = \"=1.0.0\"\nother = \"1\"\n"),
    );
    let err = resolve_text(&project, &reg).unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, PkgError::Conflict { .. }), "got {msg}");
    assert!(msg.contains("=1.0.0"), "message should name first requirement: {msg}");
    assert!(msg.contains("^1.5"), "message should name second requirement: {msg}");
}

#[test]
fn resolver_backtracks_to_a_satisfiable_version() {
    let scratch = TempDir::new("backtrack");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "a", "1.0.0", "c = \"^1\"\n");
    publish_pkg(scratch.path(), &reg_dir, "a", "1.1.0", "c = \"^2\"\n");
    publish_pkg(scratch.path(), &reg_dir, "c", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "c", "2.0.0", "");
    let reg = LocalRegistry::open(&reg_dir).unwrap();

    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", "a = \"^1\"\nc = \"^1\"\n"));
    let lock = resolve_text(&project, &reg).unwrap();
    assert_eq!(locked_version(&lock, "a"), "1.0.0");
    assert_eq!(locked_version(&lock, "c"), "1.0.0");
}

#[test]
fn resolver_is_deterministic_and_honours_preferences() {
    let scratch = TempDir::new("determinism");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.1.0", "");
    publish_pkg(scratch.path(), &reg_dir, "zeta", "0.3.0", "greeter = \"1\"\n");
    publish_pkg(scratch.path(), &reg_dir, "alpha", "0.2.0", "greeter = \"1\"\n");
    let reg = LocalRegistry::open(&reg_dir).unwrap();

    let project = scratch.path().join("app");
    write(
        &project.join(MANIFEST_FILE),
        &manifest_text("app", "0.1.0", "zeta = \"0\"\nalpha = \"0\"\n"),
    );
    let first = resolve_text(&project, &reg).unwrap().to_toml_string();
    let second = resolve_text(&project, &reg).unwrap().to_toml_string();
    assert_eq!(first, second);

    let manifest = Manifest::load(&project.join(MANIFEST_FILE)).unwrap();
    let preferred = BTreeMap::from([("greeter".to_string(), "1.0.0".to_string())]);
    let pinned =
        resolve(&manifest, &project, &ResolveOptions { registry: Some(&reg), preferred }).unwrap();
    assert_eq!(locked_version(&pinned, "greeter"), "1.0.0");
}

#[test]
fn resolver_reports_missing_packages_and_registry() {
    let scratch = TempDir::new("missing");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    let reg = LocalRegistry::open(&reg_dir).unwrap();
    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", "nope = \"1\"\n"));
    assert!(matches!(resolve_text(&project, &reg), Err(PkgError::PackageNotFound(_))));

    let manifest = Manifest::load(&project.join(MANIFEST_FILE)).unwrap();
    let no_reg = resolve(&manifest, &project, &ResolveOptions::default());
    assert!(matches!(no_reg, Err(PkgError::NoRegistry(_))));
}

// ---------- AC3: lockfile ----------

#[test]
fn lockfile_round_trips_and_serializes_deterministically() {
    let unsorted = Lockfile {
        version: 1,
        packages: vec![
            LockedPackage {
                name: "zeta".to_string(),
                version: "0.3.0".to_string(),
                source: "registry".to_string(),
                checksum: Some("ab".to_string()),
                dependencies: vec!["mid".to_string(), "alpha".to_string()],
            },
            LockedPackage {
                name: "alpha".to_string(),
                version: "0.2.0".to_string(),
                source: "path+../alpha".to_string(),
                checksum: None,
                dependencies: vec![],
            },
        ],
    };
    let text = unsorted.to_toml_string();
    let parsed = Lockfile::parse(&text).unwrap();
    assert_eq!(parsed.packages[0].name, "alpha");
    assert_eq!(parsed.packages[1].dependencies, vec!["alpha", "mid"]);
    assert_eq!(parsed.to_toml_string(), text);
    assert_eq!(Lockfile::parse(&text).unwrap(), parsed);
}

#[test]
fn lockfile_rejects_bad_versions_and_duplicates() {
    assert!(Lockfile::parse("version = 2\n").is_err());
    assert!(Lockfile::parse("version = \"one\"\n").is_err());
    let dup =
        "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\nsource = \"registry\"\n\n\
               [[package]]\nname = \"a\"\nversion = \"1.0.0\"\nsource = \"registry\"\n";
    assert!(matches!(Lockfile::parse(dup), Err(PkgError::Lockfile(_))));
}

#[test]
fn locked_install_fails_when_manifest_and_lockfile_disagree() {
    let scratch = TempDir::new("locked");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "helper", "1.0.0", "");
    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", "greeter = \"1\"\n"));

    assert!(matches!(
        commands::install(&project, Some(&reg_dir), true),
        Err(PkgError::LockfileMissing)
    ));
    commands::install(&project, Some(&reg_dir), false).unwrap();
    let lock_before = fs::read(project.join(LOCKFILE_NAME)).unwrap();
    commands::install(&project, Some(&reg_dir), true).unwrap();

    // Hand-edit the manifest so it no longer matches the lockfile.
    write(
        &project.join(MANIFEST_FILE),
        &manifest_text("app", "0.1.0", "greeter = \"1\"\nhelper = \"1\"\n"),
    );
    let err = commands::install(&project, Some(&reg_dir), true).unwrap_err();
    assert!(matches!(err, PkgError::LockedMismatch), "got {err}");
    assert_eq!(fs::read(project.join(LOCKFILE_NAME)).unwrap(), lock_before);
}

// ---------- AC4: local registry publish / consume ----------

#[test]
fn publish_refuses_remote_registries_and_opening_them() {
    let scratch = TempDir::new("remote");
    let project = make_pkg(scratch.path(), "sample", "1.0.0", "");
    assert!(matches!(
        commands::publish(&project, "https://registry.example.org"),
        Err(PkgError::Unsupported(_))
    ));
    assert!(matches!(
        LocalRegistry::open(Path::new("https://registry.example.org")),
        Err(PkgError::Unsupported(_))
    ));
}

#[test]
fn publish_refuses_duplicate_versions_and_path_dependencies() {
    let scratch = TempDir::new("dup");
    let reg_dir = scratch.path().join("registry");
    let project = make_pkg(scratch.path(), "sample", "1.0.0", "");
    let reg = reg_dir.to_str().unwrap();
    commands::publish(&project, reg).unwrap();
    assert!(matches!(commands::publish(&project, reg), Err(PkgError::AlreadyPublished(_))));

    let with_path = make_pkg(scratch.path(), "uses-path", "1.0.0", "local = { path = \"../x\" }\n");
    assert!(matches!(commands::publish(&with_path, reg), Err(PkgError::Unsupported(_))));
}

#[test]
fn test_publish_and_consume_end_to_end() {
    let scratch = TempDir::new("e2e");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();

    // Publish a sample package with a transitive dependency.
    publish_pkg(scratch.path(), &reg_dir, "text-utils", "0.2.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.1.0", "text-utils = \"^0.2\"\n");

    // Consume it from a second project.
    let parent = scratch.path().join("consumer");
    fs::create_dir_all(&parent).unwrap();
    commands::init_project(&parent, "app", false).unwrap();
    let project = parent.join("app");
    let added =
        commands::add(&project, "greeter", &DependencySpec::requirement("^1.0"), Some(&reg_dir))
            .unwrap();
    assert_eq!(added["lock_changed"], Value::Bool(true));

    let lock = lock_at(&project);
    assert_eq!(locked_version(&lock, "greeter"), "1.1.0");
    assert_eq!(locked_version(&lock, "text-utils"), "0.2.0");
    assert_eq!(lock.get("greeter").unwrap().dependencies, vec!["text-utils".to_string()]);

    // Re-resolving from scratch yields a byte-identical lockfile.
    let lock_bytes = fs::read(project.join(LOCKFILE_NAME)).unwrap();
    fs::remove_file(project.join(LOCKFILE_NAME)).unwrap();
    commands::update(&project, None, Some(&reg_dir)).unwrap();
    assert_eq!(fs::read(project.join(LOCKFILE_NAME)).unwrap(), lock_bytes);

    // A locked install reproduces the build inputs without touching the lockfile.
    let installed = commands::install(&project, Some(&reg_dir), true).unwrap();
    assert_eq!(installed["locked"], Value::Bool(true));
    assert_eq!(installed["lock_changed"], Value::Bool(false));
    assert_eq!(fs::read(project.join(LOCKFILE_NAME)).unwrap(), lock_bytes);
    assert!(project.join(".gala/packages/greeter/1.1.0/src/lib.gala").is_file());
    assert!(project.join(".gala/packages/text-utils/0.2.0/gala.toml").is_file());
}

#[test]
fn install_detects_tampered_registry_package() {
    let scratch = TempDir::new("tamper");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", "greeter = \"1\"\n"));
    commands::install(&project, Some(&reg_dir), false).unwrap();

    write(&reg_dir.join("packages/greeter/1.0.0/src/lib.gala"), "pub fn evil() {}\n");
    let err = commands::install(&project, Some(&reg_dir), true).unwrap_err();
    assert!(matches!(err, PkgError::Checksum { .. }), "got {err}");
}

// ---------- AC6: JSON output ----------

#[test]
fn test_json_outputs_have_stable_keys() {
    let scratch = TempDir::new("json");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.2.0", "");
    let project = scratch.path().join("app");
    write(&project.join(MANIFEST_FILE), &manifest_text("app", "0.1.0", ""));

    let outputs = [
        (
            "add",
            commands::add(
                &project,
                "greeter",
                &DependencySpec::requirement("1.0.0"),
                Some(&reg_dir),
            )
            .unwrap(),
        ),
        ("update", commands::update(&project, Some("greeter"), Some(&reg_dir)).unwrap()),
        ("install", commands::install(&project, Some(&reg_dir), true).unwrap()),
        ("remove", commands::remove(&project, "greeter", Some(&reg_dir)).unwrap()),
    ];
    for (action, value) in &outputs {
        // The rendered JSON must parse back and carry the stable keys.
        let parsed: Value = serde_json::from_str(&value.to_string()).unwrap();
        assert_eq!(parsed["schema"], "gala-pkg.v1", "{action}");
        assert_eq!(parsed["action"], *action);
        assert!(parsed["lock_changed"].is_boolean(), "{action}");
    }
    let (_, add) = &outputs[0];
    assert_eq!(add["package"], "greeter");
    assert_eq!(add["manifest_changed"], Value::Bool(true));
    assert_eq!(add["added"][0]["name"], "greeter");
    // Bare "1.0.0" is a caret requirement, so the highest 1.x version is locked.
    assert_eq!(add["added"][0]["version"], "1.2.0");
    assert!(add["removed"].is_array() && add["updated"].is_array());

    let (_, update) = &outputs[1];
    assert_eq!(update["package"], "greeter");
    assert_eq!(update["lock_changed"], Value::Bool(false));
    assert_eq!(update["updated"].as_array().map(Vec::len), Some(0));

    let (_, remove) = &outputs[3];
    assert_eq!(remove["removed"][0]["name"], "greeter");
    assert_eq!(remove["manifest_changed"], Value::Bool(true));
}

#[test]
fn update_moves_only_the_named_package() {
    let scratch = TempDir::new("update");
    let reg_dir = scratch.path().join("registry");
    fs::create_dir_all(&reg_dir).unwrap();
    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.0.0", "");
    publish_pkg(scratch.path(), &reg_dir, "helper", "1.0.0", "");
    let project = scratch.path().join("app");
    write(
        &project.join(MANIFEST_FILE),
        &manifest_text("app", "0.1.0", "greeter = \"1\"\nhelper = \"1\"\n"),
    );
    commands::install(&project, Some(&reg_dir), false).unwrap();

    publish_pkg(scratch.path(), &reg_dir, "greeter", "1.1.0", "");
    publish_pkg(scratch.path(), &reg_dir, "helper", "1.1.0", "");
    let value = commands::update(&project, Some("greeter"), Some(&reg_dir)).unwrap();
    let lock = lock_at(&project);
    assert_eq!(locked_version(&lock, "greeter"), "1.1.0");
    assert_eq!(locked_version(&lock, "helper"), "1.0.0");
    assert_eq!(value["updated"].as_array().map(Vec::len), Some(1));
    assert!(matches!(
        commands::update(&project, Some("unknown"), Some(&reg_dir)),
        Err(PkgError::DependencyNotFound(_))
    ));
}

#[test]
fn init_refuses_to_overwrite_and_validates_names() {
    let scratch = TempDir::new("init");
    commands::init_project(scratch.path(), "demo", true).unwrap();
    assert!(scratch.path().join("demo/src/lib.gala").is_file());
    assert!(commands::init_project(scratch.path(), "demo", false).is_err());
    assert!(matches!(
        commands::init_project(scratch.path(), "../bad", false),
        Err(PkgError::InvalidName(_))
    ));
}
