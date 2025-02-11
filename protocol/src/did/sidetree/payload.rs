use crate::keyring::jwk::Jwk;
use data_encoding::BASE64_NOPAD;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::did::sidetree::multihash;

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
    /// 公開鍵をJWK形式に変換する
    /// このトレイトにより、異なる公開鍵形式（k256, x25519）を、DID DocumentやSidetreeペイロードで利用可能なJWK形式に変換可能となる
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
        key_type: String, // JWKの"kty"（キーの種類）をあらわす（例: "EC", "OKP"）
        key_id: String,   // JWKの"id"（キーのID）をあらわす（例: "kid"）
        purpose: Vec<String>, // JWKの"purpose"（用途）をあらわす（例: ["authentication"], ["assertionMethod"]）
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum DidAction {
    #[serde(rename = "replace")]
    Replace { document: DidPatchDocument },
    #[serde(rename = "add-public-keys")]
    AddPublicKeys {
        #[serde(rename = "public_keys")]
        public_keys: Vec<PublicKeyPayload>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct DidDeltaObject {
    patches: Vec<DidAction>,
    update_commitment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DidSuffixObject {
    delta_hash: String,
    recovery_commitment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MiaxDidResponse {
    pub did_document: DidDocument,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum DidPayload {
    #[serde(rename = "create")]
    Create { delta: String, suffix_data: String },
    #[serde(rename = "update")]
    Update {
        delta: String,
        #[serde(rename = "did_suffix")]
        did_suffix: String,
        #[serde(rename = "signed_data")]
        signed_data: String,
    },
}

#[derive(Debug, Error)]
pub enum DidCreatePayloadError {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to convert to JWK: {0}")]
    Jwk(#[from] crate::keyring::jwk::K256ToJwkError),
}

// JCSを利用してJSON値を正規化し、バイト列に変換
#[inline]
fn canon<T>(value: &T) -> Result<Vec<u8>, serde_json::Error>
where
    T: ?Sized + Serialize,
{
    Ok(serde_jcs::to_string(value)?.into_bytes())
}

/// 公開鍵からコミットメントを生成（二重ハッシュによるセキュリティ強化）
///
/// コミットメントは、公開鍵をハッシュ化した値であり、DIDドキュメントの更新やリカバリの際に、
/// 公開鍵の正当性を検証するために利用される
///
/// # 処理の流れ
/// 1. 公開鍵を JWK (JSON Web Key) 形式に変換
/// 2. JWK を JCS (JSON Canonicalization Scheme) で正規化 (JSONの形式を標準化)
/// 3. 正規化された JWK をバイト列に変換
/// 4. バイト列を SHA-256 でハッシュ化 (1回目)
/// 5. ハッシュ値をさらに SHA-256 でハッシュ化 (2回目、二重ハッシュ)
/// 6. 二重ハッシュ値を Multihash 形式でエンコード (Base58Btc エンコード)
/// 7. Multihash 形式のハッシュ値を Base64URL エンコードして返す
///
/// # 参考文献
/// - [Sidetree Specification - Hashing Process](https://identity.foundation/sidetree/spec/#hashing-process)
#[inline]
fn commitment_scheme(value: &Jwk) -> Result<String, serde_json::Error> {
    Ok(multihash::double_hash_encode(&canon(value)?))
}

// 参考 : https://identity.foundation/sidetree/spec/
pub fn did_create_payload(
    replace_payload: DidPatchDocument, // 新しいDIDドキュメントの内容
    update_key: k256::PublicKey,       // 更新用の公開鍵
    recovery_key: k256::PublicKey,     // リカバリ用の公開鍵
) -> Result<String, DidCreatePayloadError> {
    // 更新・リカバリ用のコミットメント（公開鍵から生成されるハッシュ値）を生成
    let update_commitment = commitment_scheme(&update_key.try_into()?)?;
    let recovery_commitment = commitment_scheme(&recovery_key.try_into()?)?;

    // DIDドキュメントの更新操作patchを作成（Sidetreeプロトコルでは、DIDドキュメントの更新は"patch"と表現される）
    let patch = DidAction::Replace {
        document: replace_payload,
    };

    // 変更内容を作成してエンコード
    let delta = DidDeltaObject {
        patches: vec![patch],
        update_commitment,
    };
    let delta = canon(&delta)?;
    let delta_hash = multihash::hash_encode(&delta);

    // suffix（DID識別子の一部）を生成
    // suffixは、DID識別子（did:miax:suffix ）の一部として利用され、DIDのUniquenessを担保するために利用される
    let suffix = DidSuffixObject {
        delta_hash,
        recovery_commitment,
    };
    // JCSで正規化・バイト列に置換
    let suffix = canon(&suffix)?;

    // BASE64URLでエンコード
    let encoded_delta = BASE64_NOPAD.encode(&delta);
    let encoded_suffix = BASE64_NOPAD.encode(&suffix);

    // 最終的なペイロードを作成
    let payload = DidPayload::Create {
        delta: encoded_delta,
        suffix_data: encoded_suffix,
    };

    // Sidetreeプロトコルに準拠したJSON形式のペイロードを返す
    Ok(serde_jcs::to_string(&payload)?)
}
