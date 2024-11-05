use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use commands::{check_doctor, install_package, list_packages, search_packages, uninstall_package, update_packages};
mod config;
mod commands;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for packages
    Search {
        /// Package name to search for
        query: String,
    },
    /// Install a package
    Install {
        /// Package name to install
        package: String,
    },
    /// List installed packages
    List,
    /// Uninstall a package
    Uninstall {
        /// Package name to remove
        package: String,
    },
    /// Update all packages
    Update,
    /// Check system configuration and dependencies
    Doctor,
}


fn run_command(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute command: {}", cmd))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(anyhow::anyhow!(
            "Command failed: {}\nError: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}


fn check_dependencies() -> Result<()> {
    // Check for required dependencies
    let dependencies = ["fzf", "jq", "git"];
    
    for dep in dependencies {
        if run_command(&format!("which {}", dep)).is_err() {
            return Err(anyhow::anyhow!(
                "Required dependency '{}' not found. Please install it first.", 
                dep
            ));
        }
    }
    
    Ok(())
}



fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check dependencies before proceeding
    check_dependencies()?;
    
    let config = config::Config::load()?;

    match cli.command {
        Commands::Search { query } => search_packages(&config, &query),
        Commands::Install { package } => install_package(&config, &package, None),
        Commands::List => list_packages(&config),
        Commands::Uninstall { package } => uninstall_package(&config, &package),
        Commands::Update => update_packages(&config),
        Commands::Doctor => check_doctor(&config),
    }
}
