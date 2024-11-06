use anyhow::{Context, Result};
use colored::*;
use nix_editor::read;
use std::fs;
use crate::config::Config;

pub fn clean_package_name(package: &str) -> Option<String> {
    //wtf
    let clean = package
        .trim()
        .trim_matches('"')
        .trim()
        .to_string()
        .replace("}", "")
        .replace("]", "")
        .replace(";", "")
        .replace(" ", "")
        .replace("


", "");

    
        Some(clean.to_string())
}

pub fn list_packages(config: &Config) -> Result<()> {
    // List Nix packages
    let packages_path = config.get_expanded_path(&config.system_packages_path)?;

    let file_content = fs::read_to_string(&packages_path)
        .context("Failed to read configuration file")?;

    println!("==> {}", "Nix Packages".bright_blue());
    match read::getarrvals(&file_content, "environment.systemPackages") {
        Ok(packages) => {
            if packages.is_empty() {
                println!("No Nix packages installed");
            } else {
                // Clean and filter packages
                let packages: Vec<String> = packages.iter()
                    .filter_map(|p| clean_package_name(p))
                    .collect();

                if packages.is_empty() {
                    println!("No Nix packages installed");
                    return Ok(());
                }

                // Calculate the number of columns based on terminal width
                let term_width = term_size::dimensions()
                    .map(|(w, _)| w)
                    .unwrap_or(80);

                // Find the longest package name for padding
                let max_length = packages.iter()
                    .map(|p| p.len())
                    .max()
                    .unwrap_or(0);

                // Calculate number of columns that can fit
                let column_width = max_length + 2; // Add 2 for spacing
                let num_columns = term_width / column_width;
                let num_columns = std::cmp::max(1, num_columns); // Ensure at least 1 column

                // Print packages in columns
                for chunk in packages.chunks(num_columns) {
                    let line = chunk.iter()
                        .map(|p| format!("{:width$}", p, width = column_width))
                        .collect::<Vec<_>>()
                        .join("");
                    println!("{}", line);
                }
            }
        },
        Err(read::ReadError::NoAttr) => {
            println!("No Nix packages installed");
        },
        Err(e) => {
            println!("❌ Failed to read Nix packages: {}", e);
        }
    }

    // List Homebrew packages on macOS
    if cfg!(target_os = "macos") {
        let homebrew_path = config.get_expanded_path(&config.homebrew_packages_path)?;
        if let Ok(content) = fs::read_to_string(&homebrew_path) {
            println!("\n==> {}", "Formulae".bright_blue());
            match read::getarrvals(&content, "homebrew.brews") {
                Ok(packages) => {
                    if packages.is_empty() {
                        println!("No formulae installed");
                    } else {

                        let packages: Vec<String> = packages.iter()
                            .filter_map(|p| clean_package_name(p))
                            .collect();

                        if !packages.is_empty() {
                            // Calculate columns
                            let term_width = term_size::dimensions()
                                .map(|(w, _)| w)
                                .unwrap_or(80);
                            let max_length = packages.iter()
                                .map(|p| p.len())
                                .max()
                                .unwrap_or(0);
                            let column_width = max_length + 2;
                            let num_columns = std::cmp::max(1, term_width / column_width);

                            for chunk in packages.chunks(num_columns) {
                                let line = chunk.iter()
                                    .map(|p| format!("{:width$}", p, width = column_width))
                                    .collect::<Vec<_>>()
                                    .join("");
                                println!("{}", line);
                            }
                        } else {
                            println!("No formulae installed");
                        }
                    }
                },
                Err(read::ReadError::NoAttr) => {
                    println!("No formulae installed");
                },
                Err(e) => {
                    println!("❌ Failed to read Homebrew formulae: {}", e);
                }
            }

            println!("\n==> {}", "Casks".bright_blue());
            match read::getarrvals(&content, "homebrew.casks") {
                Ok(packages) => {
                    if packages.is_empty() {
                        println!("No casks installed");
                    } else {

                        let packages: Vec<String> = packages.iter()
                            .filter_map(|p| clean_package_name(p))
                            .collect();

                        if !packages.is_empty() {
                            // Calculate columns
                            let term_width = term_size::dimensions()
                                .map(|(w, _)| w)
                                .unwrap_or(80);
                            let max_length = packages.iter()
                                .map(|p| p.len())
                                .max()
                                .unwrap_or(0);
                            let column_width = max_length + 2;
                            let num_columns = std::cmp::max(1, term_width / column_width);

                            for chunk in packages.chunks(num_columns) {
                                let line = chunk.iter()
                                    .map(|p| format!("{:width$}", p, width = column_width))
                                    .collect::<Vec<_>>()
                                    .join("");
                                println!("{}", line);
                            }
                        } else {
                            println!("No casks installed");
                        }
                    }
                },
                Err(read::ReadError::NoAttr) => {
                    println!("No casks installed");
                },
                Err(e) => {
                    println!("❌ Failed to read Homebrew casks: {}", e);
                }
            }
        }
    }

    Ok(())
}
