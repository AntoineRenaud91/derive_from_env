use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

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
            let service_config = ServiceConfig::from_env();
            println!("{:?}", service_config)
        },
    );
}

#[derive(Debug, PartialEq, FromEnv)]
struct AppConfig {
    #[from_env(default = "8080")]
    port: u16,
    #[from_env(default = "0.0.0.0")]
    addr: IpAddr,
    external_service: ServiceConfig,
}

#[test]
fn test_example() {
    with_vars(
        vec![
            ("EXTERNAL_SERVICE_API_KEY", Some("test")),
            ("EXT_SERVICE_URL", Some("http://myservice.com/api")),
        ],
        || {
            let app_config = AppConfig::from_env().unwrap();
            assert_eq!(
                app_config,
                AppConfig {
                    port: 8080,
                    addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    external_service: ServiceConfig {
                        api_key: "test".into(),
                        base_url: "http://myservice.com/api".into()
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
    param_2: f64,
    #[from_env(no_prefix)]
    sub: SubConfig,
}

#[test]
fn test_1() {
    with_vars(
        vec![
            ("PARAM_2", Some("0")),
            ("PARAM_3", Some("/test/path")),
            ("PARAM_4", Some("1")),
        ],
        || {
            let test = Config::from_env().unwrap();
            assert_eq!(
                test,
                Config {
                    param_1: 0.,
                    param_2: 0.0,
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
