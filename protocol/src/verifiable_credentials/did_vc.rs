use crate::keyring::keypair;
use crate::verifiable_credentials::types::VerifiableCredentials;

#[trait_variant::make(Send)]
pub trait DidVcService: Sync {
    type GenerateError: std::error::Error + Send + Sync;
    type VerifyError: std::error::Error + Send + Sync;
    fn generate(
        &self,
        model: VerifiableCredentials,
        from_keyring: &keypair::KeyPairing,
    ) -> Result<VerifiableCredentials, Self::GenerateError>;
}
