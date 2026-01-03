use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Error, Debug)]
pub enum ArbError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("YAML parse error in {path}: {message}")]
    Yaml { path: String, message: String },

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Schema validation failed with {0} error(s)")]
    ValidationFailed(usize),

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for ArbError {
    fn from(e: std::io::Error) -> Self {
        ArbError::Io(e.to_string())
    }
}
