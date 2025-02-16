use super::did_accessor::DidAccessorImpl;
use crate::miax::utils::sidetree_client::SideTreeClient;
use protocol::did::did_repository::DidRepositoryImpl;
use protocol::didcomm::encrypted::DidCommServiceWithAttachment;
use reqwest::Url;

pub struct StudioClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
    pub didcomm_service: DidCommServiceWithAttachment<DidRepositoryImpl<SideTreeClient>>,
    pub did_accessor: DidAccessorImpl,
}
