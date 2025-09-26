
use std::{io::Read, sync::Arc};

use async_lock::OnceCell;
use cfg_if::cfg_if;
use http::request::Builder;
use http_body_util::BodyExt;
use hyper::{
    body::{Buf, Incoming}, client::conn::http1, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST}, Method, Response, StatusCode, Uri
};
use rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{error, warn};

use crate::errors::{Error, Result};

cfg_if! {
    if #[cfg(feature = "smol")] {
        use smol::net::TcpStream;
        use futures_rustls::TlsConnector;
        use smol_hyper::rt::FuturesIo as HyperIo;

    } else if #[cfg(feature = "tokio")] {
        use tokio::net::TcpStream;
        use tokio_rustls::TlsConnector;
        use hyper_util::rt::tokio::TokioIo as HyperIo;

    } else {
        compile_error!("Either smol or tokio feature must be enabled");
    }
}

fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
    cfg_if! {
        if #[cfg(feature = "smol")] {
            smol::spawn(future)
                .detach();

        } else if #[cfg(feature = "tokio")] {
            tokio::spawn(future);
        }
    }

    // NOTE: This also works, and could be a fallback for other runtimes?
    //
    // let _join = thread::spawn(|| {
    //     pollster::block_on(future);
    // });
}


const ROOT_STORE: OnceCell<Arc<RootCertStore>> = OnceCell::new();

async fn load_system_certs() -> Arc<RootCertStore> {
    ROOT_STORE.get_or_init(|| async {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Arc::new(root_store)
    }).await.clone()
}


async fn request<In>(method: Method, uri: &Uri, obj: Option<In>, auth: Option<String>) -> Result<Response<Incoming>>
where
    In: Serialize,
{
    let host = uri.host()
        .ok_or(Error::UrlError(format!("URL: {:?}", uri)))?
        .to_owned();

    let mut rb = Builder::new()
        .method(method)
        .uri(uri)
        .header(HOST, &host)
        .header(ACCEPT, "application/json");
    if let Some(auth) = auth {
        rb = rb.header(AUTHORIZATION, auth);
    }
    let req = if obj.is_some() {
        rb = rb.header(CONTENT_TYPE, "application/json");
        let body = serde_json::to_string(&obj)?;
        rb.body(body)?
    } else {
        rb.body("".to_string())?
    };

    let stream = TcpStream::connect((host.clone(), 443)).await?;

    let cert_store = load_system_certs();
    let tlsdomain = ServerName::try_from(host)?;
    let tlsconf = ClientConfig::builder()
        .with_root_certificates(cert_store.await)
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


async fn from_error(res: Response<Incoming>) -> Result<Error> {
    let code = res.status();
    let mut err = String::new();
    let _nr = res.collect().await?
        .to_bytes()
        .reader()
        .read_to_string(&mut err)?;
    error!("REST op failed: {code} {err:?}");
    Ok(Error::HttpError(format!("REST op failed: {code} {err:?}")))
}


pub async fn get<T>(uri: Uri, auth: Option<String>) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let res = request(Method::GET, &uri, None::<&str>, auth).await?;

    match res.status() {
        StatusCode::OK => {
            let body = res.collect().await?
                .aggregate();
            let obj: T = serde_json::from_reader(body.reader())?;

            Ok(Some(obj))
        }
        StatusCode::NOT_FOUND => {
            warn!("Record doesn't exist: {}", uri);
            Ok(None)
        }
        _ => {
            Err(from_error(res).await?)
        }
    }
}


pub async fn put<T>(uri: Uri, obj: &T, auth: Option<String>) -> Result<()>
where
    T: Serialize,
{
    let res = request(Method::PUT, &uri, Some(obj), auth).await?;

    if !res.status().is_success() {
        return Err(from_error(res).await?);
    }

    Ok(())
}


pub async fn post<T>(uri: Uri, obj: &T, auth: Option<String>) -> Result<()>
where
    T: Serialize,
{
    let res = request(Method::POST, &uri, Some(obj), auth).await?;

    if !res.status().is_success() {
        return Err(from_error(res).await?);
    }

    Ok(())
}

pub async fn patch<T>(uri: Uri, obj: &T, auth: Option<String>) -> Result<()>
where
    T: Serialize,
{
    let res = request(Method::PATCH, &uri, Some(obj), auth).await?;

    if !res.status().is_success() {
        return Err(from_error(res).await?);
    }

    Ok(())
}

pub async fn delete(uri: Uri, auth: Option<String>) -> Result<()>
{
    let res = request(Method::DELETE, &uri, None::<&str>, auth).await?;

    if !res.status().is_success() {
        return Err(from_error(res).await?);
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
    fn uri(path: &str) -> Uri {
        format!("https://dummyjson.com{path}")
            .parse().unwrap()
    }

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
    struct TestData {
        payload: String
    }


    async fn test_get() -> Result<()> {
        let test = get::<TestStatus>(uri("/test"), None).await?.unwrap();
        assert_eq!(Status::Ok, test.status);
        assert_eq!("GET", test.method);
        Ok(())
    }

    async fn test_get_418() -> Result<()> {
        let result = get::<TestStatus>(uri("/http/418"), None).await;
        if let Err(Error::HttpError(msg)) = result {
            assert!(msg.contains("I'm a teapot"))
        } else {
            panic!("Expected error: {result:?}");
        }

        Ok(())
    }

    async fn test_put() -> Result<()> {
        let data = TestData { payload: "test".to_string() };
        put::<TestData>(uri("/test"), &data, None).await?;
        Ok(())
    }

    async fn test_post() -> Result<()> {
        let data = TestData { payload: "test".to_string() };
        post::<TestData>(uri("/test"), &data, None).await?;
        Ok(())
    }


    #[cfg(feature = "smol")]
    #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
    mod smol_tests {
        use super::*;
        use macro_rules_attribute::apply;
        use smol_macros::test;

        #[apply(test!)]
        #[traced_test]
        async fn smol_get() -> Result<()> {
            test_get().await
        }

        #[apply(test!)]
        #[traced_test]
        async fn smol_get_418() -> Result<()> {
            test_get_418().await
        }

        #[apply(test!)]
        #[traced_test]
        async fn smol_put() -> Result<()> {
            test_put().await
        }

        #[apply(test!)]
        #[traced_test]
        async fn smol_post() -> Result<()> {
            test_post().await
        }
    }

    #[cfg(feature = "tokio")]
    #[cfg_attr(feature = "test_offline", ignore = "Online test skipped")]
    mod tokio_tests {
        use super::*;

        #[tokio::test]
        #[traced_test]
        async fn tokio_get() -> Result<()> {
            test_get().await
        }

        #[tokio::test]
        #[traced_test]
        async fn tokio_get_418() -> Result<()> {
            test_get_418().await
        }

        #[tokio::test]
        #[traced_test]
        async fn tokio_put() -> Result<()> {
            test_put().await
        }

        #[tokio::test]
        #[traced_test]
        async fn tokio_post() -> Result<()> {
            test_post().await
        }
    }
}
