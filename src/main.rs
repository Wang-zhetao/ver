use anyhow::Result;
use clap::{Parser, Subcommand};
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
    /// List all available Node.js versions
    List {
        /// Show only LTS versions
        #[arg(short, long)]
        lts: bool,
        
        /// Version type to list (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Install a specific Node.js version
    Install {
        /// Version to install (e.g., 18.17.0, "latest", or "lts")
        version: String,
        
        /// Version type to install (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Use a specific Node.js version
    Use {
        /// Version to use (e.g., 18.17.0 or an alias)
        version: String,
        
        /// Version type to use (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// List installed Node.js versions
    Installed {
        /// Version type to list (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Remove a specific Node.js version
    Remove {
        /// Version to remove (e.g., 18.17.0)
        version: String,
        
        /// Version type to remove (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Show current Node.js version
    Current {
        /// Version type to show (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Create an alias for a specific Node.js version
    Alias {
        /// Alias name
        name: String,
        /// Version to create alias for (e.g., 18.17.0)
        version: String,
        
        /// Version type for alias (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// List all aliases
    Aliases {
        /// Version type for aliases (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Set a local Node.js version for the current directory
    Local {
        /// Version to set locally (e.g., 18.17.0)
        version: String,
        
        /// Version type to set locally (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    /// Execute a command with a specific Node.js version
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
    /// Migrate from other version managers (nvm, n)
    Migrate {
        /// Source version manager (nvm, n)
        source: String,
        
        /// Version type to migrate (node or rust)
        #[arg(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Rust version management commands
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
            for version in versions {
                println!("{} {}", version.version, if version.lts { "(LTS)" } else { "" });
            }
        }
        Commands::Install { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            if version == "latest" {
                println!("Installing latest {} version...", version_type);
                manager.install_latest(version_type).await?;
            } else if version == "lts" && version_type == VersionType::Node {
                println!("Installing latest LTS Node.js version...");
                manager.install_latest_lts(version_type).await?;
            } else {
                manager.install_version(&version, version_type).await?;
            }
        }
        Commands::Use { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            // Check if version is an alias
            if let Some(aliased_version) = manager.get_alias(&version, version_type)? {
                println!("Using alias '{}' -> {} version {}", version, version_type, aliased_version);
                manager.use_version(&aliased_version, version_type)?;
            } else {
                manager.use_version(&version, version_type)?;
            }
        }
        Commands::Installed { type_ } => {
            let version_type = parse_version_type(&type_)?;
            let versions = manager.list_installed_versions(version_type)?;
            if versions.is_empty() {
                println!("No {} versions installed", version_type);
            } else {
                for version in versions {
                    println!("{}", version);
                }
            }
        }
        Commands::Remove { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.remove_version(&version, version_type)?;
        }
        Commands::Current { type_ } => {
            let version_type = parse_version_type(&type_)?;
            if let Some(version) = manager.get_current_version(version_type) {
                println!("Current {} version: {}", version_type, version);
            } else {
                println!("No active {} version", version_type);
            }
        }
        Commands::Alias { name, version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.create_alias(&name, &version, version_type)?;
            println!("Created alias '{}' -> {} version {}", name, version_type, version);
        }
        Commands::Aliases { type_ } => {
            let version_type = parse_version_type(&type_)?;
            let aliases = manager.list_aliases(version_type)?;
            if aliases.is_empty() {
                println!("No aliases defined for {}", version_type);
            } else {
                println!("Defined aliases for {}:", version_type);
                for (alias, version) in aliases {
                    println!("{} -> {}", alias, version);
                }
            }
        }
        Commands::Local { version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.set_local_version(&version, version_type)?;
            println!("Set local {} version to {} for the current directory", version_type, version);
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
                    for version in versions {
                        println!("{}", version);
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
