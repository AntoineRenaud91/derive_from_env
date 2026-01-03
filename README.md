# derive_from_env

[![Crates.io](https://img.shields.io/crates/v/derive_from_env.svg)](https://crates.io/crates/derive_from_env)
[![Documentation](https://docs.rs/derive_from_env/badge.svg)](https://docs.rs/derive_from_env)

Extract type-safe structured configuration from environment variables using procedural derive macros.

All types are parsed using the [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) trait, making it easy to use with any type that implements it - including your own custom types.

## Installation

```toml
[dependencies]
derive_from_env = "0.1"
```

## Quick Start

```rust
use derive_from_env::FromEnv;

#[derive(FromEnv)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

fn main() {
    // Reads from HOST, PORT, DEBUG environment variables
    let config = Config::from_env().unwrap();
}
```

## Struct Attributes

### `prefix = "PREFIX_"`

Sets a prefix for all environment variables in the struct.

```rust
#[derive(FromEnv)]
#[from_env(prefix = "MYAPP_")]
struct Config {
    host: String,  // Reads from MYAPP_HOST
    port: u16,     // Reads from MYAPP_PORT
}
```

## Field Attributes

### `default = "value"`

Provides a fallback value when the environment variable is not set.

```rust
#[derive(FromEnv)]
struct Config {
    #[from_env(default = "localhost")]
    host: String,
    #[from_env(default = "8080")]
    port: u16,
}
```

### `var = "NAME"`

Overrides the environment variable name completely. This ignores any prefix.

```rust
#[derive(FromEnv)]
#[from_env(prefix = "APP_")]
struct Config {
    #[from_env(var = "DATABASE_URL")]  // Uses DATABASE_URL, not APP_DATABASE_URL
    db_url: String,
}
```

### `rename = "name"`

Renames the field for environment variable lookup, but still respects the prefix.

```rust
#[derive(FromEnv)]
#[from_env(prefix = "APP_")]
struct Config {
    #[from_env(rename = "db_url")]  // Uses APP_DB_URL instead of APP_DATABASE_URL
    database_url: String,
}
```

### `flatten`

Marks a field as a nested struct. Required for any field that is itself a struct deriving `FromEnv` but not implementing `FromStr`.

```rust
#[derive(FromEnv)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[derive(FromEnv)]
struct Config {
    #[from_env(flatten)]
    database: DatabaseConfig,  // Reads from DATABASE_HOST, DATABASE_PORT
}
```

Struct prefixes combine when nesting structs:

```rust
#[derive(FromEnv)]
#[from_env(prefix = "SERVICE_")]
struct ServiceConfig {
    host: String,
    port: u16,
}

#[derive(FromEnv)]
#[from_env(prefix = "APP_")]
struct Config {
    #[from_env(flatten)]
    database: DatabaseConfig,  // Reads from APP_DATABASE_SERVICE_HOST, APP_DATABASE_SERVICE_PORT
}
```


### `no_prefix`

Used with `flatten` to prevent adding the field name to the prefix chain.

```rust
#[derive(FromEnv)]
#[from_env(prefix = "APP_")]
struct Config {
    #[from_env(flatten, no_prefix)]
    database: DatabaseConfig,  // Reads from APP_HOST, APP_PORT (not APP_DATABASE_HOST)
}
```

## Type Handling

### FromStr Types

All types are parsed using the `FromStr` trait. This includes:

- Primitives: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `char`
- Standard library types: `String`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `PathBuf`
- Any custom type implementing `FromStr`

```rust
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("unknown log level: {}", s)),
        }
    }
}

#[derive(FromEnv)]
struct Config {
    log_level: LogLevel,  // No special attribute needed!
}
```

### Option Types

`Option<T>` fields return `None` when the environment variable is not set.

```rust
#[derive(FromEnv)]
struct Config {
    host: String,           // Required
    port: Option<u16>,      // Optional, returns None if not set
}
```

## Error Handling

The `from_env()` method returns `Result<Self, FromEnvError>`.

```rust
use derive_from_env::{FromEnv, FromEnvError};

#[derive(FromEnv)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    match Config::from_env() {
        Ok(config) => println!("Loaded config: {:?}", config),
        Err(FromEnvError::MissingEnvVar { var_name }) => {
            eprintln!("Missing required variable: {}", var_name);
        }
        Err(FromEnvError::ParsingFailure { var_name, expected_type }) => {
            eprintln!("Failed to parse {} as {}", var_name, expected_type);
        }
    }
}
```

## License

MIT
