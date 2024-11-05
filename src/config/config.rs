use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub linux_packages_path: String,
    pub darwin_packages_path: String,
    pub homebrew_packages_path: String,
    pub auto_commit: bool,
    pub auto_push: bool,
    pub uninstall_message: String,
    pub install_message: String,
    pub install_command: String,
    pub uninstall_command: String,
    pub update_command: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            Self::create_default_config(&config_path)?;
        }
        
        let config_str = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
            
        // Parse TOML-style config
        let mut config = Self::default();
        for line in config_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let key = parts[0];
            let value = parts[1];
            
            match key {
                "linux_packages_path" => config.linux_packages_path = value.to_string(),
                "darwin_packages_path" => config.darwin_packages_path = value.to_string(),
                "homebrew_packages_path" => config.homebrew_packages_path = value.to_string(),
                "auto_commit" => config.auto_commit = value.parse().unwrap_or(false),
                "auto_push" => config.auto_push = value.parse().unwrap_or(false),
                "uninstall_message" => config.uninstall_message = value.trim_matches('"').to_string(),
                "install_message" => config.install_message = value.trim_matches('"').to_string(),
                "install_command" => config.install_command = value.trim_matches('"').to_string(),
                "uninstall_command" => config.uninstall_command = value.trim_matches('"').to_string(),
                "update_command" => config.update_command = value.trim_matches('"').to_string(),
                _ => {}
            }
        }
        
        Ok(config)
    }

    pub fn get_config_path() -> Result<PathBuf> {
        // Try ~/.nixp first
        if let Some(home) = dirs::home_dir() {
            let nixprc = home.join(".nixprc");
            if nixprc.exists() {
                return Ok(nixprc);
            }
        }
        // Try ~/.config/nixp/config.conf
        if let Some(config_dir) = dirs::config_dir() {
            let nixp_config = config_dir.join("nixp");
            fs::create_dir_all(&nixp_config)?;
            return Ok(nixp_config.join("config.conf"));
        }
        Err(anyhow::anyhow!("Could not determine config path"))
    }

    pub fn create_default_config(path: &Path) -> Result<()> {
        let config = r#"# Path to linux system packages nix file 
linux_packages_path ~/dotfiles/hosts/nixos/apps.nix
# Path to darwin system packages nix file 
darwin_packages_path ~/dotfiles/hosts/darwin/apps.nix
homebrew_packages_path ~/dotfiles/hosts/darwin/apps.nix
# Git setup
# Automatically add a commit when installing or uninstalling packages
auto_commit true
auto_push false
# Uninstall and install message. Use <package> to insert the package name
uninstall_message "removed <package>"
install_message "installed <package>"
# This is the command that will be run after your package has been added to the package config
install_command "make"
# This is the command that will be run after your package has been removed from the package config
uninstall_command "make"
# This is the command that will be run to update your packages
update_command "make update""#;
        
        fs::write(path, config)?;
        Ok(())
    }

    pub fn get_expanded_path(&self, path: &str) -> Result<PathBuf> {
        let expanded = shellexpand::tilde(path);
        Ok(PathBuf::from(expanded.into_owned()))
    }

    fn default() -> Self {
        Self {
            linux_packages_path: "~/dotfiles/hosts/nixos/apps.nix".to_string(),
            darwin_packages_path: "~/dotfiles/hosts/darwin/apps.nix".to_string(),
            homebrew_packages_path: "~/dotfiles/hosts/darwin/apps.nix".to_string(),
            auto_commit: true,
            auto_push: false,
            uninstall_message: "removed <package>".to_string(),
            install_message: "installed <package>".to_string(),
            install_command: "make".to_string(),
            uninstall_command: "make".to_string(),
            update_command: "make".to_string(),
        }
    }
}
