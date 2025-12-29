use std::{net::Ipv6Addr, path::PathBuf, str::FromStr};

use derive_from_env::FromEnv;
use temp_env::with_vars;

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

#[derive(Debug, PartialEq, FromEnv)]
struct AuthConfig {
    auth_method: AuthMethod,
    api_key: String,
}

#[derive(Debug, PartialEq, FromEnv)]
struct ServiceConfig {
    addr: Ipv6Addr,
    #[from_env(default = "8080")]
    port: u16,
    #[from_env(var = "EXT_SERVICE_URL")]
    base_url: String,
}

#[derive(Debug, PartialEq, FromEnv)]
#[from_env(prefix = "APP_")]
struct AppConfig {
    #[from_env(flatten)]
    service: ServiceConfig,
    #[from_env(flatten, no_prefix)]
    auth: AuthConfig,
    log_file: Option<PathBuf>,
    #[from_env(rename = "IS_PROD")]
    is_production: bool,
}

fn main() {
    with_vars(
        vec![
            ("APP_SERVICE_ADDR", Some("0:0:0:0:0:0:0:0")),
            ("EXT_SERVICE_URL", Some("http://myservice.com/api")),
            ("APP_AUTH_METHOD", Some("Bearer")),
            ("APP_API_KEY", Some("api-key")),
            ("APP_IS_PROD", Some("true")),
        ],
        || {
            let app_config = AppConfig::from_env().unwrap();
            assert_eq!(
                app_config,
                AppConfig {
                    service: ServiceConfig {
                        addr: Ipv6Addr::from_str("0:0:0:0:0:0:0:0").unwrap(),
                        port: 8080,
                        base_url: "http://myservice.com/api".into()
                    },
                    auth: AuthConfig {
                        auth_method: AuthMethod::Bearer,
                        api_key: "api-key".into()
                    },
                    log_file: None,
                    is_production: true,
                }
            )
        },
    )
}
