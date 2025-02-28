use crate::did::did_repository::{get_encrypt_key, get_sign_key, DidRepository, GetPublicKeyError};
use crate::did::sidetree::payload::DidDocument;
use crate::didcomm::types::{DidCommMessage, FindSenderError};
use crate::keyring::keypair::KeyPair;
use crate::keyring::keypair::KeyPairing;
use crate::verifiable_credentials::{
    credential_signer::CredentialSignerVerifyError,
    did_vc::DidVcService,
    types::{VerifiableCredentials, VerifiedContainer},
};
use cuid;
pub use didcomm_rs;
use didcomm_rs::{crypto::CryptoAlgorithm, AttachmentBuilder, AttachmentDataBuilder, Message};
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
        .seal(
            from_keyring.encrypt.get_secret_key().as_bytes(),
            Some(vec![public_key]),
        )?;

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
        .map_err(DidCommEncryptedServiceGenerateError::SidetreeFindRequestFailed)?
        .ok_or(DidCommEncryptedServiceGenerateError::DidDocNotFound(
            to_did.to_string(),
        ))?
        .did_document;

    didcomm_generate::<R, V>(&body, from_keyring, &to_doc, metadata, attachment_link)
}

fn didcomm_verify<R: DidRepository>(
    from_doc: &DidDocument,
    my_keyring: &KeyPairing,
    message: &DidCommMessage,
) -> Result<VerifiedContainer, DidCommEncryptedServiceVerifyError<R::FindIdentifierError>> {
    let public_key = get_encrypt_key(from_doc)?.as_bytes().to_vec();
    let public_key = Some(public_key);

    let message = Message::receive(
        &serde_json::to_string(&message)?,
        Some(my_keyring.encrypt.get_secret_key().as_bytes().as_ref()),
        public_key,
        None,
    )?;

    let metadata = message.attachment_iter().find(|item| match &item.format {
        Some(value) => value == "metadata",
        None => false,
    });

    let body = message
        .get_body()
        .map_err(|e| DidCommEncryptedServiceVerifyError::MetadataBodyNotFound(Some(e)))?;
    let body = serde_json::from_str::<VerifiableCredentials>(&body)?;

    match metadata {
        Some(metadata) => {
            let metadata = metadata.data.json.as_ref().ok_or(
                DidCommEncryptedServiceVerifyError::MetadataBodyNotFound(None),
            )?;
            let metadata = serde_json::from_str::<Value>(metadata)?;
            Ok(VerifiedContainer {
                message: body,
                metadata: Some(metadata),
            })
        }
        None => Ok(VerifiedContainer {
            message: body,
            metadata: None,
        }),
    }
}

async fn verify<R: DidRepository>(
    did_repository: &R,
    my_keyring: &KeyPairing,
    message: &DidCommMessage,
) -> Result<VerifiedContainer, DidCommEncryptedServiceVerifyError<R::FindIdentifierError>> {
    let other_did = message.find_sendeer()?;
    let other_doc = did_repository
        .find_identifier(&other_did)
        .await
        .map_err(DidCommEncryptedServiceVerifyError::SidetreeFindRequestFailed)?
        .ok_or(DidCommEncryptedServiceVerifyError::DidDocNotFound(
            other_did,
        ))?
        .did_document;
    let mut container = didcomm_verify::<R>(&other_doc, my_keyring, message)?;
    // for performance, call low level api
    let public_key = get_sign_key(&other_doc)?;
    let body = CredentialSigner::verify(container.message, &public_key)?;
    container.message = body;
    Ok(container)
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
    type VerifyError = DidCommEncryptedServiceVerifyError<R::FindIdentifierError>;
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
        DidCommEncryptedServiceGenerateError<R::FindIdentifierError, R::GenerateError>;
    type VerifyError = DidCommEncryptedServiceVerifyError<R::FindIdentifierError>;
    async fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &KeyPairing,
        to_did: &str,
        metadata: Option<&Value>,
    ) -> Result<DidCommMessage, Self::GenerateError> {
        generate::<R, R>(
            &self.vc_service,
            &self.vc_service,
            model,
            from_keyring,
            to_did,
            metadata,
            Some(&self.attachment_link),
        )
        .await
    }
    async fn verify(
        &self,
        my_keyring: &KeyPairing,
        message: &DidCommMessage,
    ) -> Result<VerifiedContainer, Self::VerifyError> {
        verify(&self.vc_service, my_keyring, message).await
    }
}

#[derive(Debug, Error)]
pub enum DidCommEncryptedServiceVerifyError<FindIdentifierError: std::error::Error> {
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("something went wrong with vc service: {0}")]
    VcService(#[from] CredentialSignerVerifyError),
    #[error("failed to find identifier: {0}")]
    SidetreeFindRequestFailed(FindIdentifierError),
    #[error("did public key not found. did: {0}")]
    DidPublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to decrypt message: {0:?}")]
    DecryptFailed(#[from] didcomm_rs::Error),
    #[error("failed to get body: {0:?}")]
    MetadataBodyNotFound(Option<didcomm_rs::Error>),
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to find sender did: {0}")]
    FindSender(#[from] FindSenderError),
}
