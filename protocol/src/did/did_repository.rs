use http::StatusCode;

use super::sidetree::{
    client::SidetreeHttpClient,
    payload::{did_create_payload, DidDocument, DidPatchDocument, MiaxDidResponse, ToPublicKey},
};
use crate::keyring::{
    jwk::Jwk,
    keypair::{KeyPair, KeyPairing},
};

// ”protocol”クレートはライブラリとして利用されることを想定しているため、anyhowは使用しない

#[derive(Debug, thiserror::Error)]
pub enum CreateIdentifierError<StudioClientError: std::error::Error> {
    #[error("Failed to convert to JWK: {0}")]
    Jwk(#[from] crate::keyring::jwk::K256ToJwkError),
    #[error("Failed to build operation payload: {0}")]
    PayloadBuildFailed(#[from] crate::did::sidetree::payload::DidCreatePayloadError),
    #[error("Failed to parse body: {0}")]
    BodyParse(#[from] serde_json::Error),
    #[error("Failed to create identifier. response: {0}")]
    SidetreeRequestFailed(String),
    #[error("Failed to send request: {0}")]
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

#[derive(Debug, thiserror::Error)]
pub enum GetPublicKeyError {
    #[error("Failed to get public key: {0}")]
    PublicKeyNotFound(String),
    #[error("Failed to convert from JWK: {0}")]
    JwkToK256(#[from] crate::keyring::jwk::JwkToK256Error),
    #[error("Failed to convert from JWK: {0}")]
    JwkToX25519(#[from] crate::keyring::jwk::JwkToX25519Error),
}

fn get_key(key_type: &str, did_document: &DidDocument) -> Result<Jwk, GetPublicKeyError> {
    let did = &did_document.id;
    let public_key = did_document
        .public_key
        .clone()
        .and_then(|pks| pks.into_iter().find(|pk| pk.id == key_type))
        .ok_or(GetPublicKeyError::PublicKeyNotFound(did.to_string()))?;
    Ok(public_key.public_key_jwk)
}

pub fn get_encrypt_key(
    did_document: &DidDocument,
) -> Result<x25519_dalek::PublicKey, GetPublicKeyError> {
    let public_key = get_key("#encryptionKey", did_document)?;
    Ok(public_key.try_into()?)
}

// Send: ある型Tが”スレッド間で安全に所有権を移動できること"を示す
// Sync: ある型が"スレッド間で安全に参照できること"を示す

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
    ) -> Result<MiaxDidResponse, CreateIdentifierError<C::Error>> {
        let sign = keyring.sign.get_public_key().to_public_key(
            "EcdsaSecp256k1VerificationKey2019".to_string(),
            "signingKey".to_string(),
            vec!["auth".to_string(), "general".to_string()],
        )?;

        let enc = keyring
            .encrypt
            .get_public_key()
            .to_public_key(
                "X25519KeyAgreementKey2019".to_string(),
                "encryptionKey".to_string(),
                vec!["auth".to_string(), "general".to_string()],
            )
            .unwrap();
        let update = keyring.update.get_public_key();
        let recovery = keyring.recovery.get_public_key();
        let document = DidPatchDocument {
            public_keys: vec![sign, enc],
            service_endpoints: vec![],
        };
        let payload = did_create_payload(document, update, recovery)?;

        let response = self
            .client
            .post_create_identifier(&payload)
            .await
            .map_err(CreateIdentifierError::SidetreeHttpClient)?;
        if response.status_code.is_success() {
            Ok(serde_json::from_str(&response.body)?)
        } else {
            Err(CreateIdentifierError::SidetreeRequestFailed(format!(
                "{:?}",
                response
            )))
        }
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
