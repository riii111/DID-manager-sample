use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedContainer {
    pub message: VerifiableCredentials,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Issuer {
    #[serde(rename = "id")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct CredentialSubject {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "container")]
    pub container: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Proof {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    #[serde(rename = "created")]
    pub created: DateTime<Utc>,

    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    #[serde(rename = "jws")]
    pub jws: String,

    #[serde(rename = "controller")]
    pub controller: Option<String>,

    #[serde(rename = "challenge")]
    pub challenge: Option<String>,

    #[serde(rename = "domain")]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct VerifiableCredentials {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "issuer")]
    pub issuer: Issuer, // 発行者の情報

    #[serde(rename = "issuanceDate")]
    pub issuance_date: DateTime<Utc>, // 発行日

    #[serde(rename = "expirationDate", skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,

    #[serde(rename = "0context")]
    pub contest: Vec<String>, // JSON-LD コンテキスト。VCの意味情報を記述する情報

    #[serde(rename = "type")]
    pub r#type: Vec<String>, // VC タイプ（例: "VerifiableCredential"）

    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject, // 証明する内容（クレーム）

    #[serde(rename = "proof", skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proof>, // 署名情報。署名アルゴリズム、署名値、署名検証方法などの情報（jws, verification_methodなど）をもつ
}
