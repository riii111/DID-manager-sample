use crate::app_config;
use crate::controllers::errors::MiaXErrorCode;
use crate::controllers::public::miax_create_identifier::MiaxDidResponse;
use crate::miax::extension::secure_keystore::FileBaseKeyStore;
use crate::miax::keyring;

pub struct MiaX {
  did_repository: DidRepositoryImpl<SideTreeClient>.
}

impl MiaX {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_identifier(&self) -> Result<MiaxDidResponse, MiaXErrorCode> {
        //  設定とキーストアの準備
        let config = app_config();
        let keystore = FileBaseKeyStore::new(config.clone());

        // 既存のDIDがあるかチェック
        if let Some(did) =
            keyring::keypair::KeyPairWithConfig::load_keyring(config.clone(), keystore.clone())
                .ok()
                .and_then(|v| v.get_identifier().ok())
        {
            unimplemented!("call find_identifier")
        }

        // 新規DIDを生成
        // キーペアを保存しDIDを変革
        todo!("unimplemented")
    }

    pub async fn find_identifier(&self, did: &str) -> anyhow::Result<Option<MiaxDidResponse>> {
        let res = self.did_repository.find_identifier(did).await?;

        Ok(res)
    }
}
