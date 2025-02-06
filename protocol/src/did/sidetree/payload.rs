use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKeyJwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
    pub y: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DidPublicKey {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "controller")]
    pub controller: String,

    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "publicKeyJwk")]
    pub public_key_jwk: PublicKeyJwk,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DidDocument {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "publicKey")]
    pub public_key: Option<Vec<DidPublicKey>>,

    // 今回は省略
    #[serde(rename = "authentication")]
    pub authentication: Option<Vec<String>>,
}
