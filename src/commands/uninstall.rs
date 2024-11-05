use anyhow::{Context, Result};
use colored::*;
use nix_editor::{write, read};
use std::fs;
use skim::{
    prelude::*,
    Skim,
};
use crate::config::Config;

#[derive(Debug)]
struct UninstallOption {
    package: String,
    location: PackageLocation,
}

#[derive(Debug)]
enum PackageLocation {
    Nix,
    HomebrewFormula,
    HomebrewCask,
}

impl UninstallOption {
    fn to_string(&self) -> String {
        match self.location {
            PackageLocation::Nix => format!("{} (nixpkgs)", self.package),
            PackageLocation::HomebrewFormula => format!("{} (homebrew formula)", self.package),
            PackageLocation::HomebrewCask => format!("{} (homebrew cask)", self.package),
        }
    }
}

pub fn uninstall_package(config: &Config, package: &str) -> Result<()> {
    let mut uninstall_options = Vec::new();

    // Check Nix packages
    let nix_path = if cfg!(target_os = "macos") {
        config.get_expanded_path(&config.darwin_packages_path)?
    } else {
        config.get_expanded_path(&config.linux_packages_path)?
    };

    if nix_path.exists() {
        let content = fs::read_to_string(&nix_path)?;
        if let Ok(packages) = read::getarrvals(&content, "environment.systemPackages") {
            if packages.iter().any(|p| p.contains(package)) {
                uninstall_options.push(UninstallOption {
                    package: package.to_string(),
                    location: PackageLocation::Nix,
                });
            }
        }
    }

    // Check Homebrew packages on macOS
    if cfg!(target_os = "macos") {
        let homebrew_path = config.get_expanded_path(&config.homebrew_packages_path)?;
        if homebrew_path.exists() {
            let content = fs::read_to_string(&homebrew_path)?;
            
            // Check formulae
            if let Ok(packages) = read::getarrvals(&content, "homebrew.brews") {
                if packages.iter().any(|p| p.contains(package)) {
                    uninstall_options.push(UninstallOption {
                        package: package.to_string(),
                        location: PackageLocation::HomebrewFormula,
                    });
                }
            }
            
            // Check casks
            if let Ok(packages) = read::getarrvals(&content, "homebrew.casks") {
                if packages.iter().any(|p| p.contains(package)) {
                    uninstall_options.push(UninstallOption {
                        package: package.to_string(),
                        location: PackageLocation::HomebrewCask,
                    });
                }
            }
        }
    }

    if uninstall_options.is_empty() {
        println!("⚠️  Package {} is not installed!", package.yellow());
        return Ok(());
    }

    // If there's more than one option, let the user choose
    let selected_option = if uninstall_options.len() > 1 {
        let items: Vec<String> = uninstall_options.iter()
            .map(|opt| opt.to_string())
            .collect();

        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .multi(false)
            .prompt(Some("Select package to uninstall > "))
            .build()
            .unwrap();

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(std::io::Cursor::new(items.join("\n")));

        match Skim::run_with(&options, Some(items)) {
            Some(output) => {
                if output.is_abort {
                    return Ok(());
                }
                if let Some(selected) = output.selected_items.first() {
                    let idx = uninstall_options.iter()
                        .position(|opt| opt.to_string() == selected.output())
                        .unwrap_or(0);
                    &uninstall_options[idx]
                } else {
                    return Ok(());
                }
            },
            None => return Ok(()),
        }
    } else {
        &uninstall_options[0]
    };

    match selected_option.location {
        PackageLocation::Nix => uninstall_nix_package(config, package),
        PackageLocation::HomebrewFormula => uninstall_homebrew_package(config, package, false),
        PackageLocation::HomebrewCask => uninstall_homebrew_package(config, package, true),
    }
}

