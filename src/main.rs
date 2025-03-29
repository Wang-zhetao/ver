use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
mod version_manager;
use version_manager::{VersionManager, VersionType};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available versions (Node.js or Rust)
    List {
        /// Show only LTS versions
        #[arg(short, long)]
        lts: bool,
        
        /// Version type to list (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Install a specific version (Node.js or Rust)
    Install {
        /// Version to install (e.g., 18.17.0, "latest", or "lts")
        version: String,
        
        /// Version type to install (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Use a specific version (Node.js or Rust)
    Use {
        /// Version to use (e.g., 18.17.0 or an alias)
        version: String,
        
        /// Version type to use (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// List installed versions (Node.js or Rust)
    Installed {
        /// Version type to list (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Remove a specific version (Node.js or Rust)
    Remove {
        /// Version to remove (e.g., 18.17.0)
        version: String,
        
        /// Version type to remove (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Show current version (Node.js or Rust)
    Current {
        /// Version type to show (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Create an alias for a specific version (Node.js or Rust)
    Alias {
        /// Alias name
        name: String,
        /// Version to create alias for (e.g., 18.17.0)
        version: String,
        
        /// Version type for alias (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// List all aliases (Node.js or Rust)
    Aliases {
        /// Version type for aliases (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Set a local version for the current directory (Node.js or Rust)
    Local {
        /// Version to set locally (e.g., 18.17.0)
        version: String,
        
        /// Version type to set locally (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Execute a command with a specific version (Node.js or Rust)
    Exec {
        /// Version to use (e.g., 18.17.0)
        version: String,
        
        /// Version type to use (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
        
        /// Command and arguments to execute
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Clean cache and unnecessary files
    Clean,
    /// Update the ver tool itself
    SelfUpdate,
    /// Migrate from other version managers (nvm, rustup)
    Migrate {
        /// Source version manager (nvm, rustup)
        source: String,
        
        /// Version type to migrate (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Rust version management commands (alternative syntax)
    #[command(subcommand)]
    Rust(RustCommands),
}

#[derive(Subcommand)]
enum RustCommands {
    /// List all available Rust versions
    List {
        /// Show only stable versions
        #[arg(short, long)]
        stable: bool,
    },
    /// Install a specific Rust version
    Install {
        /// Version to install (e.g., 1.70.0 or "latest")
        version: String,
    },
    /// Use a specific Rust version
    Use {
        /// Version to use (e.g., 1.70.0 or an alias)
        version: String,
    },
    /// List installed Rust versions
    Installed,
    /// Remove a specific Rust version
    Remove {
        /// Version to remove (e.g., 1.70.0)
        version: String,
    },
    /// Show current Rust version
    Current,
    /// Create an alias for a specific Rust version
    Alias {
        /// Alias name
        name: String,
        /// Version to create alias for (e.g., 1.70.0)
        version: String,
    },
    /// List all Rust aliases
    Aliases,
    /// Set a local Rust version for the current directory
    Local {
        /// Version to set locally (e.g., 1.70.0)
        version: String,
    },
    /// Execute a command with a specific Rust version
    Exec {
        /// Version to use (e.g., 1.70.0)
        version: String,
        /// Command and arguments to execute
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Migrate from rustup
    Migrate,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut manager = VersionManager::new()?;
    
    match cli.command {
        Commands::List { lts, type_ } => {
            let version_type = parse_version_type(&type_)?;
            let versions = manager.list_available_versions(lts, version_type).await?;
            
            // 添加版本类型标题
            match version_type {
                VersionType::Node => println!("{}", "Available Node.js Versions:".green().bold()),
                VersionType::Rust => println!("{}", "Available Rust Versions:".yellow().bold()),
            }
            
            for version in versions {
                let version_str = match version_type {
                    VersionType::Node => {
                        if version.lts {
                            format!("{} (LTS)", version.version).green()
                        } else {
                            version.version.green()
                        }
                    },
                    VersionType::Rust => {
                        if version.lts {
                            format!("{} (Stable)", version.version).yellow()
                        } else {
                            version.version.yellow()
                        }
                    },
                };
                println!("{}", version_str);
            }
        }
        Commands::Install { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            let type_color = match version_type {
                VersionType::Node => "Node.js".green().bold(),
                VersionType::Rust => "Rust".yellow().bold(),
            };
            
            if version == "latest" {
                println!("Installing latest {} version...", type_color);
                manager.install_latest(version_type).await?;
            } else if version == "lts" && version_type == VersionType::Node {
                println!("Installing latest LTS {} version...", type_color);
                manager.install_latest_lts(version_type).await?;
            } else {
                println!("Installing {} version {}...", type_color, version.bold());
                manager.install_version(&version, version_type).await?;
            }
        }
        Commands::Use { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            let type_color = match version_type {
                VersionType::Node => "Node.js".green().bold(),
                VersionType::Rust => "Rust".yellow().bold(),
            };
            
            println!("Switching to {} version {}...", type_color, version.bold());
            manager.use_version(&version, version_type)?;
        }
        Commands::Installed { type_ } => {
            let version_type = parse_version_type(&type_)?;
            let versions = manager.list_installed_versions(version_type)?;
            
            // 添加版本类型标题
            match version_type {
                VersionType::Node => println!("{}", "Installed Node.js Versions:".green().bold()),
                VersionType::Rust => println!("{}", "Installed Rust Versions:".yellow().bold()),
            }
            
            if versions.is_empty() {
                println!("No {} versions installed", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                });
                return Ok(());
            }
            
            for version in versions {
                let is_current = version.contains("(current)");
                let version_str = match version_type {
                    VersionType::Node => {
                        if is_current {
                            version.green().bold()
                        } else {
                            version.green()
                        }
                    },
                    VersionType::Rust => {
                        if is_current {
                            version.yellow().bold()
                        } else {
                            version.yellow()
                        }
                    },
                };
                println!("{}", version_str);
            }
        }
        Commands::Remove { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.remove_version(&version, version_type)?;
        }
        Commands::Current { type_ } => {
            let version_type = parse_version_type(&type_)?;
            if let Some(version) = manager.get_current_version(version_type) {
                println!("Current {} version: {}", match version_type {
                    VersionType::Node => "Node.js".green().bold(),
                    VersionType::Rust => "Rust".yellow().bold(),
                }, version);
            } else {
                println!("No active {} version", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                });
            }
        }
        Commands::Alias { name, version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.create_alias(&name, &version, version_type)?;
            println!("Created alias '{}' -> {} version {}", name, match version_type {
                VersionType::Node => "Node.js".green().bold(),
                VersionType::Rust => "Rust".yellow().bold(),
            }, version);
        }
        Commands::Aliases { type_ } => {
            let version_type = parse_version_type(&type_)?;
            let aliases = manager.list_aliases(version_type)?;
            if aliases.is_empty() {
                println!("No aliases defined for {}", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                });
            } else {
                println!("Defined aliases for {}:", match version_type {
                    VersionType::Node => "Node.js".green().bold(),
                    VersionType::Rust => "Rust".yellow().bold(),
                });
                for (alias, version) in aliases {
                    println!("{} -> {}", alias, version);
                }
            }
        }
        Commands::Local { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.set_local_version(&version, version_type)?;
            println!("Set local {} version to {} for the current directory", match version_type {
                VersionType::Node => "Node.js".green().bold(),
                VersionType::Rust => "Rust".yellow().bold(),
            }, version);
        }
        Commands::Exec { version, type_, args } => {
            let version_type = parse_version_type(&type_)?;
            if args.is_empty() {
                println!("No command specified");
                return Ok(());
            }
            
            let command = &args[0];
            let command_args = if args.len() > 1 { &args[1..] } else { &[] };
            
            manager.exec_with_version(&version, command, command_args, version_type)?;
        }
        Commands::Clean => {
            manager.clean()?;
            println!("Cleaned cache and unnecessary files");
        }
        Commands::SelfUpdate => {
            manager.self_update().await?;
            println!("Updated ver to the latest version");
        }
        Commands::Migrate { source, type_ } => {
            let version_type = parse_version_type(&type_)?;
            let count = manager.migrate_from(&source, version_type).await?;
            println!("Migrated {} versions from {}", count, source);
        }
        Commands::Rust(rust_command) => {
            match rust_command {
                RustCommands::List { stable } => {
                    let versions = manager.list_available_rust_versions(stable).await?;
                    if versions.is_empty() {
                        println!("No Rust versions available");
                    } else {
                        println!("Available Rust Versions:");
                        for version in versions {
                            println!("{}", version);
                        }
                    }
                }
                RustCommands::Install { version } => {
                    manager.install_rust_version(&version).await?;
                }
                RustCommands::Use { version } => {
                    // Check if version is an alias
                    if let Some(aliased_version) = manager.get_rust_alias(&version)? {
                        println!("Using alias '{}' -> Rust version {}", version, aliased_version);
                        manager.use_rust_version(&aliased_version)?;
                    } else {
                        manager.use_rust_version(&version)?;
                    }
                }
                RustCommands::Installed => {
                    let versions = manager.list_installed_rust_versions()?;
                    if versions.is_empty() {
                        println!("No Rust versions installed");
                    } else {
                        println!("Installed Rust Versions:");
                        for version in versions {
                            println!("{}", version);
                        }
                    }
                }
                RustCommands::Remove { version } => {
                    manager.remove_rust_version(&version)?;
                }
                RustCommands::Current => {
                    if let Some(version) = manager.get_current_rust_version() {
                        println!("Current Rust version: {}", version);
                    } else {
                        println!("No active Rust version");
                    }
                }
                RustCommands::Alias { name, version } => {
                    manager.create_rust_alias(&name, &version)?;
                    println!("Created alias '{}' -> Rust version {}", name, version);
                }
                RustCommands::Aliases => {
                    let aliases = manager.list_rust_aliases()?;
                    if aliases.is_empty() {
                        println!("No aliases defined for Rust");
                    } else {
                        println!("Defined aliases for Rust:");
                        for (alias, version) in aliases {
                            println!("{} -> {}", alias, version);
                        }
                    }
                }
                RustCommands::Local { version } => {
                    manager.set_local_rust_version(&version)?;
                    println!("Set local Rust version to {} for the current directory", version);
                }
                RustCommands::Exec { version, args } => {
                    if args.is_empty() {
                        println!("No command specified");
                        return Ok(());
                    }
                    
                    let command = &args[0];
                    let command_args = if args.len() > 1 { &args[1..] } else { &[] };
                    
                    manager.exec_with_rust_version(&version, command, command_args)?;
                }
                RustCommands::Migrate => {
                    manager.migrate_from_rustup().await?;
                }
            }
        }
    }

    Ok(())
}

fn parse_version_type(type_: &str) -> Result<VersionType> {
    match type_.to_lowercase().as_str() {
        "node" => Ok(VersionType::Node),
        "rust" => Ok(VersionType::Rust),
        _ => anyhow::bail!("Unsupported version type: {}. Use 'node' or 'rust'.", type_),
    }
}
