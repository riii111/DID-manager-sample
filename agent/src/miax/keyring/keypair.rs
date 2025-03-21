use crate::miax::extension::secure_keystore::SecureKeyStoreKey;
use crate::{config::SingletonAppConfig, miax::extension::secure_keystore::SecureKeyStore};
use protocol::keyring::keypair::{K256KeyPair, X25519KeyPair};
use protocol::rand_core::OsRng;
use thiserror::Error;

/// 設定とセキュア鍵ストアを統合した鍵ペア管理構造体
/// 鍵ペアのロード、生成、永続化、DID識別子の管理などを担当
pub struct KeyPairingWithConfig<S: SecureKeyStore> {
    sign: K256KeyPair,
    update: K256KeyPair,
    recovery: K256KeyPair,
    encrypt: X25519KeyPair,
    config: Box<SingletonAppConfig>,
    secure_keystore: S,
}

/// 鍵ペアリング処理に関連するエラー型
#[derive(Error, Debug)]
pub enum KeyPairingError {
    #[error("create keyring failed: {0}")]
    CreateKeyringFailed(#[from] protocol::keyring::keypair::KeyPairingError),
    #[error("key not found")]
    KeyNotFound,
    #[error("DID not found")]
    DIDNotFound,
}

impl<S: SecureKeyStore> KeyPairingWithConfig<S> {
    pub fn load_keyring(
        config: Box<SingletonAppConfig>,
        secure_keystore: S,
    ) -> Result<Self, KeyPairingError> {
        let sign = secure_keystore
            .read_sign()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let update = secure_keystore
            .read_update()
            .ok_or(KeyPairingError::KeyNotFound)?;

        let recovery = secure_keystore
            .read_recovery()
            .ok_or(KeyPairingError::KeyNotFound)?;
        let encrypt = secure_keystore
            .read_encrypt()
            .ok_or(KeyPairingError::KeyNotFound)?;

        Ok(KeyPairingWithConfig {
            sign,
            update,
            recovery,
            encrypt,
            config,
            secure_keystore,
        })
    }

    pub fn create_keyring(config: Box<SingletonAppConfig>, secure_keystore: S) -> Self {
        let keyring = protocol::keyring::keypair::KeyPairing::create_keyring(OsRng);

        KeyPairingWithConfig {
            sign: keyring.sign,
            update: keyring.recovery,
            recovery: keyring.update,
            encrypt: keyring.encrypt,
            config,
            secure_keystore,
        }
    }

    pub fn get_keyring(&self) -> protocol::keyring::keypair::KeyPairing {
        protocol::keyring::keypair::KeyPairing {
            sign: self.sign.clone(),
            update: self.update.clone(),
            recovery: self.recovery.clone(),
            encrypt: self.encrypt.clone(),
        }
    }

    pub fn save(&mut self, did: &str) {
        self.secure_keystore
            .write(&SecureKeyStoreKey::Sign(&self.sign));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Update(&self.update));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Recovery(&self.recovery));
        self.secure_keystore
            .write(&SecureKeyStoreKey::Encrypt(&self.encrypt));
        {
            let mut config = self.config.lock();
            config.save_did(did);
            config.save_is_initialized(true);
        }
    }

    pub fn get_identifier(&self) -> Result<String, KeyPairingError> {
        self.config
            .lock()
            .get_did()
            .ok_or(KeyPairingError::DIDNotFound)
    }
}
