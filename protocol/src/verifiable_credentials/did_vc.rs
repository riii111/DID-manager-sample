use crate::did::did_repository::{get_sign_key, DidRepository, GetPublicKeyError};
use crate::keyring::keypair;
use crate::verifiable_credentials::credential_signer::CredentialSignerVerifyError;
use crate::verifiable_credentials::types::VerifiableCredentials;
use thiserror::Error;

use super::credential_signer::{
    CredentialSigner, CredentialSignerSignError, CredentialSignerSuite,
};

#[trait_variant::make(Send)]
pub trait DidVcService: Sync {
    type GenerateError: std::error::Error + Send + Sync;
    type VerifyError: std::error::Error + Send + Sync;
    fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &keypair::KeyPairing,
    ) -> Result<VerifiableCredentials, Self::GenerateError>;
    async fn verify(
        &self,
        model: VerifiableCredentials,
    ) -> Result<VerifiableCredentials, Self::VerifyError>;
}

#[derive(Debug, Error)]
pub enum DidVcServiceVerifyError<FindIdentifierError: std::error::Error> {
    #[error("did public key not found. did: {0}")]
    PublicKeyNotFound(#[from] GetPublicKeyError),
    #[error("failed to get did document: {0}")]
    DidDocNotFound(String),
    #[error("failed to find identifier: {0}")]
    FindIdentifier(FindIdentifierError),
    #[error("credential signer error")]
    VerifyFailed(#[from] CredentialSignerVerifyError),
}

impl<R: DidRepository> DidVcService for R {
    type GenerateError = CredentialSignerSignError;
    type VerifyError = DidVcServiceVerifyError<R::FindIdentifierError>;
    fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &keypair::KeyPairing,
    ) -> Result<VerifiableCredentials, Self::GenerateError> {
        let did = &model.issuer.id.clone();
        CredentialSigner::sign(
            model,
            CredentialSignerSuite {
                did,
                key_id: "signingkey",
                context: &from_keyring.sign,
            },
        )
    }
    async fn verify(
        &self,
        model: VerifiableCredentials,
    ) -> Result<VerifiableCredentials, Self::VerifyError> {
        let did_document = self
            .find_identifier(&model.issuer.id)
            .await
            .map_err(Self::VerifyError::FindIdentifier)?;
        let did_document = did_document
            .ok_or(DidVcServiceVerifyError::DidDocNotFound(
                model.issuer.id.clone(),
            ))?
            .did_document;
        let public_key = get_sign_key(&did_document)?;
        Ok(CredentialSigner::verify(model, &public_key)?)
    }
}
