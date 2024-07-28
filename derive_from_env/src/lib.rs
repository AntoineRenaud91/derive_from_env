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
        str_value: String,
    },
}
