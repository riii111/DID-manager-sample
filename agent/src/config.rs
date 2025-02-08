use home_config::HomeConfig;
use protocol::keyring::keypair::{K256KeyPair, KeyPair, KeyPairHex, KeyPairingError};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex, MutexGuard, Once},
};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DidCommConfig {
    pub http_body_size_limit: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct KeyPairsConfig {
    sign: Option<KeyPairHex>,
    update: Option<KeyPairHex>,
    recovery: Option<KeyPairHex>,
    encrypt: Option<KeyPairHex>,
}

#[derive(Deserialize, Serialize)]
pub struct ConfigRoot {
    did: Option<String>,
    key_pairs: KeyPairsConfig,
    // extensions:
    // metrics
    didcomm: DidCommConfig,
    is_initialized: bool,
    schema_version: u8,
}

impl Default for ConfigRoot {
    fn default() -> Self {
        ConfigRoot {
            // 環境変数があればその値を、なければダミー値を設定
            did: std::env::var("MiaX_DID")
                .ok()
                .or(Some("did:example:dummy".to_string())),
            key_pairs: KeyPairsConfig {
                sign: Some(KeyPairHex {
                    public_key: std::env::var("MiaX_SIGN_PUBLIC")
                        .unwrap_or_else(|_| "dummy-sign-public".to_string()),
                    secret_key: std::env::var("MiaX_SIGN_SECRET")
                        .unwrap_or_else(|_| "dummy-sign-secret".to_string()),
                }),
                update: Some(KeyPairHex {
                    public_key: std::env::var("MiaX_UPDATE_PUBLIC")
                        .unwrap_or_else(|_| "dummy-update-public".to_string()),
                    secret_key: std::env::var("MiaX_UPDATE_SECRET")
                        .unwrap_or_else(|_| "dummy-update-secret".to_string()),
                }),
                recovery: Some(KeyPairHex {
                    public_key: std::env::var("MiaX_RECOVERY_PUBLIC")
                        .unwrap_or_else(|_| "dummy-recovery-public".to_string()),
                    secret_key: std::env::var("MiaX_RECOVERY_SECRET")
                        .unwrap_or_else(|_| "dummy-recovery-secret".to_string()),
                }),
                encrypt: Some(KeyPairHex {
                    public_key: std::env::var("MiaX_ENCRYPT_PUBLIC")
                        .unwrap_or_else(|_| "dummy-encrypt-public".to_string()),
                    secret_key: std::env::var("MiaX_ENCRYPT_SECRET")
                        .unwrap_or_else(|_| "dummy-encrypt-secret".to_string()),
                }),
            },
            didcomm: DidCommConfig {
                http_body_size_limit: std::env::var("MiaX_DIDCOMM_HTTP_BODY_SIZE_LIMIT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3 * 1024 * 1024),
            },
            is_initialized: false,
            schema_version: 1,
        }
    }
}

/// 最低限の設定例
pub struct AppConfig {
    root: ConfigRoot,
    config: HomeConfig,
}

#[derive(Error, Debug)]
pub enum AppConfigError<E: std::error::Error> {
    #[error("key decode failed")]
    DecoreFailed(E),
    #[error("failed to write config file")]
    WriteError(home_config::JsonError),
}

fn convert_to_key<U, V, T: KeyPair<U, V>>(
    config: &KeyPairHex,
) -> Result<T, AppConfigError<T::Error>> {
    T::from_hex_key_pair(config).map_err(AppConfigError::DecoreFailed)
}

#[inline]
fn load_key_pair<U, V, T: KeyPair<U, V>>(kind: &Option<KeyPairHex>) -> Option<T> {
    kind.as_ref()
        .and_then(|key| convert_to_key(key).map_err(|e| log::error!("{:?}", e)).ok())
}

impl AppConfig {
    fn touch(path: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(path)?;
        file.write_all(b"{}")?;
        Ok(())
    }

    const APP_NAME: &'static str = "miax";
    const CONFIG_FILE: &'static str = "config.json";

    pub fn new() -> Self {
        let config = HomeConfig::with_config_dir(AppConfig::APP_NAME, AppConfig::CONFIG_FILE);
        let config_dir = config.path().parent().unwrap();

        if !Path::exists(config.path()) {
            fs::create_dir_all(config_dir).unwrap(); // TODO: unwrap_logの適用
            Self::touch(config.path()).unwrap(); // TODO: unwrap_logの適用
        }

        let root = ConfigRoot::default();

        AppConfig { root, config }
    }

    pub fn write(&self) -> Result<(), AppConfigError<KeyPairingError>> {
        self.config
            .save_json(&self.root)
            .map_err(AppConfigError::WriteError)
    }

    pub fn load_sign_key_pair(&self) -> Option<K256KeyPair> {
        load_key_pair(&self.root.key_pairs.sign)
    }

    pub fn save_sign_key_pair(&mut self, value: &K256KeyPair) {
        self.root.key_pairs.sign = Some(value.to_hex_key_pair());
        self.write().unwrap();
    }

    pub fn load_update_key_pair(&self) -> Option<K256KeyPair> {
        load_key_pair(&self.root.key_pairs.update)
    }

    pub fn save_update_key_pair(&mut self, value: &K256KeyPair) {
        self.root.key_pairs.update = Some(value.to_hex_key_pair());
        self.write().unwrap();
    }

    pub fn load_recovery_key_pair(&self) -> Option<K256KeyPair> {
        load_key_pair(&self.root.key_pairs.recovery)
    }
}

#[derive(Clone)]
pub struct SingletonAppConfig {
    inner: Arc<Mutex<AppConfig>>,
}

impl SingletonAppConfig {
    // '_ : 現在のスコープが続く間だけ有効なガード
    pub fn lock(&self) -> MutexGuard<'_, AppConfig> {
        self.inner.lock().unwrap()
    }
}

pub fn app_config() -> Box<SingletonAppConfig> {
    static mut SINGLETON: Option<Box<SingletonAppConfig>> = None;
    static ONCE: Once = Once::new();

    unsafe {
        // 初期化時は競合を防ぐため、Onceで他スレッドを待機
        ONCE.call_once(|| {
            let singleton = SingletonAppConfig {
                inner: Arc::new(Mutex::new(AppConfig::new())),
            };

            SINGLETON = Some(Box::new(singleton))
        });

        SINGLETON.clone().unwrap()
    }
}

#[derive(Debug)]
pub struct ServerConfig {
    did_http_endpoint: String,
    did_attachment_link: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerConfig {
    pub fn new() -> ServerConfig {
        let did_endpoint =
            env::var("MIAX_DID_HTTP_ENDPOINT").unwrap_or("https://did.miacross.io".to_string());
        let link =
            env::var("MIAX_DID_ATTACHMENT_LINK").unwrap_or("https://did.miacross.io".to_string());

        ServerConfig {
            did_http_endpoint: did_endpoint,
            did_attachment_link: link,
        }
    }

    pub fn did_http_endpoint(&self) -> String {
        self.did_http_endpoint.clone()
    }
    pub fn did_attachment_link(&self) -> String {
        self.did_attachment_link.clone()
    }
}

pub fn server_config() -> ServerConfig {
    ServerConfig::new()
}
