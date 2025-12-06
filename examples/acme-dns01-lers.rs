use std::{env, error::Error};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use lers::{Certificate, Directory, LETS_ENCRYPT_STAGING_URL, Solver};
use random_string::charsets::ALPHANUMERIC;
use zone_update::{Config, Providers, async_impl::AsyncDnsProvider, porkbun::Auth};

fn get_dns_client() -> Result<Box<dyn AsyncDnsProvider>> {
    let auth = Auth {
        key: env::var("PORKBUN_KEY")?,
        secret: env::var("PORKBUN_SECRET")?,
    };
    let config = Config {
        domain: env::var("PORKBUN_TEST_DOMAIN")?,
        dry_run: false,
    };
    let provider = Providers::PorkBun(auth)
        .async_impl(config);

    Ok(provider)
}

struct ZoneUpdateSolver {
    dns_client: Box<dyn AsyncDnsProvider>,
    requests: papaya::HashMap<String, String>,
}

impl ZoneUpdateSolver {
    fn new() -> Result<Self> {
        Ok(Self {
            dns_client: get_dns_client()?,
            requests: papaya::HashMap::new(),
        })
    }
}

#[async_trait]
impl Solver for ZoneUpdateSolver {
    async fn present(&self,
                     cert_domain: String,
                     req_id: String,
                     challenge: String)
                     -> Result<(), Box<dyn Error + Send + Sync + 'static>>
    {
        let txt_name = format!("_acme-challenge.{cert_domain}");

        self.dns_client.create_txt_record(&txt_name, &challenge).await?;

        // req_token is used to track multiple simultaneous requests,
        // but in practice most DNS APIs don't support that. But we
        // need to store it for deletion, so let's pretend anyway.
        self.requests.pin()
            .insert(req_id, txt_name);

        Ok(())
    }

    async fn cleanup(&self, req_id: &str) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let txt_name = {
            let pinned = self.requests.pin();
            pinned.get(req_id)
                .ok_or(anyhow!("Failed to find request id {req_id} in cache"))?
                .clone()
        };

        self.dns_client.delete_txt_record(&txt_name).await?;

        Ok(())
    }
}

async fn get_cert() -> Result<Certificate> {
    println!("Starting get_cert()");

    let hostname = random_string::generate(16, ALPHANUMERIC);
    let domain = env::var("PORKBUN_TEST_DOMAIN").unwrap();
    let fqdn = format!("{hostname}.{domain}");
    let email = format!("mailto:{}", env::var("EXAMPLE_EMAIL").unwrap());

    let solver = ZoneUpdateSolver::new()?;

    let directory = Directory::builder(LETS_ENCRYPT_STAGING_URL)
        .dns01_solver(Box::new(solver))
        .build()
        .await?;

    let account = directory
        .account()
        .terms_of_service_agreed(true)
        .contacts(vec![email])
        .create_if_not_exists()
        .await?;

    let cert = account
        .certificate()
        .add_domain(fqdn)
        .obtain()
        .await?;

    let chain = String::from_utf8(cert.fullchain_to_pem()?)?;
    let key = String::from_utf8(cert.private_key_to_pem()?)?;

    // Finally download the certificate.
    println!("======================= Cert ===============================\n");
    println!("{}", chain);
    println!("====================== Private =============================\n");
    println!("{}", key);

    println!("Deleting acme challenge");

    println!("Done");

    Ok(cert)
}


fn main() -> Result<()> {
    smol::block_on(
        get_cert()
    )?;

    // Alternatively...
    //
    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()?
    //     .block_on(
    //         get_cert()
    //     )?;

    Ok(())
}

