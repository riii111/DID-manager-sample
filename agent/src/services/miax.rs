use crate::app_config;
use crate::config::server_config;
use crate::miax::extension::secure_keystore::FileBaseKeyStore;
use crate::miax::keyring;
use crate::miax::utils::sidetree_client::SideTreeClient;
use controller::managers::{
    resource::ResourceManagerTrait,
    runtime::{RuntimeManagerImpl, RuntimeManagerWithoutAsync, State},
};
use controller::validator::storage::check_storage;
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

        // キーペアを保存しDIDを返却
        let res = self
            .did_repository
            .create_identifier(keyring_with_config.get_keyring())
            .await?;
        keyring_with_config.save(&res.did_document.id);

        Ok(res)
    }

    pub async fn find_identifier(&self, did: &str) -> anyhow::Result<Option<MiaxDidResponse>> {
        let res = self.did_repository.find_identifier(did).await?;

        Ok(res)
    }

    pub async fn update_version(&self, binary_url: &str) -> anyhow::Result<()> {
        #[cfg(windows)]
        {
            unimplemented!();
        }

        #[cfg(unix)]
        {
            let handler =
                controller::managers::mmap_storage::MmapHandler::new("nodex_runtime_info")?;
            let mut runtime_manager = RuntimeManagerImpl::new_by_agent(
                handler,
                controller::managers::unix_process_manager::UnixProcessManager,
            );
            let agent_path = &runtime_manager.get_runtime_info()?.exec_path;
            let output_path = agent_path
                .parent()
                .ok_or(anyhow::anyhow!("Failed to get path of parent directory"))?;
            if !check_storage(output_path) {
                log::error!("Not enough storage space: {:?}", output_path);
                anyhow::bail!("Not enough storage space");
            }
            let resource_manager =
                controller::managers::resource::UnixResourceManager::new(agent_path);

            resource_manager.backup().map_err(|e| {
                log::error!("Failed to backup: {}", e);
                anyhow::anyhow!(e)
            })?;

            resource_manager
                .download_update_resources(binary_url, Some(output_path))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            runtime_manager.launch_controller(agent_path)?;
            runtime_manager.update_state(State::Update)?;
        }

        Ok(())
    }
}
