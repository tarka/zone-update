use std::{env, net::SocketAddr, time::Duration};

use anyhow::Result;
use dnsclient::{r#async::DNSClient, UpstreamServer};
use instant_acme::{Account, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, OrderStatus, RetryPolicy};
use random_string::charsets::ALPHA_LOWER;
use tracing::{info, level_filters::LevelFilter};
use zone_update::{Config, Provider, async_impl::AsyncDnsProvider, porkbun::Auth};


fn dns_client(domain: String, key: String, secret: String) -> Result<Box<dyn AsyncDnsProvider>> {
    let auth = Auth {
        key,
        secret,
    };
    let config = Config {
        domain: domain,
        dry_run: false,
    };
    let dns_client = Provider::PorkBun(auth)
        .async_impl(config);

    Ok(dns_client)
}


async fn get_cert() -> Result<()> {
    info!("Starting get_cert()");

    let hostname = random_string::generate(16, ALPHA_LOWER);
    //let hostname = "plug-01";
    let domain = env::var("PORKBUN_TEST_DOMAIN").unwrap();
    let fqdn = format!("{hostname}.{domain}");
    let contact = format!("mailto:{}", env::var("EXAMPLE_EMAIL").unwrap());
    let txt_name = format!("_acme-challenge.{hostname}");

    let dns_secret = env::var("PORKBUN_SECRET")?;

    let dns_key = env::var("PORKBUN_KEY")?;


    info!("Initialising account");
    let (account, _credentials) = Account::builder()?
        .create(
            &NewAccount {
                contact: &[&contact],
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            LetsEncrypt::Staging.url().to_owned(),
            None,
        )
        .await?;

    let hid = Identifier::Dns(fqdn.clone());
    info!("Create order for {hid:?}");
    let mut order = account.new_order(&NewOrder::new(&[hid])).await?;

    let dns_client = dns_client(domain.clone(), dns_key.clone(), dns_secret.clone())?;

    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result?;
        info!("Processing {:?}", authz.status);
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            _ => todo!(),
        }

        info!("Creating challenge");
        let mut challenge = authz
            .challenge(ChallengeType::Dns01)
            .ok_or_else(|| anyhow::anyhow!("no dns01 challenge found"))?;


        let token = challenge.key_authorization().dns_value();

        info!("Creating TXT: {txt_name} -> {}", token);

        dns_client.create_txt_record(&txt_name, &token).await?;

        let txt_fqdn = format!("{txt_name}.{domain}");

        //let lookup = DNSClient::new_with_system_resolvers()?;
        let upstream = UpstreamServer::new(SocketAddr::from(([1,1,1,1], 53)));
        let lookup = DNSClient::new(vec![upstream]);
        info!("Waiting for record to go live");
        for _i in 0..30 {
            info!("Lookup for {txt_fqdn}");
            let txts = lookup.query_txt(&txt_fqdn).await?;
            if txts.len() > 0 {
                info!("Found {txt_fqdn}");
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        info!("Setting challenge to ready");
        challenge.set_ready().await?;
    }

    info!("Polling challenge status");
    let status = order.poll_ready(&RetryPolicy::default()).await?;
    if status != OrderStatus::Ready {
        dns_client.delete_txt_record(&txt_name).await?;
        return Err(anyhow::anyhow!("unexpected order status: {status:?}"));
    }

    let private_key_pem = order.finalize().await?;
    let cert_chain_pem = order.poll_certificate(&RetryPolicy::default()).await?;

    // Finally download the certificate.
    println!("======================= Cert ===============================\n");
    println!("{}", cert_chain_pem);
    println!("====================== Private =============================\n");
    println!("{}", private_key_pem);

    println!("Deleting acme challenge");

    dns_client.delete_txt_record(&txt_name).await?;

    println!("Done");

    Ok(())
}


fn main() -> Result<()> {
    let trace_fmt = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .finish();
    tracing::subscriber::set_global_default(trace_fmt)?;

    let _cert = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(
            get_cert()
        )?;

    Ok(())
}
