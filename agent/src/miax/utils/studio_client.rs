use super::did_accessor::DidAccessorImpl;
use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::server_config;
use protocol::did::did_repository::DidRepositoryImpl;
use protocol::didcomm::encrypted::DidCommServiceWithAttachment;
use reqwest::Url;

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
        let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint());
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
}
