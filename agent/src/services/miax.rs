use crate::app_config;
use crate::controllers::errors::MiaXErrorCode;
use crate::controllers::public::miax_create_identifier::MiaxDidResponse;
use crate::miax::extension::secure_keystore::FileBaseKeyStore;

pub struct MiaX {
    // TODO
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

        // 新規DIDを生成
        // キーペアを保存しDIDを変革
        todo!("unimplemented")
    }
}
