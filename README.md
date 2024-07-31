# derive_from_env
Extract type safe structured data from environment variables with procedural derive macros. Currently based on the [FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html) trait. 

## Usage
- Add `derive_from_env` to your project's dependencies:
```toml
[dependencies]
derive_from_env = "0.1.1"
```
- Usage example:
```rust	
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use derive_from_env::FromEnv;

#[derive(Debug, PartialEq, FromEnv)]
struct ServiceConfig {
    api_key: String,
    #[from_env(var = "EXT_SERVICE_URL")]
    base_url: String,
}

#[derive(Debug, PartialEq)]
enum AuthMethod {
    Bearer,
    XAPIKey,
}

impl FromStr for AuthMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bearer" => Ok(AuthMethod::Bearer),
            "X-API-Key" => Ok(AuthMethod::XAPIKey),
            _ => Err("Invalid auth method".into()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[derive(FromEnv)]
struct AuthConfig {
    #[from_env(from_str)]
    auth_method: AuthMethod,
    api_key: String,
}

#[derive(Debug, PartialEq, FromEnv)]
struct AppConfig {
    #[from_env(default = "0.0.0.0")]
    addr: IpAddr,
    port: Option<u16>,
    external_service: ServiceConfig,
    #[from_env(no_prefix)]
    auth: AuthConfig 
}

fn main() {
    std::env::set_var("EXTERNAL_SERVICE_API_KEY", "api-key");
    std::env::set_var("EXT_SERVICE_URL", "http://external.service/api");
    std::env::set_var("PORT","8080");
    std::env::set_var("AUTH_METHOD","Bearer");
    std::env::set_var("API_KEY","api-key");
    let app_config = AppConfig::from_env().unwrap();
    assert_eq!(app_config, AppConfig {
        port: Some(8080),
        addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        external_service: ServiceConfig {
            api_key: "api-key".into(),
            base_url: "http://external.service/api".into() 
        },
        auth: AuthConfig {
            auth_method: AuthMethod::Bearer,
            api_key: "api-key".into()
        }
    });
}
```

# In the future...
- [serde](https://crates.io/crates/serde) support via a new attribute `from_serde` (usefull for [chrono](https://crates.io/crates/chrono)'s types)

