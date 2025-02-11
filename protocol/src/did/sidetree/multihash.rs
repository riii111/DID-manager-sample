use data_encoding::BASE64URL_NOPAD;
use sha2::{Digest, Sha256};
use std::convert::TryInto;

const MULTIHASH_SHA256_CODE: u8 = 0x12;

// [NOTE]: SHA2-256 only
// SHA-256ハッシュを計算し、マルチハッシュフォーマットでエンコード
pub fn hash(message: &[u8]) -> Vec<u8> {
    // ハッシュ値の先頭にハッシュ関数の種類・ハッシュ長の情報(MULTIHASH_SHA256_CODE)を付与することで、今後異なるハッシュ関数への移行も可能となる
    let mut prefix = Vec::from([MULTIHASH_SHA256_CODE]);
    let mut hashed = Sha256::digest(message).to_vec();
    prefix.push(hashed.len().try_into().unwrap());
    prefix.append(&mut hashed);
    prefix
}

// メッセージを二重ハッシュしてBASE64URLエンコード
pub fn double_hash_encode(message: &[u8]) -> String {
    let mes = Sha256::digest(message).to_vec();
    let mes = hash(&mes);
    BASE64URL_NOPAD.encode(&mes)
}

pub fn hash_encode(message: &[u8]) -> String {
    let mes = hash(message);
    BASE64URL_NOPAD.encode(&mes)
}
