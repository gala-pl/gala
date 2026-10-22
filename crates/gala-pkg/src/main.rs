//! `gala-pkg` command-line interface. All logic lives in the `gala_pkg` library.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use gala_pkg::{commands, DependencySpec, JSON_SCHEMA};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "gala-pkg", version, about = "Gala package manager")]
struct Cli {
    /// Print machine-readable JSON (schema `gala-pkg.v1`) instead of text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: PackageCommands,
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Initialize a new Gala package in a new directory
    Init {
        name: String,
        #[arg(long)]
        lib: bool,
    },
    /// Add a dependency to gala.toml and update gala.lock
    Add {
        name: String,
        /// Semver requirement, e.g. "^1.2" (default "*")
        #[arg(long, conflicts_with_all = ["git", "path"])]
        version: Option<String>,
        /// Git repository URL (recorded in the lockfile, not fetched in v1)
        #[arg(long, conflicts_with = "path")]
        git: Option<String>,
        /// Git revision; requires --git
        #[arg(long, requires = "git")]
        rev: Option<String>,
        /// Local path to another package
        #[arg(long)]
        path: Option<String>,
        /// Local registry directory
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Remove a dependency from gala.toml and update gala.lock
    Remove {
        name: String,
        /// Local registry directory
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Re-resolve dependencies and rewrite gala.lock
    Update {
        /// Only move this dependency to a newer version
        #[arg(long)]
        package: Option<String>,
        /// Local registry directory
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Publish the package to a local registry directory (remote registries are refused)
    Publish {
        /// Local registry directory
        #[arg(long)]
        registry: String,
    },
    /// Install the dependencies recorded in gala.lock
    Install {
        /// Fail if gala.lock does not match gala.toml; never rewrite the lockfile
        #[arg(long)]
        locked: bool,
        /// Local registry directory
        #[arg(long)]
        registry: Option<PathBuf>,
    },
}

impl PackageCommands {
    /// Action name used in JSON output.
    fn action(&self) -> &'static str {
        match self {
            Self::Init { .. } => "init",
            Self::Add { .. } => "add",
            Self::Remove { .. } => "remove",
            Self::Update { .. } => "update",
            Self::Publish { .. } => "publish",
            Self::Install { .. } => "install",
        }
    }
}

fn run(command: PackageCommands, project: &Path) -> gala_pkg::Result<Value> {
    match command {
        PackageCommands::Init { name, lib } => commands::init_project(project, &name, lib),
        PackageCommands::Add { name, version, git, rev, path, registry } => {
            let spec = if let Some(url) = git {
                DependencySpec::git(url, rev)
            } else if let Some(p) = path {
                DependencySpec::path(p)
            } else {
                DependencySpec::requirement(version.unwrap_or_else(|| "*".to_string()))
            };
            commands::add(project, &name, &spec, registry.as_deref())
        }
        PackageCommands::Remove { name, registry } => {
            commands::remove(project, &name, registry.as_deref())
        }
        PackageCommands::Update { package, registry } => {
            commands::update(project, package.as_deref(), registry.as_deref())
        }
        PackageCommands::Publish { registry } => commands::publish(project, &registry),
        PackageCommands::Install { locked, registry } => {
            commands::install(project, registry.as_deref(), locked)
        }
    }
}

fn print_human(value: &Value) {
    let action = value["action"].as_str().unwrap_or("ok");
    match action {
        "init" => println!(
            "created package '{}' at {}",
            value["package"].as_str().unwrap_or_default(),
            value["path"].as_str().unwrap_or_default()
        ),
        "publish" => println!(
            "published {}@{} to {}",
            value["package"].as_str().unwrap_or_default(),
            value["version"].as_str().unwrap_or_default(),
            value["registry"].as_str().unwrap_or_default()
        ),
        "install" => println!(
            "installed {} package(s)",
            value["installed"].as_array().map_or(0, |a| a.len())
        ),
        other => println!(
            "{other}: manifest {}, lockfile {}",
            if value["manifest_changed"].as_bool().unwrap_or(false) {
                "changed"
            } else {
                "unchanged"
            },
            if value["lock_changed"].as_bool().unwrap_or(false) { "changed" } else { "unchanged" },
        ),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let action = cli.command.action();
    let project = PathBuf::from(".");
    match run(cli.command, &project) {
        Ok(value) => {
            if cli.json {
                println!("{value}");
            } else {
                print_human(&value);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            if cli.json {
                let value = json!({
                    "schema": JSON_SCHEMA,
                    "action": action,
                    "ok": false,
                    "error": err.to_string(),
                });
                println!("{value}");
            } else {
                eprintln!("error: {err}");
            }
            ExitCode::FAILURE
        }
    }
}
