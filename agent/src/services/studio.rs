use crate::miax::utils::did_accessor::DidAccessorImpl;
use crate::miax::utils::sidetree_client::SideTreeClient;
use crate::miax::utils::studio_client::StudioClient;
use protocol::did::did_repository::DidRepositoryImpl;

pub struct Studio {
    http_client: StudioClient,
    did_repository: DidRepositoryImpl<SideTreeClient>,
    did_accessor: DidAccessorImpl,
}
