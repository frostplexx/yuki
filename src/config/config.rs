use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

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
        let config = Self {
            linux_packages_path: std::env::var("YUKI_LINUX_PACKAGES_PATH")
                .unwrap_or_else(|_| "~/dotfiles/hosts/nixos/apps.nix".to_string()),
            darwin_packages_path: std::env::var("YUKI_DARWIN_PACKAGES_PATH")
                .unwrap_or_else(|_| "~/dotfiles/hosts/darwin/apps.nix".to_string()),
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
        
        Ok(config)
    }

    pub fn get_expanded_path(&self, path: &str) -> Result<PathBuf> {
        let expanded = shellexpand::tilde(path);
        Ok(PathBuf::from(expanded.into_owned()))
    }
}
