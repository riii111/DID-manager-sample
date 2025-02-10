use crate::keyring::jwk::Jwk;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    // renameの意図はおそらく : W3CのDID仕様に準拠している。また、将来名前が変化しても維持可。そして、外部IFとしての整合性を保証（これがあることで、外部IFであることも理解できる）。
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,

    #[serde(rename = "description")]
    pub description: Option<String>,
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKeyPayload {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "jwk")]
    pub jwk: Jwk,

    #[serde(rename = "purpose")]
    pub purpose: Vec<String>,
}

pub trait ToPublicKey<T: TryInto<Jwk>> {
    fn to_public_key(
        self,
        key_type: String,
        key_id: String,
        purpose: Vec<String>,
    ) -> Result<PublicKeyPayload, T::Error>;
}

// - impl<T> ... 構造体がジェネリクスTを使用しており、実装するメソッド内でもTを扱う場合はこの宣言が必要
// - TryFromを実装すると、TryIntoも自動で実装される（Rustの仕様）。その結果、本トレイトも利用可能
impl<T> ToPublicKey<T> for T
where
    T: TryInto<Jwk>,
{
    /// K256KeyPairやX25519KeyPairが持つ公開鍵を、DIDDocumentなどで利用されるJWK形式に変換する
    fn to_public_key(
        self,
        key_type: String,
        key_id: String,
        purpose: Vec<String>,
    ) -> Result<PublicKeyPayload, <T>::Error> {
        let jwk: Jwk = self.try_into()?;
        Ok(PublicKeyPayload {
            id: key_id,
            r#type: key_type,
            jwk,
            purpose,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DidPatchDocument {
    #[serde(rename = "public_keys")]
    pub public_keys: Vec<PublicKeyPayload>,

    #[serde(rename = "service_endpoints")]
    pub service_endpoints: Vec<ServiceEndpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiaxDidResponse {
    pub did_document: DidDocument,
}
