use anyhow::{Context, Result};
use skim::{
    prelude::*,
    Skim,
};
use serde_json::{Value, from_str};
use std::process::Command;
use crate::config::Config;
use spinners::{Spinner, Spinners};

use super::install_package;

#[derive(Debug, Clone)]
pub enum PackageType {
    Nix,
    HomebrewFormula,
    HomebrewCask,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub source: PackageType,
}

impl Package {
    fn to_string(&self) -> String {
        let source = match self.source {
            PackageType::Nix => "nixpkgs",
            PackageType::HomebrewFormula => "homebrew formula",
            PackageType::HomebrewCask => "homebrew cask",
        };
        format!("{} ({}) ({})", self.name, self.version, source)
    }
}

pub fn search_packages(config: &Config, query: &str) -> Result<()> {
    if let Some(package) = search_package(config, query)? {
        install_package(config, &package.name, Some(package.source))?;
    }
    Ok(())
}


pub(crate) fn search_package(config: &Config, query: &str) -> Result<Option<Package>> {
    let mut sp = Spinner::new(Spinners::Dots, "Searching for packages...".into());
    
    // Search nixpkgs
    let mut packages = if cfg!(target_os = "macos") {
        search_nixpkgs(config.darwin_packages_path.as_str(), query)
            .context("Failed to search nixpkgs for Darwin")?
    } else {
        search_nixpkgs(config.linux_packages_path.as_str(), query)
            .context("Failed to search nixpkgs for Linux")?
    };

    // Search Homebrew on macOS
    if cfg!(target_os = "macos") {
        // Search formulae
        if let Ok(brew_packages) = search_homebrew(query, false) {
            packages.extend(brew_packages);
        }
        
        // Search casks
        if let Ok(brew_casks) = search_homebrew(query, true) {
            packages.extend(brew_casks);
        }
    }

    if packages.is_empty() {
        sp.stop_with_message(format!("No packages found matching '{}'", query));
        return Ok(None);
    }

    // Sort all packages by name
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    // Convert packages to skim items
    let items: Vec<String> = packages.iter()
        .map(|p| p.to_string())
        .collect();

    // Stop the spinner
    sp.stop_with_message("Available Packages:".into());

    // Create skim options
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .prompt(Some("Select package > "))
        .exit0(true)
        .build()
        .unwrap();

    // Create item reader
    let item_reader = SkimItemReader::default();
    let skim_items = item_reader.of_bufread(std::io::Cursor::new(items.join("\n")));

    // Run skim and handle the result
    match Skim::run_with(&options, Some(skim_items)) {
        Some(output) => {
            if output.is_abort {
                println!("\nSearch cancelled");
                return Ok(None);
            }

            if let Some(selected) = output.selected_items.first() {
                let selected_text = selected.output();
                if let Some(package) = packages.iter()
                    .find(|p| p.to_string() == selected_text) {
                    return Ok(Some(package.clone()));
                }
            }
        },
        None => {
            println!("\nSearch cancelled");
        }
    }

    Ok(None)
}

fn search_nixpkgs(packages_path: &str, query: &str) -> Result<Vec<Package>> {
    let config = Config::load()?;
    let packages_path = config.get_expanded_path(packages_path)?;

    let output = Command::new("nix")
        .args([
            "--extra-experimental-features", "nix-command",
            "--extra-experimental-features", "flakes",
            "search", "nixpkgs", query,
            "--json"
        ])
        .current_dir(packages_path.parent().unwrap_or(&packages_path))
        .output()
        .context("Failed to execute nix search")?;

    let json_str = String::from_utf8(output.stdout)
        .context("Failed to parse nix search output as UTF-8")?;

    let json: Value = from_str(&json_str)
        .context("Failed to parse JSON output")?;

    let mut packages = Vec::new();

    if let Value::Object(entries) = json {
        for (_key, value) in entries {
            if let Value::Object(pkg) = value {
                if let (Some(Value::String(name)), Some(Value::String(version))) = 
                    (pkg.get("pname").or_else(|| pkg.get("name")), pkg.get("version")) {
                    packages.push(Package {
                        name: name.clone(),
                        version: version.clone(),
                        source: PackageType::Nix,
                    });
                }
            }
        }
    }

    Ok(packages)
}

fn search_homebrew(query: &str, is_cask: bool) -> Result<Vec<Package>> {
    let mut args = vec!["search"];
    if is_cask {
        args.push("--cask");
    }
    args.push(query);

    let output = Command::new("brew")
        .args(&args)
        .output()
        .context("Failed to execute brew search")?;

    let output_str = String::from_utf8(output.stdout)
        .context("Failed to parse brew search output as UTF-8")?;

    let mut packages = Vec::new();

    for line in output_str.lines() {
        let name = line.trim();
        if !name.is_empty() {
            let version = get_homebrew_version(name, is_cask)?;
            packages.push(Package {
                name: name.to_string(),
                version,
                source: if is_cask { 
                    PackageType::HomebrewCask 
                } else { 
                    PackageType::HomebrewFormula 
                },
            });
        }
    }

    Ok(packages)
}

fn get_homebrew_version(package: &str, is_cask: bool) -> Result<String> {
    let info_output = Command::new("brew")
        .args(if is_cask {
            vec!["info", "--cask", package]
        } else {
            vec!["info", package]
        })
        .output()
        .context("Failed to get package info")?;

    if info_output.status.success() {
        let info_str = String::from_utf8_lossy(&info_output.stdout);
        Ok(info_str.lines()
            .find(|line| line.contains(':'))
            .and_then(|line| line.split(':').nth(1))
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| "latest".to_string()))
    } else {
        Ok("latest".to_string())
    }
}
