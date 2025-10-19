use std::{fmt::Display, net::Ipv4Addr};

use serde::{de::DeserializeOwned, Serialize};

use crate::{errors::Result, strip_quotes, RecordType};


#[cfg(feature = "dnsimple")]
pub mod dnsimple;


#[allow(unused)]
pub trait AsyncDnsProvider {
    fn get_record<T>(&self, rtype: RecordType, host: &String) -> impl Future<Output = Result<Option<T>>>
    where
        T: DeserializeOwned + Send + Sync + 'static;

    fn create_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static;

    fn update_record<T>(&self, rtype: RecordType, host: &String, record: &T) -> impl Future<Output = Result<()>>
    where
        T: Serialize + DeserializeOwned + Display + Clone + Send + Sync + 'static;

    fn delete_record(&self, rtype: RecordType, host: &String) -> impl Future<Output = Result<()>>;


    // Default helper impls

    // We know all the types, and they're enforced above, so this lint
    // doesn't apply here(?)
    #[allow(async_fn_in_trait)]
    async fn get_txt_record(&self, host: &String) -> Result<Option<String>> {
        self.get_record::<String>(RecordType::TXT, host).await
            .map(|opt| opt.map(|s| strip_quotes(&s)))
    }

    #[allow(async_fn_in_trait)]
    async fn create_txt_record(&self, host: &String, record: &String) -> Result<()> {
        self.create_record(RecordType::TXT, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn update_txt_record(&self, host: &String, record: &String) -> Result<()> {
        self.update_record(RecordType::TXT, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn delete_txt_record(&self, host: &String) -> Result<()> {
        self.delete_record(RecordType::TXT, host).await
    }

    #[allow(async_fn_in_trait)]
    async fn get_a_record(&self, host: &String) -> Result<Option<Ipv4Addr>> {
        self.get_record(RecordType::A, host).await
    }

    #[allow(async_fn_in_trait)]
    async fn create_a_record(&self, host: &String, record: &Ipv4Addr) -> Result<()> {
        self.create_record(RecordType::A, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn update_a_record(&self, host: &String, record: &Ipv4Addr) -> Result<()> {
        self.update_record(RecordType::A, host, record).await
    }

    #[allow(async_fn_in_trait)]
    async fn delete_a_record(&self, host: &String) -> Result<()> {
        self.delete_record(RecordType::A, host).await
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use random_string::charsets::ALPHANUMERIC;

    pub async fn test_create_update_delete_ipv4(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let ip: Ipv4Addr = "1.1.1.1".parse()?;
        client.create_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);


        // Update
        let ip: Ipv4Addr = "2.2.2.2".parse()?;
        client.update_record(RecordType::A, &host, &ip).await?;
        let cur = client.get_record(RecordType::A, &host).await?;
        assert_eq!(Some(ip), cur);


        // Delete
        client.delete_record(RecordType::A, &host).await?;
        let del: Option<Ipv4Addr> = client.get_record(RecordType::A, &host).await?;
        assert!(del.is_none());

        Ok(())
    }

    pub async fn test_create_update_delete_txt(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let txt = "a text reference".to_string();
        client.create_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Update
        let txt = "another text reference".to_string();
        client.update_record(RecordType::TXT, &host, &txt).await?;
        let cur: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Delete
        client.delete_record(RecordType::TXT, &host).await?;
        let del: Option<String> = client.get_record(RecordType::TXT, &host).await?;
        assert!(del.is_none());

        Ok(())
    }

    pub async fn test_create_update_delete_txt_default(client: impl AsyncDnsProvider) -> Result<()> {

        let host = random_string::generate(16, ALPHANUMERIC);

        // Create
        let txt = "a text reference".to_string();
        client.create_txt_record(&host, &txt).await?;
        let cur = client.get_txt_record(&host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Update
        let txt = "another text reference".to_string();
        client.update_txt_record(&host, &txt).await?;
        let cur = client.get_txt_record(&host).await?;
        assert_eq!(txt, strip_quotes(&cur.unwrap()));


        // Delete
        client.delete_txt_record(&host).await?;
        let del = client.get_txt_record(&host).await?;
        assert!(del.is_none());

        Ok(())
    }


}
