// FIXME: Remove later
#![allow(unused)]

// dns-edit: DNS provider update utilities
// Copyright (C) 2025 tarkasteve@gmail.com
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::{fmt::Debug, sync::Arc};

use cfg_if::cfg_if;
use http_body_util::BodyExt;
use hyper::{
    body::{Buf, Incoming},
    client::conn::http1,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST},
    Request, Response, StatusCode,
};
use rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use serde::{de::DeserializeOwned, Serialize};

cfg_if! {
    if #[cfg(feature = "smol")] {
        use futures_rustls::TlsConnector;
        use smol::net::TcpStream;
        use smol_hyper::rt::FuturesIo as HyperIo;

    } else if #[cfg(feature = "tokio")] {
        use tokio::net::TcpStream;
        use tokio_rustls::TlsConnector;
        use hyper_util::rt::tokio::TokioIo as HyperIo;

    } else {
        compile_error!("Either smol or tokio feature must be enabled");
    }
}

use tracing::{debug, error, warn};

use crate::errors::{Error, Result};


pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
    cfg_if! {
        if #[cfg(feature = "smol")] {
            smol::spawn(future)
                .detach();

        } else if #[cfg(feature = "tokio")] {
            tokio::spawn(future);
        }
    }
}


fn load_system_certs() -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    root_store
}


pub async fn request(host: &'static str, req: Request<String>) -> Result<Response<Incoming>> {
    let addr = format!("{host}:443");
    let stream = TcpStream::connect(addr).await?;

    let cert_store = load_system_certs();
    let tlsdomain = ServerName::try_from(host)?;
    let tlsconf = ClientConfig::builder()
        .with_root_certificates(cert_store)
        .with_no_client_auth();
    let tlsconn = TlsConnector::from(Arc::new(tlsconf));
    let tlsstream = tlsconn.connect(tlsdomain, stream).await?;

    let (mut sender, conn) = http1::handshake(HyperIo::new(tlsstream)).await?;

    spawn(async move {
        if let Err(e) = conn.await {
            error!("Connection failed: {:?}", e);
        }
    });

    let res = sender.send_request(req).await?;

    Ok(res)
}


pub async fn get<T, E>(host: &'static str, endpoint: &str, auth: Option<String>) -> Result<Option<T>>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Debug,
{
    debug!("Request https://{host}{endpoint}");
    let mut req = Request::get(endpoint)
        .header(HOST, host)
        .header(ACCEPT, "application/json");
    if let Some(auth) = auth {
        req = req.header(AUTHORIZATION, auth);
    }
    let res = request(host, req.body(String::new())?).await?;

    match res.status() {
        StatusCode::OK => {
            // Asynchronously aggregate the chunks of the body
            let body = res.collect().await?
                .aggregate();
            let obj: T = serde_json::from_reader(body.reader())?;

            Ok(Some(obj))
        }
        StatusCode::NOT_FOUND => {
            warn!("Gandi record doesn't exist: {}", endpoint);
            Ok(None)
        }
        _ => {
            let body = res.collect().await?
                .aggregate();
            let err: E = serde_json::from_reader(body.reader())?;
            error!("GET failed: {err:?}");
            Err(Error::HttpError(format!("GET failed: {err:?}")))
        }
    }
}


pub async fn put<T, E>(host: &'static str, url: &str, auth: Option<String>, obj: &T) -> Result<()>
where
    T: Serialize,
    E: DeserializeOwned + Debug,
{
    let body = serde_json::to_string(obj)?;
    let mut req = Request::put(url)
        .header(HOST, host)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json");
    if let Some(auth) = auth {
        req = req.header(AUTHORIZATION, auth);
    }

    let res = request(host, req.body(body)?).await?;

    if !res.status().is_success() {
        let code = res.status();
        let body = res.collect().await?
            .aggregate();
        let err: E = serde_json::from_reader(body.reader())?;
        error!("PUT failed: {code} {err:?}");
        return Err(Error::HttpError(format!("PUT failed: {code} {err:?}")));
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;
    use serde::{Deserialize, Serialize};
    use tracing_test::traced_test;

    // See https://dummyjson.com/docs
    const HOST: &str = "dummyjson.com";

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Status {
        Ok,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestStatus {
        /* { status: 'ok', method: 'GET' } */
        status: Status,
        // Not worth mapping to enum
        method: String
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestError {
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        payload: String
    }

    async fn test_get() -> Result<()> {
        let test = get::<TestStatus, TestError>(HOST, "/test", None).await?.unwrap();
        assert_eq!(Status::Ok, test.status);
        assert_eq!("GET", test.method);
        Ok(())
    }


    async fn test_put() -> Result<()> {
        let data = TestData { payload: "test".to_string() };
        put::<TestData, TestError>(HOST, "/test", None, &data).await?;
        Ok(())
    }

    #[cfg(feature = "smol")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
        async fn smol_get() -> Result<()> {
            test_get().await
        }

        #[apply(test!)]
        #[traced_test]
        #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
        async fn smol_put() -> Result<()> {
            test_put().await
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
        async fn tokio_get() -> Result<()> {
            test_get().await
        }

        #[tokio::test]
        #[traced_test]
        #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
        async fn tokio_put() -> Result<()> {
            test_put().await
        }
    }
}
