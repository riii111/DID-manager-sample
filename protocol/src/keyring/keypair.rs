use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyPairHex {
    // MEMO: Matching schema in MiaX config.
    // TODO: 一時的にpublic
    pub public_key: String,
    pub secret_key: String,
}
