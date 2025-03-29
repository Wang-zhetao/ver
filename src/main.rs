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

#[derive(Debug, Subcommand)]
enum Commands {
    /// List available versions (Node.js or Rust)
    #[clap(alias = "ls")]
    List {
        /// Show only LTS versions
        #[clap(long)]
        lts: bool,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Install a specific version (Node.js or Rust)
    #[clap(alias = "i")]
    Install {
        /// Version to install (e.g., 16.13.0, latest, lts)
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Use a specific version (Node.js or Rust)
    #[clap(alias = "u")]
    Use {
        /// Version to use (e.g., 16.13.0, latest, lts)
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// List installed versions (Node.js or Rust)
    Installed {
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Remove a specific version (Node.js or Rust)
    #[clap(alias = "rm")]
    Remove {
        /// Version to remove
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Show current version (Node.js or Rust)
    Current {
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Create an alias for a version (Node.js or Rust)
    Alias {
        /// Alias name
        name: String,
        
        /// Version to alias
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// List all aliases (Node.js or Rust)
    Aliases {
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Set local version for current directory (Node.js or Rust)
    Local {
        /// Version to set locally
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Execute a command with a specific version (Node.js or Rust)
    Exec {
        /// Version to use
        version: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
        
        /// Command and arguments to execute
        #[clap(last = true)]
        args: Vec<String>,
    },
    
    /// Clean cache and temporary files
    Clean,
    
    /// Update ver itself
    SelfUpdate,
    
    /// Migrate from other version managers (nvm, rustup)
    Migrate {
        /// Source to migrate from (nvm, n, rustup)
        source: String,
        
        /// Version type (node or rust)
        #[clap(short, long, default_value = "node")]
        type_: String,
    },
    
    /// Rust version management commands (alternative syntax)
    #[clap(subcommand)]
    Rust(RustCommands),
    
    /// Python version management commands (alternative syntax)
    #[clap(subcommand)]
    Python(PythonCommands),
    
    /// Go version management commands (alternative syntax)
    #[clap(subcommand)]
    Go(GoCommands),
}

#[derive(Debug, Subcommand)]
enum RustCommands {
    /// List available Rust versions
    #[clap(alias = "ls")]
    List {
        /// Show only stable versions
        #[clap(long)]
        stable: bool,
    },
    
    /// Install a specific Rust version
    #[clap(alias = "i")]
    Install {
        /// Version to install (e.g., 1.85.0, latest, stable)
        version: String,
    },
    
    /// Use a specific Rust version
    #[clap(alias = "u")]
    Use {
        /// Version to use (e.g., 1.85.0, latest, stable)
        version: String,
    },
    
    /// List installed Rust versions
    Installed,
    
    /// Remove a specific Rust version
    #[clap(alias = "rm")]
    Remove {
        /// Version to remove
        version: String,
    },
    
    /// Show current Rust version
    Current,
    
    /// Create an alias for a Rust version
    Alias {
        /// Alias name
        name: String,
        
        /// Version to alias
        version: String,
    },
    
    /// List all Rust aliases
    Aliases,
    
    /// Set local Rust version for current directory
    Local {
        /// Version to set locally
        version: String,
    },
    
    /// Execute a command with a specific Rust version
    Exec {
        /// Version to use
        version: String,
        
        /// Command and arguments to execute
        #[clap(last = true)]
        args: Vec<String>,
    },
    
    /// Migrate from other Rust version managers (rustup)
    Migrate {
        /// Source to migrate from (rustup)
        source: String,
    },
}

#[derive(Debug, Subcommand)]
enum PythonCommands {
    /// List available Python versions
    #[clap(alias = "ls")]
    List {
        /// Show only stable versions
        #[clap(long)]
        stable: bool,
    },
    
    /// Install a specific Python version
    #[clap(alias = "i")]
    Install {
        /// Version to install (e.g., 3.12.0, latest)
        version: String,
    },
    
    /// Use a specific Python version
    #[clap(alias = "u")]
    Use {
        /// Version to use (e.g., 3.12.0, latest)
        version: String,
    },
    
    /// List installed Python versions
    Installed,
    
    /// Remove a specific Python version
    #[clap(alias = "rm")]
    Remove {
        /// Version to remove
        version: String,
    },
    
    /// Show current Python version
    Current,
    
    /// Create an alias for a Python version
    Alias {
        /// Alias name
        name: String,
        
        /// Version to alias
        version: String,
    },
    
    /// List all Python aliases
    Aliases,
    
    /// Set local Python version for current directory
    Local {
        /// Version to set locally
        version: String,
    },
    
    /// Execute a command with a specific Python version
    Exec {
        /// Version to use
        version: String,
        
        /// Command and arguments to execute
        #[clap(last = true)]
        args: Vec<String>,
    },
    
    /// Migrate from other Python version managers (pyenv)
    Migrate {
        /// Source to migrate from (pyenv)
        source: String,
    },
}

#[derive(Debug, Subcommand)]
enum GoCommands {
    /// List available Go versions
    #[clap(alias = "ls")]
    List {
        /// Show only stable versions
        #[clap(long)]
        stable: bool,
    },
    
    /// Install a specific Go version
    #[clap(alias = "i")]
    Install {
        /// Version to install (e.g., 1.22.0, latest)
        version: String,
    },
    
    /// Use a specific Go version
    #[clap(alias = "u")]
    Use {
        /// Version to use (e.g., 1.22.0, latest)
        version: String,
    },
    
    /// List installed Go versions
    Installed,
    
    /// Remove a specific Go version
    #[clap(alias = "rm")]
    Remove {
        /// Version to remove
        version: String,
    },
    
    /// Show current Go version
    Current,
    
    /// Create an alias for a Go version
    Alias {
        /// Alias name
        name: String,
        
        /// Version to alias
        version: String,
    },
    
    /// List all Go aliases
    Aliases,
    
    /// Set local Go version for current directory
    Local {
        /// Version to set locally
        version: String,
    },
    
    /// Execute a command with a specific Go version
    Exec {
        /// Version to use
        version: String,
        
        /// Command and arguments to execute
        #[clap(last = true)]
        args: Vec<String>,
    },
    
    /// Migrate from other Go version managers (gvm)
    Migrate {
        /// Source to migrate from (gvm)
        source: String,
    },
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
                VersionType::Python => println!("{}", "Available Python Versions:".blue().bold()),
                VersionType::Go => println!("{}", "Available Go Versions:".red().bold()),
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
                    VersionType::Python => {
                        if version.lts {
                            format!("{} (Stable)", version.version).blue()
                        } else {
                            version.version.blue()
                        }
                    },
                    VersionType::Go => {
                        if version.lts {
                            format!("{} (Stable)", version.version).red()
                        } else {
                            version.version.red()
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
                VersionType::Python => "Python".blue().bold(),
                VersionType::Go => "Go".red().bold(),
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
                VersionType::Python => "Python".blue().bold(),
                VersionType::Go => "Go".red().bold(),
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
                VersionType::Python => println!("{}", "Installed Python Versions:".blue().bold()),
                VersionType::Go => println!("{}", "Installed Go Versions:".red().bold()),
            }
            
            if versions.is_empty() {
                println!("No {} versions installed", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                    VersionType::Python => "Python".blue(),
                    VersionType::Go => "Go".red(),
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
                    VersionType::Python => {
                        if is_current {
                            version.blue().bold()
                        } else {
                            version.blue()
                        }
                    },
                    VersionType::Go => {
                        if is_current {
                            version.red().bold()
                        } else {
                            version.red()
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
                    VersionType::Python => "Python".blue().bold(),
                    VersionType::Go => "Go".red().bold(),
                }, version);
            } else {
                println!("No active {} version", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                    VersionType::Python => "Python".blue(),
                    VersionType::Go => "Go".red(),
                });
            }
        }
        Commands::Alias { name, version, type_ } => {
            let version_type = parse_version_type(&type_)?;
            manager.create_alias(&name, &version, version_type)?;
            println!("Created alias '{}' -> {} version {}", name, match version_type {
                VersionType::Node => "Node.js".green().bold(),
                VersionType::Rust => "Rust".yellow().bold(),
                VersionType::Python => "Python".blue().bold(),
                VersionType::Go => "Go".red().bold(),
            }, version);
        }
        Commands::Aliases { type_ } => {
            let version_type = parse_version_type(&type_)?;
            let aliases = manager.list_aliases(version_type)?;
            if aliases.is_empty() {
                println!("No aliases defined for {}", match version_type {
                    VersionType::Node => "Node.js".green(),
                    VersionType::Rust => "Rust".yellow(),
                    VersionType::Python => "Python".blue(),
                    VersionType::Go => "Go".red(),
                });
            } else {
                println!("Defined aliases for {}:", match version_type {
                    VersionType::Node => "Node.js".green().bold(),
                    VersionType::Rust => "Rust".yellow().bold(),
                    VersionType::Python => "Python".blue().bold(),
                    VersionType::Go => "Go".red().bold(),
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
                VersionType::Python => "Python".blue().bold(),
                VersionType::Go => "Go".red().bold(),
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
                        println!("{}", "Available Rust Versions:".yellow().bold());
                        for version in versions {
                            // 检查版本是否为稳定版
                            let is_stable = version.contains("stable") || version.contains("Stable");
                            let version_str = if is_stable {
                                format!("{} (Stable)", version).yellow()
                            } else {
                                version.yellow()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                RustCommands::Install { version } => {
                    println!("Installing Rust version {}...", version.yellow().bold());
                    manager.install_rust_version(&version).await?;
                }
                RustCommands::Use { version } => {
                    // Check if version is an alias
                    if let Some(aliased_version) = manager.get_rust_alias(&version)? {
                        println!("Using alias '{}' -> {} version {}", 
                            version, 
                            "Rust".yellow().bold(), 
                            aliased_version.yellow());
                        manager.use_rust_version(&aliased_version)?;
                    } else {
                        println!("Switching to {} version {}...", 
                            "Rust".yellow().bold(), 
                            version.yellow());
                        manager.use_rust_version(&version)?;
                    }
                }
                RustCommands::Installed => {
                    let versions = manager.list_installed_rust_versions()?;
                    if versions.is_empty() {
                        println!("No {} versions installed", "Rust".yellow());
                    } else {
                        println!("{}", "Installed Rust Versions:".yellow().bold());
                        for version in versions {
                            let is_current = version.contains("(current)");
                            let version_str = if is_current {
                                version.yellow().bold()
                            } else {
                                version.yellow()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                RustCommands::Remove { version } => {
                    println!("Removing {} version {}...", 
                        "Rust".yellow().bold(), 
                        version.yellow());
                    manager.remove_rust_version(&version)?;
                }
                RustCommands::Current => {
                    if let Some(version) = manager.get_current_rust_version() {
                        println!("Current {} version: {}", 
                            "Rust".yellow().bold(), 
                            version.yellow());
                    } else {
                        println!("No active {} version", "Rust".yellow());
                    }
                }
                RustCommands::Alias { name, version } => {
                    manager.create_rust_alias(&name, &version)?;
                    println!("Created alias '{}' -> {} version {}", name, "Rust".yellow().bold(), version);
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
                RustCommands::Migrate { source } => {
                    manager.migrate_from(&source, VersionType::Rust).await?;
                }
            }
        }
        Commands::Python(python_command) => {
            match python_command {
                PythonCommands::List { stable } => {
                    let versions = manager.list_available_python_versions(stable).await?;
                    if versions.is_empty() {
                        println!("No Python versions available");
                    } else {
                        println!("{}", "Available Python Versions:".blue().bold());
                        for version in versions {
                            // 检查版本是否为稳定版
                            let is_stable = version.contains("stable") || version.contains("Stable");
                            let version_str = if is_stable {
                                format!("{} (Stable)", version).blue()
                            } else {
                                version.blue()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                PythonCommands::Install { version } => {
                    println!("Installing Python version {}...", version.blue().bold());
                    manager.install_python_version(&version).await?;
                }
                PythonCommands::Use { version } => {
                    // Check if version is an alias
                    if let Some(aliased_version) = manager.get_python_alias(&version)? {
                        println!("Using alias '{}' -> {} version {}", 
                            version, 
                            "Python".blue().bold(), 
                            aliased_version.blue());
                        manager.use_python_version(&aliased_version)?;
                    } else {
                        println!("Switching to {} version {}...", 
                            "Python".blue().bold(), 
                            version.blue());
                        manager.use_python_version(&version)?;
                    }
                }
                PythonCommands::Installed => {
                    let versions = manager.list_installed_python_versions()?;
                    if versions.is_empty() {
                        println!("No {} versions installed", "Python".blue());
                    } else {
                        println!("{}", "Installed Python Versions:".blue().bold());
                        for version in versions {
                            let is_current = version.contains("(current)");
                            let version_str = if is_current {
                                version.blue().bold()
                            } else {
                                version.blue()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                PythonCommands::Remove { version } => {
                    println!("Removing {} version {}...", 
                        "Python".blue().bold(), 
                        version.blue());
                    manager.remove_python_version(&version)?;
                }
                PythonCommands::Current => {
                    if let Some(version) = manager.get_current_python_version() {
                        println!("Current {} version: {}", 
                            "Python".blue().bold(), 
                            version.blue());
                    } else {
                        println!("No active {} version", "Python".blue());
                    }
                }
                PythonCommands::Alias { name, version } => {
                    manager.create_python_alias(&name, &version)?;
                    println!("Created alias '{}' -> {} version {}", name, "Python".blue().bold(), version);
                }
                PythonCommands::Aliases => {
                    let aliases = manager.list_python_aliases()?;
                    if aliases.is_empty() {
                        println!("No aliases defined for Python");
                    } else {
                        println!("Defined aliases for Python:");
                        for (alias, version) in aliases {
                            println!("{} -> {}", alias, version);
                        }
                    }
                }
                PythonCommands::Local { version } => {
                    manager.set_local_python_version(&version)?;
                    println!("Set local Python version to {} for the current directory", version);
                }
                PythonCommands::Exec { version, args } => {
                    if args.is_empty() {
                        println!("No command specified");
                        return Ok(());
                    }
                    
                    let command = &args[0];
                    let command_args = if args.len() > 1 { &args[1..] } else { &[] };
                    
                    manager.exec_with_python_version(&version, command, command_args)?;
                }
                PythonCommands::Migrate { source: _ } => {
                    manager.migrate_from_pyenv().await?;
                }
            }
        }
        Commands::Go(go_command) => {
            match go_command {
                GoCommands::List { stable } => {
                    let versions = manager.list_available_go_versions(stable).await?;
                    if versions.is_empty() {
                        println!("No Go versions available");
                    } else {
                        println!("{}", "Available Go Versions:".red().bold());
                        for version in versions {
                            // 检查版本是否为稳定版
                            let is_stable = version.contains("stable") || version.contains("Stable");
                            let version_str = if is_stable {
                                format!("{} (Stable)", version).red()
                            } else {
                                version.red()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                GoCommands::Install { version } => {
                    println!("Installing Go version {}...", version.red().bold());
                    manager.install_go_version(&version).await?;
                }
                GoCommands::Use { version } => {
                    // Check if version is an alias
                    if let Some(aliased_version) = manager.get_go_alias(&version)? {
                        println!("Using alias '{}' -> {} version {}", 
                            version, 
                            "Go".red().bold(), 
                            aliased_version.red());
                        manager.use_go_version(&aliased_version)?;
                    } else {
                        println!("Switching to {} version {}...", 
                            "Go".red().bold(), 
                            version.red());
                        manager.use_go_version(&version)?;
                    }
                }
                GoCommands::Installed => {
                    let versions = manager.list_installed_go_versions()?;
                    if versions.is_empty() {
                        println!("No {} versions installed", "Go".red());
                    } else {
                        println!("{}", "Installed Go Versions:".red().bold());
                        for version in versions {
                            let is_current = version.contains("(current)");
                            let version_str = if is_current {
                                version.red().bold()
                            } else {
                                version.red()
                            };
                            println!("{}", version_str);
                        }
                    }
                }
                GoCommands::Remove { version } => {
                    println!("Removing {} version {}...", 
                        "Go".red().bold(), 
                        version.red());
                    manager.remove_go_version(&version)?;
                }
                GoCommands::Current => {
                    if let Some(version) = manager.get_current_go_version() {
                        println!("Current {} version: {}", 
                            "Go".red().bold(), 
                            version.red());
                    } else {
                        println!("No active {} version", "Go".red());
                    }
                }
                GoCommands::Alias { name, version } => {
                    manager.create_go_alias(&name, &version)?;
                    println!("Created alias '{}' -> {} version {}", name, "Go".red().bold(), version);
                }
                GoCommands::Aliases => {
                    let aliases = manager.list_go_aliases()?;
                    if aliases.is_empty() {
                        println!("No aliases defined for Go");
                    } else {
                        println!("Defined aliases for Go:");
                        for (alias, version) in aliases {
                            println!("{} -> {}", alias, version);
                        }
                    }
                }
                GoCommands::Local { version } => {
                    manager.set_local_go_version(&version)?;
                    println!("Set local Go version to {} for the current directory", version);
                }
                GoCommands::Exec { version, args } => {
                    if args.is_empty() {
                        println!("No command specified");
                        return Ok(());
                    }
                    
                    let command = &args[0];
                    let command_args = if args.len() > 1 { &args[1..] } else { &[] };
                    
                    manager.exec_with_go_version(&version, command, command_args)?;
                }
                GoCommands::Migrate { source: _ } => {
                    manager.migrate_from_gvm().await?;
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
        "python" => Ok(VersionType::Python),
        "go" => Ok(VersionType::Go),
        _ => anyhow::bail!("Unsupported version type: {}. Use 'node', 'rust', 'python', or 'go'.", type_),
    }
}
