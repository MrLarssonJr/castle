use std::ffi::OsString;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseConfigFromEnvError {
    #[error("environment variable key is not valid unicode: \"{0:?}\"")]
    KeyNotUnicode(OsString),
    #[error("environment variable value is not valid unicode: \"{0:?}\"")]
    ValueNotUnicode(OsString),
}
