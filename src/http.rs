use std::str::FromStr;

use serde::de::DeserializeOwned;
use tracing::{error, warn};
use ureq::{http::{HeaderName, HeaderValue, Response, StatusCode}, tls::TlsConfig, Agent, Body, RequestBuilder, ResponseExt};

use crate::errors::{Error, Result};



pub(crate) trait ResponseToOption {
    fn to_option<T>(&mut self) -> Result<Option<T>>
    where
        T: DeserializeOwned;

    fn from_error(&mut self) -> Result<Error>;
}



impl ResponseToOption for Response<Body> {
    fn to_option<T>(&mut self) -> Result<Option<T>>
    where
        T: DeserializeOwned
    {
        match self.status() {
            StatusCode::OK => {
                let body = self.body_mut().read_to_string()?;
                let obj: T = serde_json::from_str(&body)?;
                Ok(Some(obj))
            }
            StatusCode::NOT_FOUND => {
                warn!("Record doesn't exist: {}", self.get_uri());
                Ok(None)
            }
            _ => {
                let body = self.body_mut().read_to_string()?;
                Err(Error::ApiError(format!("Api Error: {} -> {body}", self.status())))
            }
        }
    }

    fn from_error(&mut self) -> Result<Error> {
        let code = self.status();
        let mut err = String::new();
        let _nr = self.body_mut()
            .read_to_string()?;
        error!("REST op failed: {code} {err:?}");
        Ok(Error::HttpError(format!("REST op failed: {code} {err:?}")))
    }

}


/// Extension trait to adding multiple headers to a ureq request
/// builder.
pub(crate) trait WithHeaders<T> {
    /// Adds the provided headers to the request builder.
    ///
    /// # Arguments
    ///
    /// * `headers` - A vector of header key-value pairs where the key is a string slice
    ///   and the value is a String.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the modified `RequestBuilder` on success,
    /// or an `Error` if header validation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The headers cannot be extracted from the request builder
    /// - Any of the header names or values are invalid
    fn with_headers(self, headers: Vec<(&str, String)>) -> Result<RequestBuilder<T>>;
}

/// Implementation of the `WithHeaders` trait for `RequestBuilder`.
impl<Any> WithHeaders<Any> for RequestBuilder<Any> {
    /// Adds the specified headers to the request builder.
    ///
    /// This method iterates through the provided header pairs and adds them to the
    /// request. Each header name and value is validated before insertion.
    ///
    /// # Arguments
    ///
    /// * `headers` - A vector of tuples containing header names and values
    ///
    /// # Returns
    ///
    /// Returns the modified `RequestBuilder` wrapped in a `Result` on success,
    /// or an `Error` if any header validation fails.
    fn with_headers(mut self, headers: Vec<(&str, String)>) -> Result<Self> {
        let mut reqh = self.headers_mut()
            .ok_or(Error::HttpError("Failed to get headers from ureq".to_string()))?;

        for (k, v) in headers {
            reqh.insert(HeaderName::from_str(k)?, HeaderValue::from_str(&v)?);
        }

        Ok(self)
    }
}

pub(crate) fn client() -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .tls_config(
            // At least one provider (DnsMadeEasy) uses legacy TLS
            // protocol versions that Rustls doesn't support on their
            // sandbox.
            TlsConfig::builder()
                .provider(ureq::tls::TlsProvider::NativeTls)
                .build()
        )
        .build()
        .new_agent()
}
