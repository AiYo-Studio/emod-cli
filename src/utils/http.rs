use reqwest::blocking::Client;
use crate::error::Result;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder().build()?;
        Ok(Self { client })
    }

    pub fn new_with_proxy(proxy_url: &str) -> Result<Self> {
        let proxy = reqwest::Proxy::all(proxy_url)?;
        let client = Client::builder().proxy(proxy).build()?;
        Ok(Self { client })
    }

    pub fn get(&self, url: &str) -> Result<reqwest::blocking::Response> {
        Ok(self.client
            .get(url)
            .header("User-Agent", "emod-cli")
            .send()?)
    }
}
