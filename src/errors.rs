use std::result;
use thiserror::Error;


/// Error type for operations in this crate.
///
/// Represents various failure modes encountered when communicating with
/// DNS provider APIs or performing local operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("API usage error: {0}")]
    ApiError(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("URL error: {0}")]
    UrlError(String),

    #[error("Unexpected record value: {0}")]
    UnexpectedRecord(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    UreqError(#[from] ureq::Error),

    #[error("Failed to lock: {0}")]
    LockingError(String),

    #[error(transparent)]
    HeaderNameError(#[from] ureq::http::header::InvalidHeaderName),

    #[error(transparent)]
    HeaderValueError(#[from] ureq::http::header::InvalidHeaderValue),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    // #[error(transparent)]
    // RustlsError(#[from] rustls::Error),
}

/// Result type returned by functions in this crate.
///
/// Uses the crate-local `Error` type as the error variant.
pub type Result<T> = result::Result<T, Error>;
