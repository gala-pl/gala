//! Package manager for Gala.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gala-pkg", version, about = "Gala package manager")]
struct Cli {
    #[command(subcommand)]
    command: PackageCommands,
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Initialize a new Gala package
    Init {
        name: String,
        #[arg(long)]
        lib: bool,
    },
    /// Add a dependency
    Add {
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        git: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Remove a dependency
    Remove { name: String },
    /// Update dependencies
    Update {
        #[arg(long)]
        package: Option<String>,
    },
    /// Publish to the registry
    Publish {
        #[arg(long)]
        registry: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    /// Install all dependencies
    Install,
}

/// Gala package manifest (gala.toml).
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "package")]
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    #[serde(default)]
    pub license: Option<String>,
}

fn default_edition() -> String {
    "2021".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        git: Option<String>,
        path: Option<String>,
        features: Option<Vec<String>>,
    },
}

/// Lockfile entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Lockfile (gala.lock).
#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    pub version: u32,
    pub packages: Vec<LockEntry>,
}

/// Read manifest from gala.toml.
pub fn read_manifest(path: &PathBuf) -> Result<Manifest, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("cannot read manifest: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("manifest parse error: {e}"))
}

/// Write manifest to gala.toml.
pub fn write_manifest(path: &PathBuf, manifest: &Manifest) -> Result<(), String> {
    let content =
        toml::to_string_pretty(manifest).map_err(|e| format!("serialization error: {e}"))?;
    std::fs::write(path, content).map_err(|e| format!("write error: {e}"))
}

/// Initialize a new Gala project.
pub fn init_project(name: &str, is_lib: bool) -> Result<(), String> {
    let dir = PathBuf::from(name);
    std::fs::create_dir_all(dir.join("src")).map_err(|e| format!("cannot create dir: {e}"))?;

    let manifest = Manifest {
        package: PackageInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            description: None,
            authors: None,
            license: Some("Apache-2.0".to_string()),
        },
        dependencies: HashMap::from([(
            "gala-std".to_string(),
            Dependency::Simple("0.1.0".to_string()),
        )]),
        dev_dependencies: HashMap::new(),
    };

    write_manifest(&dir.join("gala.toml"), &manifest)?;

    let source_file = if is_lib { "lib.gala" } else { "main.gala" };
    let source = if is_lib {
        format!("// {name}\npub fn hello() -> String {{\n    return \"Hello, Gala!\";\n}}\n")
    } else {
        format!("// {name}\nfn main() -> Int {{\n    return 0;\n}}\n")
    };
    std::fs::write(dir.join("src").join(source_file), source)
        .map_err(|e| format!("cannot write source: {e}"))?;

    println!("created package '{name}' at ./{name}/");
    Ok(())
}

/// Add a dependency to gala.toml.
pub fn add_dependency(
    name: &str,
    version: Option<String>,
    git: Option<String>,
    path: Option<PathBuf>,
) -> Result<(), String> {
    let manifest_path = PathBuf::from("gala.toml");
    let mut manifest = read_manifest(&manifest_path)?;

    let dep = if let Some(g) = git {
        Dependency::Detailed { version: None, git: Some(g), path: None, features: None }
    } else if let Some(p) = path {
        Dependency::Detailed {
            version: None,
            git: None,
            path: Some(p.display().to_string()),
            features: None,
        }
    } else {
        Dependency::Simple(version.unwrap_or_else(|| "*".to_string()))
    };

    manifest.dependencies.insert(name.to_string(), dep);
    write_manifest(&manifest_path, &manifest)?;
    println!("added '{name}' to dependencies");
    Ok(())
}

/// Remove a dependency.
pub fn remove_dependency(name: &str) -> Result<(), String> {
    let manifest_path = PathBuf::from("gala.toml");
    let mut manifest = read_manifest(&manifest_path)?;
    manifest.dependencies.remove(name);
    write_manifest(&manifest_path, &manifest)?;
    println!("removed '{name}' from dependencies");
    Ok(())
}

/// Update dependencies and regenerate lockfile.
pub fn update_dependencies(package: Option<String>) -> Result<(), String> {
    let manifest_path = PathBuf::from("gala.toml");
    let manifest = read_manifest(&manifest_path)?;

    let lockfile = resolve_dependencies(&manifest)?;
    let lock_content =
        toml::to_string_pretty(&lockfile).map_err(|e| format!("serialization error: {e}"))?;
    std::fs::write("gala.lock", lock_content).map_err(|e| format!("write error: {e}"))?;

    if let Some(pkg) = package {
        println!("updated '{pkg}'");
    } else {
        println!("updated all dependencies");
    }
    Ok(())
}

/// Resolve dependencies (placeholder).
fn resolve_dependencies(manifest: &Manifest) -> Result<Lockfile, String> {
    let mut packages = Vec::new();

    for (name, dep) in &manifest.dependencies {
        let (version, source) = match dep {
            Dependency::Simple(v) => (v.clone(), format!("registry+{v}")),
            Dependency::Detailed { version, git, path, .. } => {
                if let Some(g) = git {
                    ("0.1.0".to_string(), format!("git+{g}"))
                } else if let Some(p) = path {
                    ("0.1.0".to_string(), format!("path+{p}"))
                } else {
                    (
                        version.clone().unwrap_or_else(|| "*".to_string()),
                        format!("registry+{}", version.as_deref().unwrap_or("*")),
                    )
                }
            }
        };

        packages.push(LockEntry {
            name: name.clone(),
            version,
            source,
            checksum: None,
            dependencies: Vec::new(),
        });
    }

    Ok(Lockfile { version: 1, packages })
}

/// Publish package to registry.
pub fn publish_package(registry: Option<String>, token: Option<String>) -> Result<(), String> {
    let _reg = registry.unwrap_or_else(|| "default".to_string());
    if let Some(_t) = token {
        // Authenticate
    }
    println!("package published (simulated)");
    Ok(())
}

/// Install dependencies from lockfile.
pub fn install_dependencies() -> Result<(), String> {
    let lockfile_path = PathBuf::from("gala.lock");
    let content = std::fs::read_to_string(&lockfile_path)
        .map_err(|e| format!("cannot read lockfile: {e}"))?;
    let lockfile: Lockfile =
        toml::from_str(&content).map_err(|e| format!("lockfile parse error: {e}"))?;

    println!("installed {} package(s)", lockfile.packages.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest {
            package: PackageInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                edition: "2021".to_string(),
                description: None,
                authors: None,
                license: Some("Apache-2.0".to_string()),
            },
            dependencies: HashMap::from([(
                "gala-std".to_string(),
                Dependency::Simple("0.1.0".to_string()),
            )]),
            dev_dependencies: HashMap::new(),
        };

        let toml = toml::to_string(&manifest).unwrap();
        assert!(toml.contains("gala-std"));
    }

    #[test]
    fn test_lockfile_resolution() {
        let manifest = Manifest {
            package: PackageInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                edition: "2021".to_string(),
                description: None,
                authors: None,
                license: Some("Apache-2.0".to_string()),
            },
            dependencies: HashMap::from([(
                "dep1".to_string(),
                Dependency::Simple("1.0.0".to_string()),
            )]),
            dev_dependencies: HashMap::new(),
        };

        let lockfile = resolve_dependencies(&manifest).unwrap();
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].name, "dep1");
    }
}
