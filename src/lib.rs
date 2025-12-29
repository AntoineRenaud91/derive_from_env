//! # derive_from_env
//!
//! Extract type-safe structured configuration from environment variables using procedural derive macros.
//!
//! All types are parsed using the [`FromStr`](std::str::FromStr) trait, making it easy to use
//! with any type that implements it - including your own custom types.
//!
//! ## Quick Start
//!
//! ```rust
//! use derive_from_env::FromEnv;
//!
//! #[derive(FromEnv)]
//! struct Config {
//!     database_url: String,
//!     #[from_env(default = "8080")]
//!     port: u16,
//!     debug: Option<bool>,
//! }
//!
//! // Reads from DATABASE_URL, PORT, DEBUG environment variables
//! # std::env::set_var("DATABASE_URL", "postgres://localhost/mydb");
//! let config = Config::from_env().unwrap();
//! # assert_eq!(config.database_url, "postgres://localhost/mydb");
//! # assert_eq!(config.port, 8080);
//! # assert_eq!(config.debug, None);
//! ```
//!
//! ## Field Attributes
//!
//! The `#[from_env(...)]` attribute supports the following options:
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `default = "value"` | Fallback value when env var is not set |
//! | `var = "NAME"` | Use exact env var name (ignores prefix) |
//! | `rename = "name"` | Override field name (respects prefix) |
//! | `flatten` | Mark field as nested struct |
//! | `no_prefix` | Don't add field name to prefix chain (use with `flatten`) |
//!
//! ## Struct Attributes
//!
//! | Attribute | Description |
//! |-----------|-------------|
//! | `prefix = "PREFIX_"` | Prefix for all env vars in the struct |
//!
//! ## Nested Structs
//!
//! Use `#[from_env(flatten)]` to embed nested configuration structs:
//!
//! ```rust
//! use derive_from_env::FromEnv;
//!
//! #[derive(FromEnv)]
//! struct DatabaseConfig {
//!     host: String,
//!     #[from_env(default = "5432")]
//!     port: u16,
//! }
//!
//! #[derive(FromEnv)]
//! #[from_env(prefix = "APP_")]
//! struct Config {
//!     #[from_env(flatten)]
//!     database: DatabaseConfig,  // Reads APP_DATABASE_HOST, APP_DATABASE_PORT
//! }
//! # std::env::set_var("APP_DATABASE_HOST", "localhost");
//! # let config = Config::from_env().unwrap();
//! # assert_eq!(config.database.host, "localhost");
//! # assert_eq!(config.database.port, 5432);
//! ```
//!
//! ## Custom Types
//!
//! Any type implementing [`FromStr`](std::str::FromStr) works automatically:
//!
//! ```rust
//! use std::str::FromStr;
//! use derive_from_env::FromEnv;
//!
//! #[derive(Debug, PartialEq)]
//! enum LogLevel { Debug, Info, Warn, Error }
//!
//! impl FromStr for LogLevel {
//!     type Err = String;
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         match s.to_lowercase().as_str() {
//!             "debug" => Ok(LogLevel::Debug),
//!             "info" => Ok(LogLevel::Info),
//!             "warn" => Ok(LogLevel::Warn),
//!             "error" => Ok(LogLevel::Error),
//!             _ => Err(format!("unknown log level: {}", s)),
//!         }
//!     }
//! }
//!
//! #[derive(FromEnv)]
//! struct Config {
//!     log_level: LogLevel,  // No special attribute needed!
//! }
//! # std::env::set_var("LOG_LEVEL", "debug");
//! # let config = Config::from_env().unwrap();
//! # assert_eq!(config.log_level, LogLevel::Debug);
//! ```
//!
//! ## Error Handling
//!
//! The [`from_env()`] method returns `Result<Self, FromEnvError>`:
//!
//! - [`FromEnvError::MissingEnvVar`] - Required environment variable not set
//! - [`FromEnvError::ParsingFailure`] - Failed to parse value with `FromStr`
//!
//! ```rust
//! use derive_from_env::{FromEnv, FromEnvError};
//!
//! #[derive(FromEnv)]
//! struct Config {
//!     port: u16,
//! }
//!
//! # std::env::remove_var("PORT");
//! match Config::from_env() {
//!     Ok(config) => println!("Port: {}", config.port),
//!     Err(FromEnvError::MissingEnvVar { var_name }) => {
//!         eprintln!("Missing: {}", var_name);
//!     }
//!     Err(FromEnvError::ParsingFailure { var_name, expected_type }) => {
//!         eprintln!("Failed to parse {} as {}", var_name, expected_type);
//!     }
//! }
//! ```

#[doc(hidden)]
pub mod _inner_trait;
pub use derive_from_env_proc::FromEnv;

#[derive(Debug, PartialEq, Clone)]
pub enum FromEnvError {
    MissingEnvVar {
        var_name: String,
    },
    ParsingFailure {
        var_name: String,
        expected_type: String,
    },
}

impl std::fmt::Display for FromEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromEnvError::MissingEnvVar { var_name } => {
                write!(f, "missing required environment variable: {}", var_name)
            }
            FromEnvError::ParsingFailure {
                var_name,
                expected_type,
            } => {
                write!(
                    f,
                    "failed to parse environment variable {} as {}",
                    var_name, expected_type
                )
            }
        }
    }
}

impl std::error::Error for FromEnvError {}
