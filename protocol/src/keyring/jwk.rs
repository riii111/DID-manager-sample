// JWK: Json Web Key. RFC7517で定義
// https://tex2e.github.io/rfc-translater/html/rfc7517.html

use serde::{Deserialize, Serialize};

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
