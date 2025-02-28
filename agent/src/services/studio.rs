use crate::config::server_config;
use crate::miax::utils::did_accessor::DidAccessorImpl;
use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::miax::utils::studio_client::{StudioClient, StudioClientConfig};
use protocol::did::did_repository::DidRepositoryImpl;
use serde::Deserialize;

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
}
