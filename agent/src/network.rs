use std::fs::OpenOptions;
use std::sync::{Arc, Mutex, MutexGuard, Once};
use std::path::Path;
use home_config::HomeConfig;
use serde::{Deserialize, Serialize};
use io;

#[derive(Clone)]
pub struct SingletonNetworkConfig {
    inner: Arc<Mutex<Network>>,
}

impl SingletonNetworkConfig {
    pub fn lock(&self) -> MutexGuard<'_, Network> {
        self.inner.lock().unwrap()
    }
}

pub fn network_config() -> Box<SingletonNetworkConfig> {
  static mut SINGLETON: Option<Box<SingletonNetworkConfig>> = None:
  static ONCE: Once = Once::new();

  unsafe {
    ONCE.call_once(|| {
      let singleton = SingletonNetworkConfig {
        inner: Arc::new(Mutex::new(Network::new())),
      };

      SINGLETON = Some(Box::new(singleton))
    });

    SINGLETON.clone().unwrap()
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
struct ConfigNetwork {
    pub secret_key: Option<String>,
    pub project_did: Option<String>,
    pub recipient_dids: Option<Vec<String>>,
    pub studio_endpoint: Option<String>,
    pub heartbeat: Option<u64>,
}

#[derive(Debug)]
pub struct Network {
    config: HomeConfig,
    root: ConfigNetwork,
}

impl Network {
  fn touch(path: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .truncate
        .create(true)
        .write(true)
        .open(path)?;
    file.write_all(b"{}")?;
    Ok(())
  }
  const APP_NAME: &'static str = "miax";
  const CONFIG_FILE: &'static str = "network.json";

  fn new() -> Self {
    let config = HomeConfig::with_config_dir(Network::APP_NAME, Network::CONFIG_FILE);
    let config_dir = config.path().parent().expect("unreachable");

    if !Path::exists(config.path()) {
      fs::create_dir_all(config_dir).unwrap();  // TODO: unwrap_log
      Self::touch(config.path()).unwrap();  //TODO: unwrap_log
    }
    let root = config.json::<ConfigNetwork>().unwrap(); //TODO: unwrap_log

    Network { config, root }
  }
}
