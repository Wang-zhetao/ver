use anyhow::Result;
use clap::{Parser, Subcommand};
mod version_manager;
use version_manager::VersionManager;
use std::path::PathBuf;

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
    },
    /// Install a specific Node.js version
    Install {
        /// Version to install (e.g., 18.17.0, "latest", or "lts")
        version: String,
    },
    /// Use a specific Node.js version
    Use {
        /// Version to use (e.g., 18.17.0 or an alias)
        version: String,
    },
    /// List installed Node.js versions
    Installed,
    /// Remove a specific Node.js version
    Remove {
        /// Version to remove (e.g., 18.17.0)
        version: String,
    },
    /// Show current Node.js version
    Current,
    /// Create an alias for a specific Node.js version
    Alias {
        /// Alias name
        name: String,
        /// Version to create alias for (e.g., 18.17.0)
        version: String,
    },
    /// List all aliases
    Aliases,
    /// Set a local Node.js version for the current directory
    Local {
        /// Version to set locally (e.g., 18.17.0)
        version: String,
    },
    /// Execute a command with a specific Node.js version
    Exec {
        /// Version to use (e.g., 18.17.0)
        version: String,
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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut manager = VersionManager::new()?;
    
    match cli.command {
        Commands::List { lts } => {
            let versions = manager.list_available_versions(lts).await?;
            for version in versions {
                println!("{} {}", version.version, if version.lts { "(LTS)" } else { "" });
            }
        }
        Commands::Install { version } => {
            if version == "latest" {
                println!("Installing latest Node.js version...");
                manager.install_latest().await?;
            } else if version == "lts" {
                println!("Installing latest LTS Node.js version...");
                manager.install_latest_lts().await?;
            } else {
                manager.install_version(&version).await?;
            }
        }
        Commands::Use { version } => {
            // Check if version is an alias
            if let Some(aliased_version) = manager.get_alias(&version)? {
                println!("Using alias '{}' -> Node.js version {}", version, aliased_version);
                manager.use_version(&aliased_version)?;
            } else {
                manager.use_version(&version)?;
            }
        }
        Commands::Installed => {
            let versions = manager.list_installed_versions()?;
            if versions.is_empty() {
                println!("No Node.js versions installed");
            } else {
                for version in versions {
                    println!("{}", version);
                }
            }
        }
        Commands::Remove { version } => {
            manager.remove_version(&version)?;
        }
        Commands::Current => {
            if let Some(version) = manager.get_current_version() {
                println!("Current Node.js version: {}", version);
            } else {
                println!("No active Node.js version");
            }
        }
        Commands::Alias { name, version } => {
            manager.create_alias(&name, &version)?;
            println!("Created alias '{}' -> Node.js version {}", name, version);
        }
        Commands::Aliases => {
            let aliases = manager.list_aliases()?;
            if aliases.is_empty() {
                println!("No aliases defined");
            } else {
                println!("Defined aliases:");
                for (alias, version) in aliases {
                    println!("{} -> {}", alias, version);
                }
            }
        }
        Commands::Local { version } => {
            manager.set_local_version(&version)?;
            println!("Set local Node.js version to {} for the current directory", version);
        }
        Commands::Exec { version, args } => {
            if args.is_empty() {
                println!("No command specified");
                return Ok(());
            }
            
            let command = &args[0];
            let command_args = if args.len() > 1 { &args[1..] } else { &[] };
            
            manager.exec_with_version(&version, command, command_args)?;
        }
        Commands::Clean => {
            manager.clean()?;
            println!("Cleaned cache and unnecessary files");
        }
        Commands::SelfUpdate => {
            manager.self_update().await?;
            println!("Updated ver to the latest version");
        }
        Commands::Migrate { source } => {
            let count = manager.migrate_from(&source).await?;
            println!("Migrated {} versions from {}", count, source);
        }
    }

    Ok(())
} 
