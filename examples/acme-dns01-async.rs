use std::{env, time::Duration};

use acme_micro::{create_p384_key, Certificate, Directory, DirectoryUrl};
use anyhow::Result;
use random_string::charsets::ALPHANUMERIC;
use zone_edit::{async_impl::{AsyncDnsProvider, gandi::Gandi}, gandi::Auth, Config};


fn get_dns_client() -> Result<impl AsyncDnsProvider> {
    // Gandi supports 2 types of API key
    let auth = if let Some(key) = env::var("GANDI_APIKEY").ok() {
        Auth::ApiKey(key)
    } else if let Some(key) = env::var("GANDI_PATKEY").ok() {
        Auth::PatKey(key)
    } else {
        panic!("No Gandi auth key set");
    };

    let config = Config {
        domain: env::var("GANDI_TEST_DOMAIN")?,
        dry_run: false,
    };
    let gandi = Gandi::new(config, auth);

    Ok(gandi)
}

async fn get_cert() -> Result<Certificate> {
    println!("Starting get_cert()");

    let dns_client = get_dns_client()?;

    let hostname = random_string::generate(16, ALPHANUMERIC);
    let domain = env::var("GANDI_TEST_DOMAIN").unwrap();
    let email = format!("mailto:{}", env::var("EXAMPLE_EMAIL").unwrap());

    // The following is based on the acme-micro example. In practice
    // most of the operations should be wrapped in `blocking()` as
    // acme-micro uses the synchronous ureq crate, but it doesn't
    // matter here.

    let dir = Directory::from_url(DirectoryUrl::LetsEncryptStaging)?;

    println!("Registering account");
    let acc = dir.register_account(vec![email])?;

    println!("Place order");
    let mut ord_new = acc.new_order(&format!("{hostname}.{domain}"), &[])?;
    let txt_name = format!("_acme-challenge.{hostname}");

    let ord_csr = loop {

        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        let auths = ord_new.authorizations()?;

        let chall = auths[0].dns_challenge().unwrap();

        let token = chall.dns_proof()?;
        println!("Challenge string is {token}");

        println!("Creating challenge TXT record {txt_name}");
        dns_client.create_txt_record(&txt_name, &token).await?;

        println!("Validating");
        chall.validate(Duration::from_millis(5000))?;

        ord_new.refresh()?;
    };

    let pkey_pri = create_p384_key()?;
    let ord_cert = ord_csr.finalize_pkey(pkey_pri, Duration::from_millis(5000))?;

    // Finally download the certificate.
    let cert = ord_cert.download_cert()?;
    println!("======================= Cert ===============================\n");
    println!("{}", cert.certificate());
    println!("====================== Private =============================\n");
    println!("{}", cert.private_key());

    println!("Deleting acme challenge");

    dns_client.delete_txt_record(&txt_name).await?;

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

