use hex::FromHexError;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyPairHex {
    // MEMO: Matching schema in MiaX config.
    public_key: String,
    secret_key: String,
}

#[derive(Error, Debug)]
pub enum KeyPairingError {
    #[error("from hex error: {0}")]
    FromHex(#[from] FromHexError),
    #[error("crypt error: {0}")]
    Crypt(String),
}

#[derive(Clone)]
pub struct K256KeyPair {
    secret_key: k256::SecretKey,
    public_key: k256::PublicKey,
}

impl K256KeyPair {
    pub fn new(secret_key: k256::SecretKey) -> Self {
        let public_key = secret_key.public_key();
        K256KeyPair {
            public_key,
            secret_key,
        }
    }
}

pub trait KeyPair<S, P>: Sized {
    type Error: std::error::Error;
    fn get_secret_key(&self) -> S;
    fn get_public_key(&self) -> P;
    fn to_hex_key_pair(&self) -> KeyPairHex;
    fn from_hex_key_pair(kp: &KeyPairHex) -> Result<Self, Self::Error>;
}

impl KeyPair<k256::SecretKey, k256::PublicKey> for K256KeyPair {
    type Error = KeyPairingError;
    fn get_secret_key(&self) -> k256::SecretKey {
        self.secret_key.clone()
    }

    fn get_public_key(&self) -> k256::PublicKey {
        self.public_key
    }

    fn to_hex_key_pair(&self) -> KeyPairHex {
        let sk = self.secret_key.to_bytes();
        let secret_key = hex::encode(sk);
        let pk = self.public_key.to_encoded_point(false);
        let public_key = hex::encode(pk.as_bytes());
        KeyPairHex {
            secret_key,
            public_key,
        }
    }

    fn from_hex_key_pair(kp: &KeyPairHex) -> Result<Self, Self::Error> {
        let secret_key = hex::decode(&kp.secret_key)?;
        let secret_key = k256::SecretKey::from_slice(&secret_key)
            .map_err(|e| KeyPairingError::Crypt(e.to_string()))?;
        let public_key = hex::decode(&kp.public_key)?;
        let public_key = k256::PublicKey::from_sec1_bytes(&public_key)
            .map_err(|e| KeyPairingError::Crypt(e.to_string()))?;
        Ok(K256KeyPair {
            public_key,
            secret_key,
        })
    }
}

#[derive(Clone)]
pub struct X25519KeyPair {
    secret_key: x25519_dalek::StaticSecret,
    public_key: x25519_dalek::PublicKey,
}

impl X25519KeyPair {
    pub fn new(secret_key: x25519_dalek::StaticSecret) -> Self {
        let public_key = x25519_dalek::PublicKey::from(&secret_key);
        X25519KeyPair {
            public_key,
            secret_key,
        }
    }
}

impl KeyPair<x25519_dalek::StaticSecret, x25519_dalek::PublicKey> for X25519KeyPair {
    type Error = KeyPairingError;
    fn get_secret_key(&self) -> x25519_dalek::StaticSecret {
        self.secret_key.clone()
    }
    fn get_public_key(&self) -> x25519_dalek::PublicKey {
        self.public_key
    }
    fn to_hex_key_pair(&self) -> KeyPairHex {
        let sk = self.secret_key.to_bytes();
        let secret_key = hex::encode(sk);
        let pk = self.public_key.as_bytes();
        let public_key = hex::encode(pk);
        KeyPairHex {
            secret_key,
            public_key,
        }
    }
    fn from_hex_key_pair(kp: &KeyPairHex) -> Result<Self, KeyPairingError> {
        let secret_key = hex::decode(&kp.secret_key)?;
        let secret_key: [u8; 32] = secret_key.try_into().map_err(|e: Vec<u8>| {
            KeyPairingError::Crypt(format!("array length mismatch: {}", e.len()))
        })?;
        let secret_key = x25519_dalek::StaticSecret::from(secret_key);
        let public_key = hex::decode(&kp.public_key)?;
        let public_key: [u8; 32] = public_key.try_into().map_err(|e: Vec<u8>| {
            KeyPairingError::Crypt(format!("array length mismatch: {}", e.len()))
        })?;
        let public_key = x25519_dalek::PublicKey::from(public_key);
        Ok(X25519KeyPair {
            public_key,
            secret_key,
        })
    }
}

#[derive(Clone)]
pub struct KeyPairing {
    pub sign: K256KeyPair,
    pub update: K256KeyPair,
    pub recovery: K256KeyPair,
    pub encrypt: X25519KeyPair,
}

impl KeyPairing {
    pub fn create_keyring<T: RngCore + CryptoRng>(mut csprng: T) -> Self {
        let sign = K256KeyPair::new(k256::SecretKey::random(&mut csprng));
        let update = K256KeyPair::new(k256::SecretKey::random(&mut csprng));
        let recovery = K256KeyPair::new(k256::SecretKey::random(&mut csprng));
        let encrypt = X25519KeyPair::new(x25519_dalek::StaticSecret::random_from_rng(&mut csprng));
        KeyPairing {
            sign,
            update,
            recovery,
            encrypt,
        }
    }
}
