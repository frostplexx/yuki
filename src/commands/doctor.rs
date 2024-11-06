use anyhow::{Context, Result};
use colored::*;
use std::{fs, process::Command, os::unix::fs::MetadataExt};
use crate::config::Config;

pub fn check_doctor(config: &Config) -> Result<()> {
    println!("{}", "==> Checking configuration...".bright_blue());
    
    // Check config file paths and permissions
    check_config_paths(config)?;
    
    // Check config file contents
    check_config_contents(config)?;
    
    // Check required commands
    check_commands()?;
    
    // Check git repository
    check_git_repo(config)?;
    
    // Test search functionality
    check_search(config)?;

    println!("\n{}", "Everything looks good! 🎉".green());
    Ok(())
}

fn check_config_paths(config: &Config) -> Result<()> {
    println!("\n{}", "Checking configuration paths and permissions:".bright_blue());
    
    // Check Linux packages path
    let system_path = config.get_expanded_path(&config.system_packages_path)?;
    print!("System packages path ({}): ", system_path.display());
    if system_path.exists() {
        println!("{}", "✓".green());
        check_file_permissions(&system_path, "System packages file");
    } else {
        println!("{}", "⨯ File not found".red());
    }
    
    
    // Check Homebrew packages path
    let homebrew_path = config.get_expanded_path(&config.homebrew_packages_path)?;
    print!("Homebrew packages path ({}): ", homebrew_path.display());
    if homebrew_path.exists() {
        println!("{}", "✓".green());
        check_file_permissions(&homebrew_path, "Homebrew packages");
    } else {
        println!("{}", "⨯ File not found".red());
    }

    Ok(())
}

fn check_file_permissions(path: &std::path::Path, file_type: &str) {
    if let Ok(metadata) = path.metadata() {
        let mode = metadata.mode();
        print!("{} permissions: ", file_type);
        
        let readable = mode & 0o444 != 0;
        let writable = mode & 0o222 != 0;
        
        if readable && writable {
            println!("{}", "✓ (read/write)".green());
        } else if readable {
            println!("{}", "! (read-only)".yellow());
        } else if writable {
            println!("{}", "! (write-only)".yellow());
        } else {
            println!("{}", "⨯ (no access)".red());
        }
    } else {
        println!("{}", "⨯ Unable to check permissions".red());
    }
}

fn check_config_contents(config: &Config) -> Result<()> {
    println!("\n{}", "Checking configuration contents:".bright_blue());
    
    // Check Nix packages content
    let packages_path = config.get_expanded_path(&config.system_packages_path)?;

    print!("Validating systemPackages array: ");
    match fs::read_to_string(&packages_path) {
        Ok(content) => {
            match nix_editor::read::getarrvals(&content, "environment.systemPackages") {
                Ok(packages) => {
                    println!("{} ({} packages found)", "✓".green(), packages.len());
                },
                Err(e) => {
                    println!("{} ({})", "⨯ Array not found or invalid".red(), e);
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
        
        print!("Validating homebrew.brews array: ");
        match fs::read_to_string(&homebrew_path) {
            Ok(content) => {
                match nix_editor::read::getarrvals(&content, "homebrew.brews") {
                    Ok(packages) => {
                        println!("{} ({} formulae found)", "✓".green(), packages.len());
                    },
                    Err(e) => {
                        println!("{} ({})", "⨯ Array not found or invalid".red(), e);
                    }
                }
            },
            Err(e) => {
                println!("{} ({})", "⨯ Failed to read file".red(), e);
            }
        }

        print!("Validating homebrew.casks array: ");
        match fs::read_to_string(&homebrew_path) {
            Ok(content) => {
                match nix_editor::read::getarrvals(&content, "homebrew.casks") {
                    Ok(packages) => {
                        println!("{} ({} casks found)", "✓".green(), packages.len());
                    },
                    Err(e) => {
                        println!("{} ({})", "⨯ Array not found or invalid".red(), e);
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

// Rest of the functions (check_commands, check_git_repo, test_nix_search, 
// test_homebrew_search, check_search) remain unchanged

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

// New function to test search without user interaction
fn test_nix_search(query: &str) -> Result<bool> {
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features", "nix-command",
            "--extra-experimental-features", "flakes",
            "search", "nixpkgs", query,
            "--json"
        ])
        .output()
        .context("Failed to execute nix search")?;

    Ok(output.status.success())
}

fn test_homebrew_search(query: &str) -> Result<bool> {
    let output = Command::new("brew")
        .args(["search", query])
        .output()
        .context("Failed to execute brew search")?;
    Ok(output.status.success())
}

fn check_search(_config: &Config) -> Result<()> {
    println!("\n{}", "Testing search functionality:".bright_blue());
    print!("Testing Nix search: ");
    match test_nix_search("git") {
        Ok(true) => {
            println!("{}", "✓".green());
        },
        Ok(false) => {
            println!("{}", "⨯ Search returned no results".red());
        },
        Err(e) => {
            println!("{} ({})", "⨯ Search failed".red(), e);
        }
    }

    if cfg!(target_os = "macos") {
        print!("Testing Homebrew search: ");
        match test_homebrew_search("git") {
            Ok(true) => {
                println!("{}", "✓".green());
            },
            Ok(false) => {
                println!("{}", "⨯ Search returned no results".red());
            },
            Err(e) => {
                println!("{} ({})", "⨯ Search failed".red(), e);
            }
        }
    }
    
    Ok(())
}

