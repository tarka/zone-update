use std::str::FromStr;

use serde::de::DeserializeOwned;
use tracing::{error, warn};
use ureq::{http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderName, HeaderValue, Response, StatusCode}, tls::TlsConfig, Agent, Body, RequestBuilder, ResponseExt};

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


/// Extension trait for adding headers and authentication to a `ureq` request builder.
///
/// This trait provides convenient methods for adding multiple headers, authentication tokens,
/// and common JSON headers to a `RequestBuilder`.
pub(crate) trait WithHeaders<T> {
    /// Adds a collection of headers to the request builder.
    ///
    /// This method takes a vector of key-value pairs and adds them as headers to the request.
    /// It validates both the header names and values, returning an error if either is invalid.
    ///
    /// # Arguments
    ///
    /// * `headers` - A vector of `(&str, String)` tuples representing header key-value pairs.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the modified `RequestBuilder` on success, or an `Error`
    /// if header validation fails.
    fn with_headers(self, headers: Vec<(&str, String)>) -> Result<RequestBuilder<T>>;

    /// Adds an `AUTHORIZATION` header to the request builder.
    ///
    /// This is a convenience method for setting the `AUTHORIZATION` header, commonly used for
    /// bearer tokens or other authentication schemes.
    ///
    /// # Arguments
    ///
    /// * `auth` - The authentication token or value as a `String`.
    ///
    /// # Returns
    ///
    /// Returns the modified `RequestBuilder`.
    fn with_auth(self, auth: String) -> RequestBuilder<T>;

    /// Adds `ACCEPT` and `CONTENT_TYPE` headers for JSON content.
    ///
    /// This method sets the `ACCEPT` and `CONTENT_TYPE` headers to `application/json`,
    /// which is a common requirement for REST APIs.
    ///
    /// # Returns
    ///
    /// Returns the modified `RequestBuilder`.
    fn with_json_headers(self) -> RequestBuilder<T>;
}

/// Implementation of the `WithHeaders` trait for `RequestBuilder`.
impl<Any> WithHeaders<Any> for RequestBuilder<Any> {

    fn with_headers(mut self, headers: Vec<(&str, String)>) -> Result<Self> {
        let mut reqh = self.headers_mut()
            .ok_or(Error::HttpError("Failed to get headers from ureq".to_string()))?;

        for (k, v) in headers {
            reqh.insert(HeaderName::from_str(k)?, HeaderValue::from_str(&v)?);
        }

        Ok(self)
    }

    fn with_auth(self, auth: String) -> Self {
        self.header(AUTHORIZATION, auth)
    }

    fn with_json_headers(self) -> Self {
        self.header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
    }
}

/// Create and return a configured HTTP ureq agent.
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
