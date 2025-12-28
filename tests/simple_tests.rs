use std::{net::Ipv6Addr, path::PathBuf, str::FromStr};

use derive_from_env::{FromEnv, FromEnvError};
use temp_env::with_vars;

#[derive(Debug, PartialEq, FromEnv)]
struct ServiceConfig {
    api_key: String,
    #[from_env(var = "EXT_SERVICE_URL")]
    base_url: String,
}

#[test]
fn test_0() {
    with_vars(
        vec![
            ("API_KEY", Some("api-key")),
            ("EXT_SERVICE_URL", Some("test")),
        ],
        || {
            let service_config = ServiceConfig::from_env().unwrap();
            assert_eq!(
                service_config,
                ServiceConfig {
                    api_key: "api-key".into(),
                    base_url: "test".into(),
                }
            );
        },
    );
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

#[derive(Debug, PartialEq, FromEnv)]
struct AuthConfig {
    #[from_env(from_str)]
    auth_method: AuthMethod,
    api_key: String,
}

#[derive(Debug, PartialEq, FromEnv)]
struct AppConfig {
    #[from_env(default = "0:0:0:0:0:0:0:0")]
    addr: Ipv6Addr,
    port: Option<u16>,
    external_service: ServiceConfig,
    #[from_env(no_prefix)]
    auth: AuthConfig,
}

#[test]
fn test_example() {
    with_vars(
        vec![
            ("EXTERNAL_SERVICE_API_KEY", Some("api-key")),
            ("EXT_SERVICE_URL", Some("http://myservice.com/api")),
            ("PORT", Some("8080")),
            ("AUTH_METHOD", Some("Bearer")),
            ("API_KEY", Some("api-key")),
        ],
        || {
            let app_config = AppConfig::from_env().unwrap();
            assert_eq!(
                app_config,
                AppConfig {
                    port: Some(8080),
                    addr: Ipv6Addr::from_str("0:0:0:0:0:0:0:0").unwrap(),
                    external_service: ServiceConfig {
                        api_key: "api-key".into(),
                        base_url: "http://myservice.com/api".into()
                    },
                    auth: AuthConfig {
                        auth_method: AuthMethod::Bearer,
                        api_key: "api-key".into()
                    }
                }
            )
        },
    )
}

#[derive(Debug, PartialEq, FromEnv)]
struct SubConfig {
    param_3: PathBuf,
    param_4: i32,
}

#[derive(Debug, PartialEq, FromEnv)]
struct Config {
    #[from_env(default = "0")]
    param_1: f64,
    param_2: Option<f64>,
    #[from_env(no_prefix)]
    sub: SubConfig,
}

#[test]
fn test_1() {
    with_vars(
        vec![("PARAM_3", Some("/test/path")), ("PARAM_4", Some("1"))],
        || {
            let test = Config::from_env().unwrap();
            assert_eq!(
                test,
                Config {
                    param_1: 0.,
                    param_2: None,
                    sub: SubConfig {
                        param_3: PathBuf::from("/test/path"),
                        param_4: 1
                    }
                }
            );
        },
    )
}

#[test]
fn test_2() {
    with_vars(
        vec![
            ("PARAM_2", Some("0")),
            ("PARAM_3", Some("/test/path")),
            ("PARAM_4", Some("wrong")),
        ],
        || {
            let test = Config::from_env().unwrap_err();
            assert_eq!(
                test,
                FromEnvError::ParsingFailure {
                    var_name: "PARAM_4".into(),
                    expected_type: "i32".into(),
                    str_value: "wrong".into()
                }
            );
        },
    )
}
