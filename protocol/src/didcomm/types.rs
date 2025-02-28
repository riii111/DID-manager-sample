use data_encoding::BASE64URL_NOPAD;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DidCommMessage {
    pub ciphertext: String,
    pub iv: String,
    pub protected: String,
    pub recipients: Vec<Recipient>,
    pub tag: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Recipient {
    pub encrypted_key: String,
    pub header: Header,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Header {
    pub alg: String,
    pub epk: Epk,
    pub iv: String,
    pub key_ops: Vec<String>,
    pub kid: String,
    pub tag: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Epk {
    pub crv: String,
    pub kty: String,
    pub x: String,
}

#[derive(Debug, Error)]
pub enum FindSenderError {
    #[error("failed serialize/deserialize: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to base64 decode protected: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("failed to base64 decode protected: {0}")]
    Decode(#[from] data_encoding::DecodeError),
    #[error("skid error")]
    Skid,
}

impl DidCommMessage {
    pub fn find_sender(&self) -> Result<String, FindSenderError> {
        let protected = &self.protected;

        let decoded = BASE64URL_NOPAD.decode(protected.as_bytes())?;
        let decoded = String::from_utf8(decoded)?;
        let decoded = serde_json::from_str::<serde_json::Value>(&decoded)?;

        let from_did = decoded
            .get("skid")
            .ok_or(FindSenderError::Skid)?
            .as_str()
            .ok_or(FindSenderError::Skid)?
            .to_string();

        Ok(from_did)
    }
}
