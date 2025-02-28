use std::convert::TryInto;

use data_encoding::BASE64URL_NOPAD;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct JwsHeader {
    alg: String,
    b64: bool,
    crit: Vec<String>,
}

#[derive(Debug, Error)]
pub enum JwsEncodeError {
    #[error("PublicKeyConvertError : {0:?}")]
    SignatureError(#[from] k256::ecdsa::Error),
    #[error("CanonicalizeError : {0:?}")]
    CanonicalizeError(#[from] serde_json::Error),
}

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

pub fn sign(object: &Value, secret_key: &k256::SecretKey) -> Result<String, JwsEncodeError> {
    let header = JwsHeader {
        alg: "ES256K".to_string(),
        b64: false,
        crit: vec!["b64".to_string()],
    };
    let header = serde_jcs::to_string(&header)?;
    let header = BASE64URL_NOPAD.encode(header.as_bytes());

    let payload = BASE64URL_NOPAD.encode(object.to_string().as_bytes());

    let message = [header.clone(), payload].join(".");
    let message: &[u8] = message.as_bytes();

    let signing_key: SigningKey = secret_key.into();
    let signature: Signature = signing_key.try_sign(message)?;
    let signature = BASE64URL_NOPAD.encode(&signature.to_vec());

    Ok([header, "".to_string(), signature].join("."))
}

pub fn verify(
    object: &Value,
    jws: &str,
    public_key: &k256::PublicKey,
) -> Result<(), JwsDecodeError> {
    let split: Vec<String> = jws.split('.').map(|v| v.to_string()).collect();

    if split.len() != 3 {
        return Err(JwsDecodeError::InvalidJws(jws.to_string()));
    }

    let _header = split[0].clone();
    let __payload = split[1].clone();
    let _signature = split[2].clone();

    let decoded = BASE64URL_NOPAD.decode(_header.as_bytes())?;
    let decoded = String::from_utf8(decoded)?;
    let header = serde_json::from_str::<JwsHeader>(&decoded)?;

    if header.alg != *"ES256K" {
        return Err(JwsDecodeError::InvalidAlgorithm(header.alg));
    }
    if header.b64 {
        return Err(JwsDecodeError::B64NotSupported);
    }
    if header.crit.iter().all(|v| v != "b64") {
        return Err(JwsDecodeError::B64NotSupportedButContained);
    }

    if __payload != *"".to_string() {
        return Err(JwsDecodeError::EmptyPayload);
    }
    let _payload = BASE64URL_NOPAD.encode(object.to_string().as_bytes());

    let message = [_header, _payload].join(".");

    let signature = BASE64URL_NOPAD.decode(_signature.as_bytes())?;
    if signature.len() != 64 {
        return Err(JwsDecodeError::InvalidSignatureLength(signature.len()));
    }
    let r: &[u8; 32] = &signature[0..32].try_into().unwrap();
    let s: &[u8; 32] = &signature[32..].try_into().unwrap();
    let wrapped_signature = Signature::from_scalars(*r, *s)?;

    let verify_key = VerifyingKey::from(public_key);
    Ok(verify_key.verify(message.as_bytes(), &wrapped_signature)?)
}
