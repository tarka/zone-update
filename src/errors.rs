
use std::result;
use rustls::pki_types::InvalidDnsNameError;
use thiserror::Error;


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

    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    HostError(#[from] InvalidDnsNameError),

    #[error(transparent)]
    HyperError(#[from] hyper::Error),

    #[error(transparent)]
    HyperHttpError(#[from] hyper::http::Error),

    #[error(transparent)]
    HeaderNameError(#[from] http::header::InvalidHeaderName),

    #[error(transparent)]
    HeaderValueError(#[from] http::header::InvalidHeaderValue),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
}

pub type Result<T> = result::Result<T, Error>;
