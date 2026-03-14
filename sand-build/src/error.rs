use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unknown Minecraft version '{0}'. Check the version string and try again.")]
    UnknownVersion(String),

    #[error("SHA1 checksum mismatch for '{path}': expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error(
        "Java not found on PATH. Please install Java 21+ and ensure the `java` binary is accessible."
    )]
    JavaNotFound,

    #[error("Data generator exited with code {code}.\nstderr:\n{stderr}")]
    DataGeneratorFailed { code: i32, stderr: String },

    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("Missing field '{field}' in {context}")]
    MissingField {
        field: &'static str,
        context: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
