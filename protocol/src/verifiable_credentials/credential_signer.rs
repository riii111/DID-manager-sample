use serde_json::json;
use thiserror::Error;

use super::types::Proof;
use crate::{
    keyring::keypair::{K256KeyPair, KeyPair},
    verifiable_credentials::{jws, types::VerifiableCredentials},
};

pub struct CredentialSignerSuite<'a> {
    pub did: &'a str,
    pub key_id: &'a str,
    pub context: &'a K256KeyPair,
}

#[derive(Debug, Error)]
pub enum CredentialSignerSignError {
    #[error("jws error: {0:?}")]
    Jws(#[from] jws::JwsEncodeError),
    #[error("json parse error: {0:?}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum CredentialSignerVerifyError {
    #[error("jws error: {0:?}")]
    Jws(#[from] jws::JwsDecodeError),
    #[error("jws parse error: {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("proof not found")]
    ProofNotFound,
}

pub struct CredentialSigner {}

impl CredentialSigner {
    pub fn sign(
        mut object: VerifiableCredentials,
        suite: CredentialSignerSuite,
    ) -> Result<VerifiableCredentials, CredentialSignerSignError> {
        let jws = jws::sign(&json!(object), &suite.context.get_secret_key())?;
        let did = suite.did;
        let key_id = suite.key_id;
        object.proof = Some(Proof {
            r#type: "EcdsaSecp256k1Signature2019".to_string(),
            proof_purpose: "authentication".to_string(),
            // Assume that object.issuance_date is correct data
            created: object.issuance_date,
            verification_method: format!("{}#{}", did, key_id),
            jws,
            domain: None,
            controller: None,
            challenge: None,
        });
        Ok(object)
    }

    pub fn verify(
        mut object: VerifiableCredentials,
        public_key: &k256::PublicKey,
    ) -> Result<VerifiableCredentials, CredentialSignerVerifyError> {
        let proof = object
            .proof
            .take()
            .ok_or(CredentialSignerVerifyError::ProofNotFound)?;
        let jws = proof.jws;
        let payload = serde_json::to_value(&object)?;
        jws::verify(&payload, &jws, public_key)?;
        Ok(object)
    }
}
