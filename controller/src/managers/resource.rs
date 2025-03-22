use std::path::PathBuf;

use crate::config::get_config;

#[cfg(unix)]
pub struct UnixResourceManager {
    tmp_path: PathBuf,
    agent_path: PathBuf,
}
