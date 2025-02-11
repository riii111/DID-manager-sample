use home_config::HomeConfig;
use protocol::keyring::keypair::{
    K256KeyPair, KeyPair, KeyPairHex, KeyPairingError, X25519KeyPair,
};
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
    /// DIDComm HTTP Body Size Limit
    pub http_body_size_limit: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct KeyPairsConfig {
    /// 署名鍵ペア
    sign: Option<KeyPairHex>,
    /// 更新鍵ペア
    update: Option<KeyPairHex>,
    /// リカバリ鍵ペア
    recovery: Option<KeyPairHex>,
    /// 暗号化鍵ペア
    encrypt: Option<KeyPairHex>,
}

#[derive(Deserialize, Serialize)]
pub struct ConfigRoot {
    /// DID
    did: Option<String>,
    /// 鍵ペア設定
    key_pairs: KeyPairsConfig,
    // extensions:
    // metrics
    /// DIDComm設定
    didcomm: DidCommConfig,
    /// 初期化済みフラグ
    is_initialized: bool,
    /// 設定スキーマバージョン
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
                sign: None,
                update: None,
                recovery: None,
                encrypt: None,
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

/// アプリケーション全体の設定を管理する構造体。
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

/// KeyPairHex設定からKeyPair型へ変換する関数
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
    /// 設定ファイルが存在しない場合に作成する
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

    pub fn save_recovery_key_pair(&mut self, value: &K256KeyPair) {
        self.root.key_pairs.recovery = Some(value.to_hex_key_pair());
        self.write().unwrap();
    }

    pub fn load_encrypt_key_pair(&self) -> Option<X25519KeyPair> {
        load_key_pair(&self.root.key_pairs.encrypt)
    }

    pub fn save_encrypt_key_pair(&mut self, value: &X25519KeyPair) {
        self.root.key_pairs.encrypt = Some(value.to_hex_key_pair());
        self.write().unwrap();
    }

    pub fn get_did(&self) -> Option<String> {
        self.root.did.clone()
    }

    pub fn save_did(&mut self, value: &str) {
        self.root.did = Some(value.to_string());
        self.write().unwrap() // TODO: unwrap_log
    }

    pub fn save_is_initialized(&mut self, value: bool) {
        self.root.is_initialized = value;
        self.write().unwrap() // TODO: unwrap_log
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

#[allow(static_mut_refs)]
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
    // did_attachment_link: String,
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
        // let link =
        // env::var("MIAX_DID_ATTACHMENT_LINK").unwrap_or("https://did.miacross.io".to_string());

        ServerConfig {
            did_http_endpoint: did_endpoint,
            // did_attachment_link: link,
        }
    }

    pub fn did_http_endpoint(&self) -> String {
        self.did_http_endpoint.clone()
    }
    // pub fn did_attachment_link(&self) -> String {
    // self.did_attachment_link.clone()
    // }
}

pub fn server_config() -> ServerConfig {
    ServerConfig::new()
}
