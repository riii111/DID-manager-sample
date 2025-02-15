use protocol::did::did_repository::DidRepositoryImpl;

use crate::miax::utils::sidetree_client::SideTreeClient;

pub struct Studio {
    http_client: StudioClient,
    did_repository: DidRepositoryImpl<SideTreeClient>,
    did_accessor: DidAccessorImpl,
}
