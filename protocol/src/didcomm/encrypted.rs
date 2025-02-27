use crate::did::did_repository::{DidRepository, GetPublicKeyError, get_encrypt_key};
use crate::did::sidetree::payload::DidDocument;
use crate::didcomm::types::DidCommMessage;
use crate::keyring::keypair::KeyPair;
use crate::keyring::keypair::KeyPairing;
use crate::verifiable_credentials::did_vc::DidVcService;
use crate::verifiable_credentials::types::{VerifiableCredentials, VerifiedContainer};
use cuid;
pub use didcomm_rs;
use didcomm_rs::{AttachmentBuilder, AttachmentDataBuilder, Message, crypto::CryptoAlgorithm};
use serde_json::Value;
use thiserror::Error;

#[trait_variant::make(Send)]
pub trait DidCommEncryptedService: Sync {
    type GenerateError: std::error::Error;
    type VerifyError: std::error::Error;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError>;

    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError>;
}

fn didcomm_generate<R: DidRepository, V: DidVcService>(
    body: &VerifiableCredentials,
    from_keyring: &KeyPairing,
    to_doc: &DidDocument,
    metadata: Option<&Value>,
    attachment_link: Option<&str>,
) -> Result<
    DidCommMessage,
    DidCommEncryptedServiceGenerateError<R::FindIdentifierError, V::GenerateError>,
> {
    let to_did = &to_doc.id;
    let from_did = &body.issuer.id;
    let body = serde_json::to_string(body)?;

    let mut message = Message::new().from(from_did).to(&[to_did]).body(&body)?;

    if let Some(value) = metadata {
        let id = cuid::cuid2();

        let data = AttachmentDataBuilder::new().with_json(&value.to_string());

        let data = if let Some(attachment_link) = attachment_link {
            data.with_link(attachment_link)
        } else {
            data
        };

        message.append_attachment(
            AttachmentBuilder::new(true)
                .with_id(&id)
                .with_format("metadata")
                .with_data(data),
        )
    }

    let public_key = get_encrypt_key(to_doc)?.as_bytes().to_vec();
    let public_key = Some(public_key);

    let seal_message = message
    .as_jwe(&CryptoAlgorithm::XC20P, public_key.clone())
    .seal(from_keyring.encrypt.get_secret_key().as_bytes(), Some(vec![public_key]))?;

    Ok(serde_json::from_str::<DidCommMessage>(&seal_message)?)
}

async fn generate<R: DidRepository, V: DidVcService>(
    did_repository: &R,
    vc_service: &V,
    model: VerifiableCredentials,
    from_keyring: &KeyPairing,
    to_did: &str,
    metadata: Option<&Value>,
    attachment_link: Option<&str>,
) -> Result<
    DidCommMessage,
    DidCommEncryptedServiceGenerateError<R::FindIdentifierError, V::GenerateError>,
> {
    let body = vc_service
        .generate(model, from_keyring)
        .map_err(DidCommEncryptedServiceGenerateError::VcService)?;
    let to_doc = did_repository
        .find_identifier(to_did)
        .await
        .map_err(DidCommEncryptedServiceGenerateError::SidetreeFindRequestFailed)?;
        .ok_or(DidCommEncryptedServiceGenerateError::DidDocNotFound(
            to_did.to_string()
        ))?
        .did_document;

    didcomm_generate::<R, V>(
        &body,
        from_keyring,
        &to_doc,
        metadata,
        attachment_link,
    )
}

#[derive(Debug, Error)]
pub enum DidCommEncryptedServiceGenerateError<FindIdentifierError, CredentialSignerSignError>
where
    FindIdentifierError: std::error::Error,
    CredentialSignerSignError: std::error::Error,
{
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("something went wrong with vc service: {0}")]
    VcService(CredentialSignerSignError),
    #[error("failed to create identifier: {0}")]
    SidetreeFindRequestFailed(FindIdentifierError),
    #[error("failed to encrypt message with error: {0}")]
    EncryptFailed(#[from] didcomm_rs::Error),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
}

impl<R> DidCommEncryptedService for R
where
    R: DidRepository + DidVcService,
{
    type GenerateError =
        DidCommEncryptedServiceGenerateError<R::FindIdentifierError, R::GenerateError>;
    type VerifyError = DidCommEncryptedServiceVerifyError<R::FindeIdentifierError>;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError> {
        generate::<R, R>(self, self, model, from_keyring, to_did, metadata, None).await
    }
    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError> {
        verify(self, my_keyring, message).await
    }
}

pub struct DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    vc_service: R,
    attachment_link: String,
}

impl<R> DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    pub fn new(did_repository: R, attachment_link: String) -> Self {
        Self {
            vc_service: did_repository,
            attachment_link,
        }
    }
}

impl<R> DidCommEncryptedService for DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    type GenerateError =
        DidCommEncryptedServiceGeneratedError<R::FindIdentifierError, R::GenerateError>;
}
