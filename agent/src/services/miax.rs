use crate::app_config;
use crate::config::server_config;
use crate::miax::extension::secure_keystore::FileBaseKeyStore;
use crate::miax::keyring;
use crate::miax::utils::sidetree_client::SideTreeClient;
use protocol::did::did_repository::{DidRepository, DidRepositoryImpl};

use protocol::did::sidetree::payload::MiaxDidResponse;

pub struct MiaX {
    did_repository: DidRepositoryImpl<SideTreeClient>,
}

impl MiaX {
    pub fn new() -> Self {
        let server_config = server_config();
        let sidetree_client = SideTreeClient::new(&server_config.did_http_endpoint()).unwrap();
        let did_repository = DidRepositoryImpl::new(sidetree_client);

        MiaX { did_repository }
    }

    pub fn did_repository(&self) -> &DidRepositoryImpl<SideTreeClient> {
        &self.did_repository
    }

    pub async fn create_identifier(&self) -> anyhow::Result<MiaxDidResponse> {
        //  設定とキーストアの準備
        let config = app_config();
        let keystore = FileBaseKeyStore::new(config.clone());

        // 既存のDIDがあるかチェック
        if let Some(did) =
            keyring::keypair::KeyPairingWithConfig::load_keyring(config.clone(), keystore.clone())
                .ok()
                .and_then(|v| v.get_identifier().ok())
        {
            if let Some(json) = self.find_identifier(&did).await? {
                return Ok(json);
            }
        }

        // 新規DIDを生成
        let mut keyring_with_config =
            keyring::keypair::KeyPairingWithConfig::create_keyring(config, keystore);

        let res = self
            .did_repository
            .create_identifier(keyring_with_config.get_keyring())
            .await?;
        // キーペアを保存しDIDを変革
        unimplemented!("save keypair")
    }

    pub async fn find_identifier(&self, did: &str) -> anyhow::Result<Option<MiaxDidResponse>> {
        let res = self.did_repository.find_identifier(did).await?;

        Ok(res)
    }
}
