use crate::config::SingletonAppConfig;

#[derive(Clone)]
pub struct FileBaseKeyStore {
    config: Box<SingletonAppConfig>,
}

impl FileBaseKeyStore {
    pub fn new(config: Box<SingletonAppConfig>) -> Self {
        FileBaseKeyStore { config }
    }
}
