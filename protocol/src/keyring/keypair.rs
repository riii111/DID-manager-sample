use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyPairHex {
    // MEMO: Matching schema in MiaX config.
    public_key: String,
    secret_key: String,
}
