use serde::de::DeserializeOwned;
use tracing::{error, warn};
use ureq::{http::{Response, StatusCode}, Agent, Body, ResponseExt};

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
                //Err(self.from_error()?)
                Err(Error::ApiError("TEST".to_string()))
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


pub(crate) fn client() -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .new_agent()
}

