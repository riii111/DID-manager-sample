use crate::did::did_repository::GetPublicKeyError;
use crate::keyring::keypair;
use crate::verifiable_credentials::credential_signer::CredentialSignerVerifyError;
use crate::verifiable_credentials::types::VerifiableCredentials;
use thiserror::Error;

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
