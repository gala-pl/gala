//! Package manager for Gala.
//!
//! Manages project scaffolding, dependency resolution, registry interaction,
//! and lockfile generation. Uses `gala.toml` as the manifest format
//! (analogous to Cargo.toml) and `gala.lock` for reproducible builds.

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

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
    Remove {
        name: String,
    },
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

/// A Gala package manifest (gala.toml).
#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    #[serde(rename = "package")]
    package: PackageInfo,
    dependencies: Option<HashMap<String, Dependency>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    edition: Option<String>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        git: Option<String>,
        path: Option<String>,
        features: Option<Vec<String>>,
    },
}

/// A resolved dependency lock entry.
#[derive(Debug, Serialize, Deserialize)]
struct LockEntry {
    name: String,
    version: String,
    source: String,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

/// The lockfile (gala.lock).
#[derive(Debug, Serialize, Deserialize)]
struct Lockfile {
    version: u32,
    packages: Vec<LockEntry>,
}

fn read_manifest(path: &PathBuf) -> Result<Manifest, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("cannot read manifest: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("manifest parse error: {e}"))
}

fn write_manifest(path: &PathBuf, manifest: &Manifest) -> Result<(), String> {
    let content = toml::to_string_pretty(manifest).map_err(|e| format!("serialization error: {e}"))?;
    std::fs::write(path, &content).map_err(|e| format!("write error: {e}"))
}

fn resolve_dependencies(manifest: &Manifest) -> Result<Lockfile, String> {
    let mut packages = Vec::new();

    if let Some(deps) = &manifest.dependencies {
        for (name, dep) in deps {
            let (version, source) = match dep {
                Dependency::Simple(v) => (v.clone(), format!("registry+{}", v)),
                Dependency::Detailed { version, git, path, .. } => {
                    if let Some(g) = git {
                        ("0.1.0".to_string(), format!("git+{g}"))
                    } else if let Some(p) = path {
                        ("0.1.0".to_string(), format!("path+{p}"))
                    } else {
                        let v = version.clone().unwrap_or_else(|| "*".to_string());
                        (v.clone(), format!("registry+{v}"))
                    }
                }
            };

            packages.push(LockEntry {
                name: name.clone(),
                version,
                source,
                checksum: None,
                dependencies: vec![],
            });
        }
    }

    Ok(Lockfile {
        version: 1,
        packages,
    })
}

fn init_project(name: &str, is_lib: bool) -> Result<(), String> {
    let dir = PathBuf::from(name);
    std::fs::create_dir_all(dir.join("src")).map_err(|e| format!("cannot create dir: {e}"))?;

    let manifest = Manifest {
        package: PackageInfo {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: Some("2021".to_string()),
            description: None,
            authors: None,
            license: Some("Apache-2.0".to_string()),
        },
        dependencies: Some(HashMap::from([(
            "gala-std".to_string(),
            Dependency::Simple("0.1.0".to_string()),
        )])),
        dev_dependencies: None,
    };

    write_manifest(&dir.join("gala.toml"), &manifest)?;

    let source_file = if is_lib { "lib.gala" } else { "main.gala" };
    let source = if is_lib {
        "// {name}\npub fn hello() -> String {{\n    return \"Hello, Gala!\";\n}}\n"
    } else {
        "// {name}\nfn main() -> Int {{\n    return 0;\n}}\n"
    };
    std::fs::write(dir.join("src").join(source_file), source)
        .map_err(|e| format!("cannot write source: {e}"))?;

    println!("created package '{name}' at ./{name}/");
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        PackageCommands::Init { name, lib } => {
            if let Err(e) = init_project(name, *lib) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
        PackageCommands::Add { name, version, git, path } => {
            let manifest_path = PathBuf::from("gala.toml");
            let mut manifest = read_manifest(&manifest_path).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            let deps = manifest.dependencies.get_or_insert_with(HashMap::new);
            let dep = if let Some(g) = git {
                Dependency::Detailed {
                    version: None,
                    git: Some(g.clone()),
                    path: None,
                    features: None,
                }
            } else if let Some(p) = path {
                Dependency::Detailed {
                    version: None,
                    git: None,
                    path: Some(p.display().to_string()),
                    features: None,
                }
            } else {
                Dependency::Simple(version.clone().unwrap_or_else(|| "*".to_string()))
            };
            deps.insert(name.clone(), dep);
            write_manifest(&manifest_path, &manifest).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            println!("added '{name}' to dependencies");
        }
        PackageCommands::Remove { name } => {
            let manifest_path = PathBuf::from("gala.toml");
            let mut manifest = read_manifest(&manifest_path).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            if let Some(deps) = &mut manifest.dependencies {
                deps.remove(name);
            }
            write_manifest(&manifest_path, &manifest).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            println!("removed '{name}' from dependencies");
        }
        PackageCommands::Update { package } => {
            let manifest_path = PathBuf::from("gala.toml");
            let manifest = read_manifest(&manifest_path).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            let lockfile = resolve_dependencies(&manifest).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            let lock_content = toml::to_string_pretty(&lockfile).unwrap();
            std::fs::write("gala.lock", &lock_content).unwrap_or_else(|e| {
                eprintln!("error: cannot write lockfile: {e}");
                process::exit(1);
            });
            if let Some(pkg) = package {
                println!("updated '{pkg}'");
            } else {
                println!("updated all dependencies");
            }
        }
        PackageCommands::Publish { registry, token } => {
            let _reg = registry.as_deref().unwrap_or("default");
            if token.is_some() {
                log::info!("authenticating with registry");
            }
            println!("package published (simulated)");
        }
        PackageCommands::Install => {
            let manifest_path = PathBuf::from("gala.toml");
            let manifest = read_manifest(&manifest_path).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            let lockfile = resolve_dependencies(&manifest).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            println!(
                "installed {} package(s)",
                lockfile.packages.len()
            );
        }
    }
}
