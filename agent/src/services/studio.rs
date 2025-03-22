use crate::config::server_config;
use crate::miax::utils::did_accessor::DidAccessorImpl;
use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::miax::utils::studio_client::{StudioClient, StudioClientConfig};
use protocol::did::did_repository::DidRepositoryImpl;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EmptyResponse {}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub id: String,
    pub raw_message: String,
}

#[derive(Deserialize, Debug, Clone)]
struct ErrorResponse {
    pub message: String,
}

pub struct Studio {
    http_client: StudioClient,
    did_repository: DidRepositoryImpl<SideTreeClient>,
    did_accessor: DidAccessorImpl,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkResponse {
    pub secret_key: String,
    pub project_did: String,
    pub recipient_dids: Vec<String>,
    pub studio_endpoint: String,
    pub heartbeat: u64,
}

impl Studio {
    pub fn new() -> Self {
        let server_config = server_config();
        let client_config: StudioClientConfig = StudioClientConfig {
            base_url: server_config.studio_http_endpoint(),
        };
        let client = match StudioClient::new(&client_config) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{:?}", e);
                panic!()
            }
        };

        let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint())
            .expect("failed to create sidetree client");
        let did_repository = DidRepositoryImpl::new(sidetree_client);
        let did_accessor = DidAccessorImpl {};

        Studio {
            http_client: client,
            did_repository,
            did_accessor,
        }
    }

    pub async fn get_message(&self, project_did: &str) -> anyhow::Result<Vec<MessageResponse>> {
        let res = self
            .http_client
            .get_message("/v1/message/list", project_did)
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => match res.json::<Vec<MessageResponse>>().await {
                Ok(v) => Ok(v),
                Err(e) => anyhow::bail!("StatusCode=200, but parse failed. {:?}", e),
            },
            reqwest::StatusCode::BAD_REQUEST => match res.json::<ErrorResponse>().await {
                Ok(v) => anyhow::bail!("StatusCode=400, error message = {:?}", v.message),
                Err(e) => anyhow::bail!("StatusCode=400, but parse failed. {:?}", e),
            },
            other => anyhow::bail!("StatusCode={other}, unexpected response"),
        }
    }

    pub async fn ack_message(
        &self,
        project_did: &str,
        message_id: String,
        is_verified: bool,
    ) -> anyhow::Result<()> {
        let res = self
            .http_client
            .ack_message("/v1/message/ack", project_did, message_id, is_verified)
            .await?;

        res.json::<EmptyResponse>().await?;
        Ok(())
    }
}
