use http::StatusCode;

#[derive(Clone, Debug)]
pub struct SidetreeHttpClientResponse {
    pub(crate) status_code: StatusCode, // pub(crate): クレート内でのみpublic, 外部クレートからはprivate
    pub(crate) body: String,
}

impl SidetreeHttpClientResponse {
    pub fn new(status_code: StatusCode, body: String) -> Self {
        Self { status_code, body }
    }
}

#[trait_variant::make(Send)]
/// Sidetree HTTPクライアントインターフェース
///
/// Sidetreeプロトコルで定義された操作をHTTPリクエストとして送信し、
/// Sidetreeネットワークと通信するためのインターフェースを定義する
pub trait SidetreeHttpClient {
    type Error: std::error::Error;
    /// Sidetree create operation を実行し、新しいDIDをSidetreeネットワークに登録
    async fn post_create_identifier(
        &self,
        body: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error>;
    /// Sidetree resolve operation を実行し、DID識別子に対応するDIDドキュメントをSidetreeネットワークから取得
    async fn get_find_identifier(
        &self,
        did: &str,
    ) -> Result<SidetreeHttpClientResponse, Self::Error>;
}
