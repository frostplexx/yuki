use anyhow::{Context, Result};
use colored::*;
use std::{fs, process::Command};
use crate::{commands::search::search_package, config::Config};

pub fn check_doctor(config: &Config) -> Result<()> {
    println!("{}", "==> Checking configuration...".bright_blue());
    
    // Check config file paths
    check_config_paths(config)?;
    
    // Check required commands
    check_commands()?;
    
    // Check git repository
    check_git_repo(config)?;
    
    // Test search functionality
    check_search(config)?;
    
    // Test package list parsing
    check_package_lists(config)?;

    println!("\n{}", "Everything looks good! 🎉".green());
    Ok(())
}

fn check_config_paths(config: &Config) -> Result<()> {
    println!("\n{}", "Checking configuration paths:".bright_blue());
    
    // Check Linux packages path
    let linux_path = config.get_expanded_path(&config.linux_packages_path)?;
    print!("Linux packages path ({}): ", linux_path.display());
    if linux_path.exists() {
        println!("{}", "✓".green());
    } else {
        println!("{}", "⨯ File not found".red());
    }
    
    // Check Darwin packages path
    let darwin_path = config.get_expanded_path(&config.darwin_packages_path)?;
    print!("Darwin packages path ({}): ", darwin_path.display());
    if darwin_path.exists() {
        println!("{}", "✓".green());
    } else {
        println!("{}", "⨯ File not found".red());
    }
    
    // Check Homebrew packages path
    let homebrew_path = config.get_expanded_path(&config.homebrew_packages_path)?;
    print!("Homebrew packages path ({}): ", homebrew_path.display());
    if homebrew_path.exists() {
        println!("{}", "✓".green());
    } else {
        println!("{}", "⨯ File not found".red());
    }

    Ok(())
}

fn check_commands() -> Result<()> {
    println!("\n{}", "Checking required commands:".bright_blue());
    
    let commands = vec![
        ("git", "Required for version control"),
        ("nix", "Required for package management"),
        ("brew", "Required for Homebrew package management (macOS only)"),
        ("make", "Required for running installation commands"),
    ];
    
    for (cmd, description) in commands {
        print!("{:<10} ({}): ", cmd, description);
        match Command::new("which")
            .arg(cmd)
            .output() 
        {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", "✓".green());
                } else {
                    println!("{}", "⨯ Not found".red());
                }
            },
            Err(_) => {
                println!("{}", "⨯ Failed to check".red());
            }
        }
    }
    
    Ok(())
}

fn check_git_repo(config: &Config) -> Result<()> {
    println!("\n{}", "Checking git repository:".bright_blue());
    
    let config_dir = config.get_expanded_path(&config.darwin_packages_path)?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .to_path_buf();
    
    print!("Git repository status: ");
    
    let status = Command::new("git")
        .args(&["status"])
        .current_dir(&config_dir)
        .output();
        
    match status {
        Ok(output) => {
            if output.status.success() {
                println!("{}", "✓".green());
                
                // Check for uncommitted changes
                print!("Checking for uncommitted changes: ");
                let changes = Command::new("git")
                    .args(&["status", "--porcelain"])
                    .current_dir(&config_dir)
                    .output()?;
                    
                if changes.stdout.is_empty() {
                    println!("{}", "✓ Working directory clean".green());
                } else {
                    println!("{}", "! Uncommitted changes present".yellow());
                }
            } else {
                println!("{}", "⨯ Not a git repository".red());
            }
        },
        Err(_) => {
            println!("{}", "⨯ Failed to check git status".red());
        }
    }
    
    Ok(())
}

fn check_search(config: &Config) -> Result<()> {
    println!("\n{}", "Testing search functionality:".bright_blue());
    
    print!("Searching for a test package: ");
    match search_package(config, "git") {
        Ok(_) => {
            println!("{}", "✓".green());
        },
        Err(e) => {
            println!("{} ({})", "⨯ Search failed".red(), e);
        }
    }
    
    Ok(())
}

fn check_package_lists(config: &Config) -> Result<()> {
    println!("\n{}", "Checking package lists:".bright_blue());
    
    // Check Nix packages
    let packages_path = if cfg!(target_os = "macos") {
        config.get_expanded_path(&config.darwin_packages_path)?
    } else {
        config.get_expanded_path(&config.linux_packages_path)?
    };

    print!("Reading Nix packages: ");
    match fs::read_to_string(&packages_path) {
        Ok(content) => {
            match nix_editor::read::getarrvals(&content, "environment.systemPackages") {
                Ok(packages) => {
                    println!("{} ({} packages found)", "✓".green(), packages.len());
                },
                Err(e) => {
                    println!("{} ({})", "⨯ Failed to parse".red(), e);
                }
            }
        },
        Err(e) => {
            println!("{} ({})", "⨯ Failed to read file".red(), e);
        }
    }
    
    // Check Homebrew packages on macOS
    if cfg!(target_os = "macos") {
        let homebrew_path = config.get_expanded_path(&config.homebrew_packages_path)?;
        print!("Reading Homebrew formulae: ");
        match fs::read_to_string(&homebrew_path) {
            Ok(content) => {
                match nix_editor::read::getarrvals(&content, "homebrew.brews") {
                    Ok(packages) => {
                        println!("{} ({} formulae found)", "✓".green(), packages.len());
                    },
                    Err(e) => {
                        println!("{} ({})", "⨯ Failed to parse".red(), e);
                    }
                }
            },
            Err(e) => {
                println!("{} ({})", "⨯ Failed to read file".red(), e);
            }
        }

        print!("Reading Homebrew casks: ");
        match fs::read_to_string(&homebrew_path) {
            Ok(content) => {
                match nix_editor::read::getarrvals(&content, "homebrew.casks") {
                    Ok(packages) => {
                        println!("{} ({} casks found)", "✓".green(), packages.len());
                    },
                    Err(e) => {
                        println!("{} ({})", "⨯ Failed to parse".red(), e);
                    }
                }
            },
            Err(e) => {
                println!("{} ({})", "⨯ Failed to read file".red(), e);
            }
        }
    }

    Ok(())
}
