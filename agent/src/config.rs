use protocol::keyring::keypair::KeyPairHex;
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Mutex, MutexGuard, Once},
};



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

/// 最低限の設定例
pub struct AppConfig {
    did: Option<String>,
    key_pairs: KeyPairsConfig,
    is_initialized: bool,
}

impl AppConfig {
  fn new() -> Self {
    self.did,
    self.key_pairs,
    self.is_initialized
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
