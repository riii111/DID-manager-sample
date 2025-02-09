use anyhow::Context;
use chrono::{format::parse, ParseError};
use protocol::did::sidetree::client::{SidetreeHttpClient, SidetreeHttpClientResponse};

#[derive(Clone)]
pub struct SideTreeClient {
    base_url: Url,
    client: reqwest::Client,
}

impl SideTreeClient {
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        let base_url =
            Url::parse(base_url).context("MIAX_DID_HTTP_ENDPOINT must be a valid URL")?;
        Ok(Self {
            base_url,
            client: reqwest::Client::new(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SideTreeClientError {
    #[error("parse error {0}")]
    ParseError(#[from] ParseError),
    #[error("reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
}

impl SidetreeHttpClient for SideTreeClient {
    type Error = SideTreeClientError;
    async fn post_create_identifier(
        &self,
        body: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error> {
        unimplemented!("post_create_identifier");
    }

    async fn get_find_identifier(
        &self,
        did: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error> {
        unimplemented!("get_find_identifier")
    }
}
