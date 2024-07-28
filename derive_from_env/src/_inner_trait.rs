use crate::FromEnvError;

pub trait FromEnv: Sized {
    fn from_env() -> Result<Self, FromEnvError>;
    fn from_env_with_prefix(prefix: &str) -> Result<Self, FromEnvError>;
}
