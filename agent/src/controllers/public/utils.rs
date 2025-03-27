use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::server_config;
use anyhow::Context as _;
use protocol::did::did_repository::DidRepositoryImpl;

pub fn did_repository() -> DidRepositoryImpl<SideTreeClient> {
    let server_config = server_config();
    let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint())
        .context("")
        .unwrap();
    DidRepositoryImpl::new(sidetree_client)
}
