use super::sidetree::{client::SidetreeHttpClient, payload::MiaxDidResponse};
use crate::keyring::keypair::KeyPairing;
use http::StatusCode;

// ”protocol”クレートはライブラリとして利用されることを想定しているため、anyhowは使用しない

#[derive(Debug, thiserror::Error)]
pub enum CreateIdentifierError<StudioClientError: std::error::Error> {
    // TODO: other error
    #[error("Failed to parse body: {0}")]
    BodyParse(#[from] serde_json::Error),
    #[error("Failed to create identifier. response: {0}")]
    SidetreeRequestFailed(String),
    #[error("Failed to send requests: {0}")]
    SidetreeHttpClient(StudioClientError),
}

#[derive(Debug, thiserror::Error)]
pub enum FindIdentifierError<StudioClientError: std::error::Error> {
    #[error("Failed to send request to sidetree: {0}")]
    SidetreeRequestFailed(String),
    #[error("Failed to parse body: {0}")]
    BodyParse(#[from] serde_json::Error),
    #[error("Failed to send request: {0}")]
    SidetreeHttpClient(StudioClientError),
}

// Send: ある型Tが”スレッド間で安全に所有権を移動できること"を示す
// Sync: ある型が"スレッド間で安全に参照できること"を示すs

// DID Repositoryのインターフェース定義
#[trait_variant::make(Send)] // トレイト自体がSendであり、トレイトオブジェクト（dyn Trait）が"Send + Sync + 'static"であることを保証
pub trait DidRepository: Sync {
    type CreateIdentifierError: std::error::Error + Send + Sync;
    type FindIdentifierError: std::error::Error + Send + Sync;
    async fn create_identifier(
        &self,
        keyring: KeyPairing,
    ) -> Result<MiaxDidResponse, Self::CreateIdentifierError>;
    async fn find_identifier(
        &self,
        did: &str,
    ) -> Result<Option<MiaxDidResponse>, Self::FindIdentifierError>;
}

// Sidetreeプロトコルとの通信
#[derive(Clone)]
pub struct DidRepositoryImpl<C: SidetreeHttpClient> {
    client: C,
}

impl<C: SidetreeHttpClient> DidRepositoryImpl<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> DidRepository for DidRepositoryImpl<C>
where
    C: SidetreeHttpClient + Send + Sync,
    C::Error: Send + Sync,
{
    type CreateIdentifierError = CreateIdentifierError<C::Error>;
    type FindIdentifierError = FindIdentifierError<C::Error>;

    async fn create_identifier(
        &self,
        keyring: KeyPairing,
    ) -> Result<MiaxDidResponse, Self::CreateIdentifierError> {
        unimplemented!("create_identifier")
    }

    async fn find_identifier(
        &self,
        did: &str,
    ) -> Result<Option<MiaxDidResponse>, Self::FindIdentifierError> {
        let response = self
            .client
            .get_find_identifier(did)
            .await
            .map_err(FindIdentifierError::SidetreeHttpClient)?;

        match response.status_code {
            StatusCode::OK => Ok(Some(serde_json::from_str(&response.body)?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(FindIdentifierError::SidetreeRequestFailed(format!(
                "{:?}",
                response
            ))),
        }
    }
}
