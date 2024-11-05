use anyhow::{Context, Result};
use colored::*;
use std::process::Command;
use crate::config::Config;

pub fn update_packages(config: &Config) -> Result<()> {
    println!("🔄 Updating packages...");

    // Get the base directory for running commands
    let config_dir = config.get_expanded_path(&config.darwin_packages_path)?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .to_path_buf();

    // If there's a custom update command, run it
    if !config.update_command.is_empty() {
        println!("⚙️  Running update command: {}", config.update_command);
        match Command::new("sh")
            .args(&["-c", &config.update_command])
            .current_dir(&config_dir)
            .output() 
        {
            Ok(output) => {
                if output.status.success() {
                    println!("✨ Update command completed successfully");
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("⚠️  Update command failed: {}", error.red());
                }
            }
            Err(e) => {
                println!("⚠️  Failed to run update command: {}", e.to_string().red());
            }
        }
    }

    // If auto_commit is enabled and there are changes, commit them
    if config.auto_commit {
        // Check if there are any changes
        if let Ok(output) = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&config_dir)
            .output() 
        {
            if !output.stdout.is_empty() {
                println!("📝 Committing changes...");
                
                // Stage all changes
                if let Ok(_) = Command::new("git")
                    .args(&["add", "."])
                    .current_dir(&config_dir)
                    .output() 
                {
                    // Commit changes
                    if let Ok(_) = Command::new("git")
                        .args(&["commit", "-m", "chore: update packages"])
                        .current_dir(&config_dir)
                        .output() 
                    {
                        println!("✨ Changes committed");

                        // Push if auto_push is enabled
                        if config.auto_push {
                            if let Ok(_) = Command::new("git")
                                .args(&["push"])
                                .current_dir(&config_dir)
                                .output() 
                            {
                                println!("🚀 Changes pushed to remote");
                            }
                        }
                    }
                }
            }
        }
    }

    println!("✅ Package update complete!");
    Ok(())
}

// Helper function for running shell commands
pub fn run_command(command: &str) -> Result<()> {
    Command::new("sh")
        .args(&["-c", command])
        .status()
        .context(format!("Failed to run command: {}", command))?;
    Ok(())
}
