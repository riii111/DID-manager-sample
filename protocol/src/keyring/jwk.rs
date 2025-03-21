// JWK: Json Web Key. RFC7517で定義
// https://tex2e.github.io/rfc-translater/html/rfc7517.html
use std::convert::{From, Into, TryFrom, TryInto};

pub use data_encoding;
use data_encoding::{DecodeError, DecodePartial, BASE64URL_NOPAD, BASE64_NOPAD};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// DID DocumentやSidetreeペイロードで利用される形式
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Jwk {
    #[serde(rename = "kty")] // key type. example: "EC", "OKP", "RSA"...
    kty: String,

    #[serde(rename = "crv")] // curve. 使用する楕円曲線（ktyが"EC" or "OKP"の場合）
    crv: String,

    #[serde(rename = "x")] // 楕円曲線上のx座標(エンコード済)
    x: String,

    #[serde(rename = "y", skip_serializing_if = "Option::is_none")]
    // 楕円曲線上のy座標（kty="EC"かつ曲線次第では省略可）
    y: Option<String>,
}

#[derive(Error, Debug)]
pub enum JwkToK256Error {
    #[error("missing y")]
    MissingY,
    #[error("decode error")]
    Decode(DecodePartial),
    #[error("different crv")]
    DifferentCrv,
    #[error("crypt error: {0}")]
    Crypt(#[from] k256::elliptic_curve::Error),
}

#[derive(Error, Debug)]
pub enum JwkToX25519Error {
    #[error("decode error: {0:?}")]
    Decode(Option<DecodeError>),
    #[error("different crv")]
    DifferentCrv,
}

#[derive(Error, Debug)]
pub enum K256ToJwkError {
    #[error("points are invalid")]
    PointsInvalid,
}

fn decode_base64url(
    s: &str,
) -> Result<k256::elliptic_curve::FieldBytes<k256::Secp256k1>, JwkToK256Error> {
    let mut result = k256::elliptic_curve::FieldBytes::<k256::Secp256k1>::default();
    BASE64_NOPAD
        .decode_mut(s.as_bytes(), &mut result)
        .map_err(JwkToK256Error::Decode)?;
    Ok(result)
}

/// JWK から k256::PublicKey への変換処理
/// TryFromを実装することで、自動的にTryIntoも利用可能となる（Rustの仕様）
impl TryFrom<Jwk> for k256::PublicKey {
    type Error = JwkToK256Error;
    fn try_from(value: Jwk) -> Result<Self, Self::Error> {
        if value.crv != "secp256k1" {
            return Err(JwkToK256Error::DifferentCrv);
        }
        if let Some(y) = value.y {
            // Base64URLエンコードされた座標をエンコード
            let x = decode_base64url(&value.x)?;
            let y = decode_base64url(&y)?;

            // 座標から公開鍵を構築
            let pk = k256::EncodedPoint::from_affine_coordinates(&x, &y, false);
            let pk = k256::PublicKey::from_sec1_bytes(pk.as_bytes())?;
            Ok(pk)
        } else {
            Err(JwkToK256Error::MissingY)
        }
    }
}
/// k256::PublicKey から JWK への変換処理
impl TryFrom<k256::PublicKey> for Jwk {
    type Error = K256ToJwkError;
    fn try_from(value: k256::PublicKey) -> Result<Self, Self::Error> {
        let value = value.to_encoded_point(false);
        let kty = "EC".to_string();
        let crv = "secp256k1".to_string();
        match value.coordinates() {
            k256::elliptic_curve::sec1::Coordinates::Uncompressed { x, y } => {
                let x = BASE64URL_NOPAD.encode(x);
                let y = Some(BASE64URL_NOPAD.encode(y));
                Ok(Jwk { kty, crv, x, y })
            }
            _ => Err(K256ToJwkError::PointsInvalid),
        }
    }
}

impl TryFrom<Jwk> for x25519_dalek::PublicKey {
    type Error = JwkToX25519Error;
    fn try_from(value: Jwk) -> Result<Self, Self::Error> {
        if value.crv != "X25519" {
            return Err(JwkToX25519Error::DifferentCrv);
        }
        let pk = BASE64URL_NOPAD
            .decode(value.x.as_bytes())
            .map_err(|e| JwkToX25519Error::Decode(Some(e)))?;
        let pk: [u8; 32] = pk.try_into().map_err(|_| JwkToX25519Error::Decode(None))?;
        Ok(pk.into())
    }
}

impl From<x25519_dalek::PublicKey> for Jwk {
    fn from(value: x25519_dalek::PublicKey) -> Self {
        let x = BASE64URL_NOPAD.encode(value.as_bytes());
        let kty = "OKP".to_string();
        let crv = "X25519".to_string();
        Jwk {
            kty,
            crv,
            x,
            y: None,
        }
    }
}

// TODO: テストコード
