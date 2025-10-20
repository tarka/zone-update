
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


pub(crate) trait WithHeaders<T> {
    fn with_headers(self, headers: Vec<(&str, String)>) -> Result<RequestBuilder<T>>;
}

impl<Any> WithHeaders<Any> for RequestBuilder<Any> {
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
