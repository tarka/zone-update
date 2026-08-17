use anyhow::Result;
use serde::Deserialize;

mod sync {
    use super::*;

    use zone_update::DnsProvider;
    #[cfg(feature = "dnsimple")]
    use zone_update::dnsimple;
    #[cfg(feature = "dnsmadeeasy")]
    use zone_update::dnsmadeeasy;
    #[cfg(feature = "gandi")]
    use zone_update::gandi;
    #[cfg(feature = "porkbun")]
    use zone_update::porkbun;

    /// Test helper enum describing provider configurations.
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase", tag = "name")]
    pub enum Providers {
        #[cfg(feature = "gandi")]
        Gandi(gandi::Auth),
        #[cfg(feature = "dnsimple")]
        Dnsimple(dnsimple::Auth),
        #[cfg(feature = "dnsmadeeasy")]
        DnsMadeEasy(dnsmadeeasy::Auth),
        #[cfg(feature = "porkbun")]
        PorkBun(porkbun::Auth),
    }


    /// Helper to construct a synchronous DNS provider from the test enum.
    pub fn get_dns_provider(pe: Providers) -> Result<Box<dyn DnsProvider>> {

        let dns_conf = zone_update::Config {
            domain: "example.com".to_string(),
            dry_run: false,
        };

        let provider: Box<dyn DnsProvider> = match pe {
            #[cfg(feature = "gandi")]
            Providers::Gandi(auth) => Box::new(gandi::Gandi::new(dns_conf, auth)),
            #[cfg(feature = "dnsimple")]
            Providers::Dnsimple(auth) => Box::new(dnsimple::Dnsimple::new(dns_conf, auth, None)),
            #[cfg(feature = "dnsmadeeasy")]
            Providers::DnsMadeEasy(auth) => Box::new(dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth)),
            #[cfg(feature = "porkbun")]
            Providers::PorkBun(auth) => Box::new(porkbun::Porkbun::new(dns_conf, auth)),
        };

        Ok(provider)
    }

    #[test]
    fn test_get_providers() -> Result<()> {
        let pe = Providers::PorkBun(porkbun::Auth{
            key: "a_key".to_string(),
            secret: "a_secret".to_string(),
        });

        let p = get_dns_provider(pe)?;
        let _r = p.get_a_record("test");
        Ok(())
    }

}


#[cfg(feature = "async")]
mod r#async {
    use super::*;
    use zone_update::async_impl::AsyncDnsProvider;
    #[cfg(feature = "dnsimple")]
    use zone_update::async_impl::dnsimple;
    #[cfg(feature = "dnsmadeeasy")]
    use zone_update::async_impl::dnsmadeeasy;
    #[cfg(feature = "gandi")]
    use zone_update::async_impl::gandi;
    #[cfg(feature = "porkbun")]
    use zone_update::async_impl::porkbun;

    /// Test helper enum describing provider configurations for async tests.
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase", tag = "name")]
    pub enum Providers {
        #[cfg(feature = "gandi")]
        Gandi(gandi::Auth),
        #[cfg(feature = "dnsimple")]
        Dnsimple(dnsimple::Auth),
        #[cfg(feature = "dnsmadeeasy")]
        DnsMadeEasy(dnsmadeeasy::Auth),
        #[cfg(feature = "porkbun")]
        PorkBun(porkbun::Auth),
    }


    /// Helper to construct an async DNS provider from the test enum.
    pub fn get_dns_provider(pe: Providers) -> Result<Box<dyn AsyncDnsProvider>> {

        let dns_conf = zone_update::Config {
            domain: "example.com".to_string(),
            dry_run: false,
        };

        let provider: Box<dyn AsyncDnsProvider> = match pe {
            #[cfg(feature = "gandi")]
            Providers::Gandi(auth) => Box::new(gandi::Gandi::new(dns_conf, auth)),
            #[cfg(feature = "dnsimple")]
            Providers::Dnsimple(auth) => Box::new(dnsimple::Dnsimple::new(dns_conf, auth, None)),
            #[cfg(feature = "dnsmadeeasy")]
            Providers::DnsMadeEasy(auth) => Box::new(dnsmadeeasy::DnsMadeEasy::new(dns_conf, auth)),
            #[cfg(feature = "porkbun")]
            Providers::PorkBun(auth) => Box::new(porkbun::Porkbun::new(dns_conf, auth)),
        };

        Ok(provider)
    }

    #[test]
    fn test_get_providers() -> Result<()> {
        let pe = Providers::PorkBun(porkbun::Auth{
            key: "a_key".to_string(),
            secret: "a_secret".to_string(),
        });

        let p = get_dns_provider(pe)?;
        let h = "test".to_string();
        let _r = p.get_a_record(&h);
        Ok(())
    }

}
