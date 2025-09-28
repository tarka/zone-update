use std::{env, time::Duration};

use acme_micro::{create_p384_key, Certificate, Directory, DirectoryUrl};
use anyhow::Result;
use zone_edit::{gandi::{Auth, Gandi}, Config, DnsProvider};


fn get_dns_client() -> Result<impl DnsProvider> {

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

    Ok(Gandi {
        config,
        auth,
    })
}

async fn get_cert() -> Result<Certificate> {
    println!("Starting get_cert()");

    let dns_client = get_dns_client()?;

    // The following is based on the acme-micro example. In practice
    // most of the operations should be wrapped in `blocking()` (as
    // acme-micro uses the synchronous ureq crate).

    let dir = Directory::from_url(DirectoryUrl::LetsEncryptStaging)?;
    let contact = vec!["mailto:ssmith@haltcondition.net".to_string()];

    println!("Registering account");
    let acc = dir.register_account(contact.clone())?;

    println!("Place order");
    let mut ord_new = acc.new_order("test.haltcondition.net", &[])?;
    let host = format!("_acme-challenge.test");

    let ord_csr = loop {

        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        let auths = ord_new.authorizations()?;

        let chall = auths[0].dns_challenge().unwrap();

        // The token is the filename.
        let token = chall.dns_proof()?;
        println!("Challenge string is {token}");

        println!("Creating challenge TXT record");
        dns_client.create_txt_record(&host, &token).await?;

        println!("Validating");
        chall.validate(Duration::from_millis(5000))?;

        // Update the state against the ACME API.
        ord_new.refresh()?;
    };

    let pkey_pri = create_p384_key()?;
    let ord_cert = ord_csr.finalize_pkey(pkey_pri, Duration::from_millis(5000))?;

    // Finally download the certificate.
    let cert = ord_cert.download_cert()?;
    println!("{cert:?}");

    println!("Deleting acme challenge");

    dns_client.delete_txt_record(&host).await?;

    println!("Done");

    Ok(cert)
}


fn main() -> Result<()> {

    #[cfg(feature = "smol")]
    smol::block_on(
        get_cert()
    )?;

    #[cfg(feature = "tokio")]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(
            get_cert()
        )?;

    Ok(())
}

