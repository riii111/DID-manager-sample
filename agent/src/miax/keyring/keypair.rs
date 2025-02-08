use axum::http::Request;
use protocol::keyring::keypair::{K256KeyPair, KeyPairingError};

use crate::{
    config::SingletonAppConfig,
    miax::extension::secure_keystore::{self, SecureKeyStore},
};

pub struct KeyPairWithConfig<S: SecureKeyStore> {
    sign: K256KeyPair,
    update: K256KeyPair,
    recovery: K256KeyPair,
    // encrypt:
    config: Box<SingletonAppConfig>,
    secure_keystore: S,
}

impl<S: SecureKeyStore> KeyPairWithConfig<S> {
    pub fn load_keyring(
        config: Box<SingletonAppConfig>,
        secure_keystore: S,
    ) -> Result<Self, KeyPairingError> {
        unimplemented!("load keyring");
    }

    pub fn get_identifier(&self) -> Result<String, KeyPairingError> {
        unimplemented!("get did")
    }
}
