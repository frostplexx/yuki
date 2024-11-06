use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing)]
    #[deprecated(
        since = "0.2.0",
        note = "Use system_packages_path instead. This will be removed in a future version."
    )]
    pub linux_packages_path: String,
    
    #[serde(skip_serializing)]
    #[deprecated(
        since = "0.2.0",
        note = "Use system_packages_path instead. This will be removed in a future version."
    )]
    pub darwin_packages_path: String,
    
    pub system_packages_path: String,
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
        // First try to get the unified path
        let system_packages_path = std::env::var("YUKI_SYSTEM_PACKAGES_PATH").unwrap_or_else(|_| {
            // If not found, try the legacy paths based on the current platform
            if cfg!(target_os = "macos") {
                std::env::var("YUKI_DARWIN_PACKAGES_PATH")
            } else {
                std::env::var("YUKI_LINUX_PACKAGES_PATH")
            }.unwrap_or_else(|_| "~/dotfiles/apps.nix".to_string())
        });

        let config = Self {
            // Set legacy fields to the same value for backward compatibility
            linux_packages_path: system_packages_path.clone(),
            darwin_packages_path: system_packages_path.clone(),
            system_packages_path,
            homebrew_packages_path: std::env::var("YUKI_HOMEBREW_PACKAGES_PATH")
                .unwrap_or_else(|_| "~/dotfiles/hosts/darwin/apps.nix".to_string()),
            auto_commit: std::env::var("YUKI_AUTO_COMMIT")
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
            auto_push: std::env::var("YUKI_AUTO_PUSH")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            uninstall_message: std::env::var("YUKI_UNINSTALL_MESSAGE")
                .unwrap_or_else(|_| "removed <package>".to_string()),
            install_message: std::env::var("YUKI_INSTALL_MESSAGE")
                .unwrap_or_else(|_| "installed <package>".to_string()),
            install_command: std::env::var("YUKI_INSTALL_COMMAND")
                .unwrap_or_else(|_| "make".to_string()),
            uninstall_command: std::env::var("YUKI_UNINSTALL_COMMAND")
                .unwrap_or_else(|_| "make".to_string()),
            update_command: std::env::var("YUKI_UPDATE_COMMAND")
                .unwrap_or_else(|_| "make update".to_string()),
        };
        
        // Print deprecation warning if old environment variables are used
        if std::env::var("YUKI_LINUX_PACKAGES_PATH").is_ok() || 
           std::env::var("YUKI_DARWIN_PACKAGES_PATH").is_ok() {
            eprintln!("Warning: YUKI_LINUX_PACKAGES_PATH and YUKI_DARWIN_PACKAGES_PATH are deprecated.");
            eprintln!("Please use YUKI_SYSTEM_PACKAGES_PATH instead.");
        }
        
        Ok(config)
    }

    pub fn get_expanded_path(&self, path: &str) -> Result<PathBuf> {
        let expanded = shellexpand::tilde(path);
        Ok(PathBuf::from(expanded.into_owned()))
    }

    #[deprecated(
        since = "0.2.0",
        note = "Use get_system_packages_path instead. This will be removed in a future version."
    )]
    pub fn get_linux_packages_path(&self) -> Result<PathBuf> {
        self.get_expanded_path(&self.system_packages_path)
    }

    #[deprecated(
        since = "0.2.0",
        note = "Use get_system_packages_path instead. This will be removed in a future version."
    )]
    pub fn get_darwin_packages_path(&self) -> Result<PathBuf> {
        self.get_expanded_path(&self.system_packages_path)
    }

    pub fn get_system_packages_path(&self) -> Result<PathBuf> {
        self.get_expanded_path(&self.system_packages_path)
    }

    pub fn get_homebrew_packages_path(&self) -> Result<PathBuf> {
        self.get_expanded_path(&self.homebrew_packages_path)
    }
}
