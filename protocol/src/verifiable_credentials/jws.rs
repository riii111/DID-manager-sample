use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwsDecodeError {
    #[error("DecodeError: {0:?}")]
    DecodeError(#[from] data_encoding::DecodeError),
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    #[error("invalid signature length: {0}")]
    InvalidSignatureLength(usize),
    #[error("InvalidAlgorithm: {0}")]
    InvalidAlgorithm(String),
    #[error("b64 option is not supported")]
    B64NotSupported,
    #[error("b64 option is not supported, but contained")]
    B64NotSupportedButContained,
    #[error("EmptyPayload")]
    EmptyPayload,
    #[error("InvalidJws : {0}")]
    InvalidJws(String),
    #[error("CryptError: {0:?}")]
    CryptError(#[from] k256::ecdsa::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}
