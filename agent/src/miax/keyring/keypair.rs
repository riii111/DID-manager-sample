use protocol::keyring::keypair::K256KeyPair;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum KeyPairingError {
    #[error("create keyring failed: {0}")]
    CreateKeyringFailed(#[from] protocol::keyring::keypair::KeyPairingError),
    #[error("key not found")]
    KeyNotFound,
    #[error("DID not found")]
    DIDNotFound,
}

impl<S: SecureKeyStore> KeyPairWithConfig<S> {
    pub fn load_keyring(
        config: Box<SingletonAppConfig>,
        secure_keystore: S,
    ) -> Result<Self, KeyPairingError> {
        let sign = secure_keystore
            .read_update()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let update = secure_keystore
            .read_update()
            .ok_or(KeyPairingError::KeyNotFound)?;

        let recovery = secure_keystore
            .read_recovery()
            .ok_or(KeyPairingError::KeyNotFound)?;
        // let encrypt

        Ok(KeyPairWithConfig {
            sign,
            update,
            recovery,
            // encrypt,
            config,
            secure_keystore,
        })
    }

    pub fn get_identifier(&self) -> Result<String, KeyPairingError> {
        self.config
            .lock()
            .get_did()
            .ok_or(KeyPairingError::DIDNotFound)
    }
}
