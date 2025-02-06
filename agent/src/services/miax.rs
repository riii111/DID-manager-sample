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
        // 1. 設定取得
        // 2. 既存確認
        // 3. 新規生成
        // 4. リポジトリ呼び出し
        // 5. ほぞん
        todo!("Implement DID generation logic")
    }
}
