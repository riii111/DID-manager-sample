use super::did_accessor::{DidAccessor, DidAccessorImpl};
use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::server_config;
use chrono::Utc;
use protocol::did::did_repository::DidRepositoryImpl;
use protocol::didcomm::encrypted::{DidCommEncryptedService, DidCommServiceWithAttachment};
use protocol::verifiable_credentials::types::VerifiableCredentials;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Url,
};
use serde_json::json;

pub struct StudioClientConfig {
    pub base_url: String,
}

pub struct StudioClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
    pub didcomm_service: DidCommServiceWithAttachment<DidRepositoryImpl<SideTreeClient>>,
    pub did_accessor: DidAccessorImpl,
}

impl StudioClient {
    pub fn new(_config: &StudioClientConfig) -> anyhow::Result<Self> {
        let url = Url::parse(&_config.base_url.to_string())?;
        let client = reqwest::Client::new();
        let server_config = server_config();
        let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint())?;
        let did_repository = DidRepositoryImpl::new(sidetree_client);
        let didcomm_service =
            DidCommServiceWithAttachment::new(did_repository, server_config.did_attachment_link());
        let did_accessor = DidAccessorImpl {};

        Ok(StudioClient {
            instance: client,
            base_url: url,
            didcomm_service,
            did_accessor,
        })
    }

    pub async fn post(&self, path: &str, body: &str) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let response = self
            .instance
            .post(url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await?;

        Ok(response)
    }

    pub async fn get_message(
        &self,
        path: &str,
        project_did: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let my_did = self.did_accessor.get_my_did();
        let my_keyring = self.did_accessor.get_my_keyring();

        let model = VerifiableCredentials::new(my_did, serde_json::Value::Null, Utc::now());
        let payload = self
            .didcomm_service
            .generate(model, &my_keyring, project_did, None)
            .await?;
        let payload = serde_json::to_string(&payload)?;
        let url = self.base_url.join(path)?;
        self.post(url.as_ref(), &payload).await
    }

    pub async fn ack_message(
        &self,
        path: &str,
        project_did: &str,
        message_id: String,
        is_verified: bool,
    ) -> anyhow::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let payload = json!({
            "message_id": message_id,
            "is_verified": is_verified,
        });
        let my_did = self.did_accessor.get_my_did();
        let my_keyring = self.did_accessor.get_my_keyring();

        let model = VerifiableCredentials::new(my_did, payload, Utc::now());
        let payload = self
            .didcomm_service
            .generate(model, &my_keyring, project_did, None)
            .await?;

        let payload = serde_json::to_string(&payload)?;
        self.post(url.as_ref(), &payload).await
    }

    pub async fn network(
        &self,
        path: &str,
        project_did: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let my_did = self.did_accessor.get_my_did();
        let my_keyring = self.did_accessor.get_my_keyring();

        let model = VerifiableCredentials::new(my_did, serde_json::Value::Null, Utc::now());
        let payload = self
            .didcomm_service
            .generate(model, &my_keyring, project_did, None)
            .await?;
        let payload = serde_json::to_string(&payload)?;
        self.post(path, &payload).await
    }
}
