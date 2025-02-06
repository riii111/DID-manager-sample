use crate::controllers::errors::MiaXErrorCode;
use crate::controllers::public::miax_create_identifier::MiaxDidResponse;

pub struct MiaX {
    // TODO
}

impl MiaX {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_identifier(&self) -> Result<MiaxDidResponse, MiaXErrorCode> {
        // 設定取得
        // 既存のDIDを探す（load_keyring)
        // 新しい鍵ペアを生成（create_keyring）
        // リポジトリ呼び出し保存
        todo!("Implement DID generation logic")
    }
}
