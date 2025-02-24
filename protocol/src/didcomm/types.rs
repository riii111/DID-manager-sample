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
