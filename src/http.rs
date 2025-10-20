use std::str::FromStr;

use serde::de::DeserializeOwned;
use tracing::{error, warn};
use ureq::{http::{HeaderName, HeaderValue, Response, StatusCode}, tls::TlsConfig, Agent, Body, RequestBuilder, ResponseExt};

use crate::errors::{Error, Result};



/// Extension trait for converting ureq HTTP responses to optional
/// values or error information.
///
/// This trait provides methods for handling HTTP responses in a way that:
/// - Converts successful responses (status 200 OK) into deserialized values
/// - Treats not found responses (status 404) as `None` values
/// - Converts other error statuses into appropriate error types
pub(crate) trait ResponseToOption {
    /// Converts the HTTP response body to an optional value based on the status code.
    ///
    /// This method handles different HTTP status codes as follows:
    /// - `200 OK`: Deserializes the response body into the requested type `T`
    /// - `404 NOT_FOUND`: Returns `None` without erroring (useful for optional lookups)
    /// - All other status codes: Returns an `ApiError` with the status and response body
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the response body into. Must implement `DeserializeOwned`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Some(T))` if the response status is 200 OK and deserialization succeeds
    /// - `Ok(None)` if the response status is 404 NOT_FOUND
    /// - `Err(Error)` if the response status is not 200 or 404, or if deserialization fails
    fn to_option<T>(&mut self) -> Result<Option<T>>
    where
        T: DeserializeOwned;

    /// Extracts error information from the HTTP response.
    ///
    /// This method reads the response body and creates an `Error` based on the
    /// response status code and content, primarily used for handling error responses.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Error>` containing the error information extracted from the response.
    fn from_error(&mut self) -> Result<Error>;
}



/// Implementation of the `ResponseToOption` trait for `Response<Body>`.
///
/// This implementation provides concrete functionality for converting HTTP responses
/// into optional values or error information based on response status codes.
impl ResponseToOption for Response<Body> {
    /// Converts an HTTP response to an optional value based on its status.
    ///
    /// For successful responses (200 OK), this method deserializes the response body
    /// into the requested type. For 404 responses, it returns `None`. For other
    /// status codes, it returns an `ApiError`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the response body into, must implement `DeserializeOwned`
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - For 200 OK responses with successfully deserialized content
    /// * `Ok(None)` - For 404 NOT_FOUND responses
    /// * `Err(Error)` - For other status codes or deserialization failures
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

    /// Extracts error information from an HTTP response.
    ///
    /// Reads the response body and status code to create an appropriate error.
    ///
    /// # Returns
    ///
    /// Returns an `Error::HttpError` containing information about the failed request.
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

/// Create and return a configured HTTP client agent.
///
/// This function sets up a ureq Agent with specific configuration options
/// that are suitable for DNS provider APIs, including support for legacy
/// TLS protocol versions that some providers may still use.
///
/// The client is configured with:
/// - HTTP status codes not treated as errors (http_status_as_error(false))
/// - Native TLS provider to support legacy TLS versions that Rustls doesn't support
///
/// # Returns
///
/// Returns a configured `Agent` instance that can be used to make HTTP requests.
///
/// # Example
///
/// ```
/// use ureq::Agent;
///
/// let client = client();
/// let response = client.get("https://api.example.com/records").call();
/// ```
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
