use std::path::PathBuf;

use derive_from_env::{FromEnv, FromEnvError};
use temp_env::with_vars;

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
fn test_0() {
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
fn test_1() {
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
                FromEnvError::ParsingFailure { var_name: "PARAM_4".into(), expected_type: "i32".into(), str_value: "wrong".into() } 
            );
        },
    )
}
