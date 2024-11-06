use anyhow::{Context, Result};
use colored::*;
use nix_editor::{write, read};
use std::fs;
use crate::config::Config;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

use super::search::{search_package, PackageType};

pub fn install_package(config: &Config, package: &str, package_type: Option<PackageType>) -> Result<()> {
    match package_type {
        Some(pkg_type) => {
            // Direct installation with known package type
            match pkg_type {
                PackageType::Nix => install_nix_package(config, package),
                PackageType::HomebrewFormula => install_homebrew_package(config, package, false),
                PackageType::HomebrewCask => install_homebrew_package(config, package, true),
            }
        }
        None => {
            // If no package type specified, search and let user choose
            let selected = search_package(config, package)?;
            match selected {
                Some(pkg) => install_package(config, &pkg.name, Some(pkg.source)),
                None => Ok(()),
            }
        }
    }
}

fn install_nix_package(config: &Config, package: &str) -> Result<()> {
    let packages_path = config.get_expanded_path(&config.system_packages_path)?;
    if !packages_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found at: {}",
            packages_path.display()
        ));
    }
    
    println!("📦 Installing package: {}", package.bright_blue());
    println!("📄 Using nix file: {}", packages_path.display());

    // Read the current file content
    let file_content = fs::read_to_string(&packages_path)
        .context("Failed to read configuration file")?;

    match read::getarrvals(&file_content, "environment.systemPackages") {
        Ok(packages) => {
            println!("📦 Found {} existing packages", packages.len());
            if packages.iter().any(|p| p.contains(package)) {
                println!("⚠️  Package {} is already installed!", package.yellow());
                return Ok(());
            }
            
            let new_content = write::addtoarr(&file_content, "environment.systemPackages", vec![package.to_string()])
                .map_err(|e| anyhow::anyhow!("Failed to add package to array: {}", e))?;

            fs::write(&packages_path, new_content)
                .context("Failed to write configuration file")?;

            handle_post_install(config, package)?;
        },
        Err(read::ReadError::NoAttr) => {
            println!("⚠️  Could not find environment.systemPackages, attempting to initialize...");
            
            let initial_content = format!("
  environment.systemPackages = with pkgs; [
    {}
  ];", package);

            let new_content = write::write(&file_content, "environment.systemPackages", &initial_content)
                .map_err(|e| anyhow::anyhow!("Failed to initialize environment.systemPackages: {}", e))?;

            fs::write(&packages_path, new_content)
                .context("Failed to write configuration file")?;

            handle_post_install(config, package)?;
        },
        Err(e) => {
            println!("❌ Current file content that failed to parse:");
            println!("{}", file_content);
            return Err(anyhow::anyhow!("Failed to read packages: {}", e));
        }
    }

    Ok(())
}

fn install_homebrew_package(config: &Config, package: &str, is_cask: bool) -> Result<()> {
    let packages_path = config.get_expanded_path(&config.homebrew_packages_path)?;

    if !packages_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found at: {}",
            packages_path.display()
        ));
    }

    println!("📦 Installing {} package: {}", if is_cask { "cask" } else { "formula" }, package.bright_blue());
    println!("📄 Using homebrew file: {}", packages_path.display());

    let file_content = fs::read_to_string(&packages_path)
        .context("Failed to read configuration file")?;

    let array_path = if is_cask { "homebrew.casks" } else { "homebrew.brews" };

    // Quote the package name
    let package_str = format!("\"{}\"", package);

    match read::getarrvals(&file_content, array_path) {
        Ok(packages) => {
            println!("📦 Found {} existing packages", packages.len());
            if packages.iter().any(|p| p.trim_matches('"') == package) {
                println!("⚠️  Package {} is already installed!", package.yellow());
                return Ok(());
            }
            
            let new_content = write::addtoarr(&file_content, array_path, vec![package_str])
                .map_err(|e| anyhow::anyhow!("Failed to add package to array: {}", e))?;

            fs::write(&packages_path, new_content)
                .context("Failed to write configuration file")?;

            handle_post_install(config, package)?;
        },
        Err(read::ReadError::NoAttr) => {
            println!("⚠️  Could not find {}, attempting to initialize...", array_path);
            
            let initial_content = format!("
  {} = [
    {}
  ];", array_path, package_str);

            let new_content = write::write(&file_content, array_path, &initial_content)
                .map_err(|e| anyhow::anyhow!("Failed to initialize {}: {}", array_path, e))?;

            fs::write(&packages_path, new_content)
                .context("Failed to write configuration file")?;

            handle_post_install(config, package)?;
        },
        Err(e) => {
            println!("❌ Current file content that failed to parse:");
            println!("{}", file_content);
            return Err(anyhow::anyhow!("Failed to read packages: {}", e));
        }
    }

    Ok(())
}


fn handle_post_install(config: &Config, package: &str) -> Result<()> {
    println!("✨ Successfully added {}", package.green());
    
    // Get home directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    // Handle git operations if auto_commit is enabled
    if config.auto_commit {
        let commit_msg = config.install_message.replace("<package>", package);
        
        if let Ok(_) = Command::new("git")
            .args(&["add", "."])
            .current_dir(&home_dir.join("dotfiles"))
            .output() {
            if let Ok(_) = Command::new("git")
                .args(&["commit", "-m", &commit_msg])
                .current_dir(&home_dir.join("dotfiles"))
                .output() {
                println!("📝 Changes committed to git");

                if config.auto_push {
                    if let Ok(_) = Command::new("git")
                        .args(&["push"])
                        .current_dir(&home_dir.join("dotfiles"))
                        .output() {
                        println!("🚀 Changes pushed to remote");
                    }
                }
            }
        }
    }

    // Run install command
    if !config.install_command.is_empty() {
        let clean_command = config.install_command.trim();
        println!("🔄 Running install command: {}", clean_command.bright_blue());
        
        // Create command with piped output
        let mut child = Command::new("sh")
            .args(&["-c", clean_command])
            .current_dir(&home_dir.join("dotfiles"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn command")?;

        // Handle stdout in real-time
        if let Some(stdout) = child.stdout.take() {
            let stdout_reader = BufReader::new(stdout);
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            }
        }

        // Handle stderr in real-time
        if let Some(stderr) = child.stderr.take() {
            let stderr_reader = BufReader::new(stderr);
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    eprintln!("{}", line.red());
                }
            }
        }

        // Wait for the command to complete and check status
        match child.wait() {
            Ok(status) => {
                if status.success() {
                    println!("✅ Install command completed successfully");
                } else {
                    return Err(anyhow::anyhow!("Install command failed with status: {}", status));
                }
            },
            Err(e) => {
                println!("❌ Failed to execute install command: {}", e.to_string().red());
                return Err(anyhow::anyhow!("Failed to execute install command: {}", e));
            }
        }
    }

    Ok(())
}
