// Re-export modules
pub mod config;

// Re-export specific items if needed
pub use config::Config;

// Any shared types, traits, or functions that are common across modules
pub type Result<T> = anyhow::Result<T>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}