use std::{env, error::Error};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use lers::{Certificate, Directory, Solver, LETS_ENCRYPT_STAGING_URL};
use random_string::charsets::ALPHANUMERIC;
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::FmtSubscriber;
use zone_update::{Config, Providers, async_impl::AsyncDnsProvider, porkbun::Auth};

struct ZoneUpdateSolver {
    domain: String,
    dns_client: Box<dyn AsyncDnsProvider>,
    requests: papaya::HashMap<String, String>,
}

impl ZoneUpdateSolver {
    fn new(domain: String, key: String, secret: String) -> Result<Self> {
        let auth = Auth {
            key,
            secret,
        };
        let config = Config {
            domain: domain.clone(),
            dry_run: false,
        };
        let dns_client = Providers::PorkBun(auth)
            .async_impl(config);

        Ok(Self {
            domain,
            dns_client,
            requests: papaya::HashMap::new(),
        })
    }
}

#[async_trait]
impl Solver for ZoneUpdateSolver {
    async fn present(&self,
                     cert_fqdn: String,
                     req_id: String,
                     challenge: String)
                     -> Result<(), Box<dyn Error + Send + Sync + 'static>>
    {
        println!("Creating proof for {cert_fqdn}");
        // Lers provides the FQDN challenge, but many DNS APIs only
        // work with unqualified names. Strip off the domain.
        let challenge_host = cert_fqdn.strip_suffix(&format!(".{}",self.domain))
            .unwrap_or(&cert_fqdn);
        let txt_name = format!("_acme-challenge.{challenge_host}");

        println!("Creating TXT record '{txt_name}' -> '{challenge}");

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

        println!("Deleting TXT record '{txt_name}'");
//        self.dns_client.delete_txt_record(&txt_name).await?;

        Ok(())
    }
}

async fn get_cert() -> Result<Certificate> {
    println!("Starting get_cert()");

    let hostname = random_string::generate(16, ALPHANUMERIC);
    let domain = env::var("PORKBUN_TEST_DOMAIN").unwrap();
    let fqdn = format!("{hostname}.{domain}");
    let email = format!("mailto:{}", env::var("EXAMPLE_EMAIL").unwrap());

    let dns_key = env::var("PORKBUN_KEY")?;
    let dns_secret = env::var("PORKBUN_SECRET")?;

    let solver = ZoneUpdateSolver::new(domain, dns_key, dns_secret)?;

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

    println!("Starting req for {fqdn}");
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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    // let trace_fmt = tracing_subscriber::fmt()
    //     .with_max_level(LevelFilter::INFO)
    //     .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let _cert = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(
            get_cert()
        )?;

    Ok(())
}

