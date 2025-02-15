use super::{did_accessor::DidAccessorImpl, sidetree_client::SideTreeClient};
use protocol::did::did_repository::DidRepositoryImpl;
use reqwest::Url;

pub struct StudioClient {
    pub base_url: Url,
    pub instance: reqwest::Client,
    pub didcomm_service: DidCommServiceWithAttachment<DidRepositoryImpl<SideTreeClient>>,
    pub did_accessor: DidAccessorImpl,
}