fn uninstall_nix_package(config: &Config, package: &str) -> Result<()> {
    let packages_path = if cfg!(target_os = "macos") {
        config.get_expanded_path(&config.darwin_packages_path)?
    } else {
        config.get_expanded_path(&config.linux_packages_path)?
    };

    println!("🗑️  Uninstalling Nix package: {}", package.bright_blue());
    println!("📄 Using configuration file: {}", packages_path.display());

    let file_content = fs::read_to_string(&packages_path)
        .context("Failed to read configuration file")?;

    match write::rmarr(&file_content, "environment.systemPackages", vec![package.to_string()]) {
        Ok(new_content) => {
            fs::write(&packages_path, new_content)
                .context("Failed to write configuration file")?;
            
            handle_post_uninstall(config, package)?;
        },
        Err(e) => {
            println!("❌ Failed to remove {}", package.red());
            println!("Error: {}", e);
        }
    }

    Ok(())
}

fn uninstall_homebrew_package(config: &Config, package: &str, is_cask: bool) -> Result<()> {
    let packages_path = config.get_expanded_path(&config.homebrew_packages_path)?;
    println!("🗑️  Uninstalling Homebrew {}: {}", 
        if is_cask { "cask" } else { "formula" },
        package.bright_blue()
    );
    println!("📄 Using configuration file: {}", packages_path.display());

    let file_content = fs::read_to_string(&packages_path)
        .context("Failed to read configuration file")?;

    let array_path = if is_cask { "homebrew.casks" } else { "homebrew.brews" };

    // First verify the package exists
    match read::getarrvals(&file_content, array_path) {
        Ok(packages) => {
            // Check if package exists (case-sensitive)
            let package_exists = packages.iter().any(|p| {
                let cleaned = p.trim()
                    .trim_matches('"')
                    .trim_end_matches(';')
                    .trim();
                cleaned == package
            });

            if !package_exists {
                println!("⚠️  Package {} is not installed!", package.yellow());
                return Ok(());
            }

            // Create the package string as it appears in the file
            let package_str = if package.contains('@') {
                package.to_string() // Don't quote strings with @ symbol
            } else {
                format!("\"{}\"", package) // Quote normal package names
            };

            match write::rmarr(&file_content, array_path, vec![package_str]) {
                Ok(new_content) => {
                    fs::write(&packages_path, new_content)
                        .context("Failed to write configuration file")?;
                    
                    handle_post_uninstall(config, package)?;
                },
                Err(e) => {
                    println!("❌ Failed to remove {}", package.red());
                    println!("Error: {}", e);
                }
            }
        },
        Err(read::ReadError::NoAttr) => {
            println!("⚠️  No packages found in configuration");
        },
        Err(e) => {
            println!("❌ Failed to read packages: {}", e);
        }
    }

    Ok(())
}

fn handle_post_uninstall(config: &Config, package: &str) -> Result<()> {
    println!("✨ Successfully removed {}", package.green());
    
    let config_dir = config.get_expanded_path(&config.darwin_packages_path)?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .to_path_buf();

    // Handle git operations if auto_commit is enabled
    if config.auto_commit {
        let commit_msg = config.uninstall_message.replace("<package>", package);
        
        if let Ok(_) = std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&config_dir)
            .output() {
            if let Ok(_) = std::process::Command::new("git")
                .args(&["commit", "-m", &commit_msg])
                .current_dir(&config_dir)
                .output() {
                println!("📝 Changes committed to git");

                if config.auto_push {
                    if let Ok(_) = std::process::Command::new("git")
                        .args(&["push"])
                        .current_dir(&config_dir)
                        .output() {
                        println!("🚀 Changes pushed to remote");
                    }
                }
            }
        }
    }

    // Run uninstall command
    if !config.uninstall_command.is_empty() {
        if let Ok(_) = std::process::Command::new("sh")
            .args(&["-c", &config.uninstall_command])
            .current_dir(&config_dir)
            .output() {
            println!("🔄 Uninstall command executed");
        }
    }

    Ok(())
}
