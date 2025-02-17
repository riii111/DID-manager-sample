use crate::verifiable_credentials::jws;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialSignerVerifyError {
    #[error("jws error: {0:?}")]
    Jws(#[from] jws::JwsDecodeError),
    #[error("jws parse error: {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("proof not found")]
    ProofNotFound,
}
