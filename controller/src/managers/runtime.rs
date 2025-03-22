use semver::Version;
use std::path::PathBuf;
use tokio::sync::watch;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Idle,
    Update,
    Rollback,
}

pub enum MiaxSignal {
    Terminate,
    SendFd,
}

// ": Clone"... 「このtraitを実装する全ての型は、Cloneトレイトも実装必要」
pub trait ProcessManager: Clone {
    fn is_running(&self, process_id: u32) -> bool;
    fn spawn_process(&self, cmd: impl AsRef<Path>, args: &[&str]) -> Result<u32, std::io::Error>;
    fn kill_process(&self, process_id: u32, signal: MiaxSignal) -> Result<(), std::io::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Failed to open file: {0}")]
    FileOpen(#[source] std::io::Error),
    #[error("Failed to read file: {0}")]
    FileRead(#[source] std::io::Error),
    #[error("Failed to write data to file: {0}")]
    FileWrite(#[source] std::io::Error),
    #[error("Failed to remove file: {0}")]
    FileRemove(#[source] std::io::Error),
    #[error("Failed to acquire exclusive file lock: {0}")]
    FileLock(#[source] std::io::Error),
    #[error("Failed to unlock file: {0}")]
    FileUnlock(#[source] std::io::Error),
    #[error("Failed to serialize runtime info to JSON: {0}")]
    JsonSerialize(#[source] serde_json::Error),
    #[error("Failed to deserialize runtime info from JSON: {0}")]
    JsonDeserialize(#[source] serde_json::Error),
    #[error("Failed to kill process")]
    Kill(std::io::Error),
    #[error("Failed to kill processes")]
    Kills(Vec<RuntimeError>),
    #[error("Failed to create command: {0}")]
    Command(#[source] std::io::Error),
    #[error("Failed to fork: {0}")]
    Fork(#[source] std::io::Error),
    #[error("failed to know path of self exe: {0}")]
    FailedCurrentExe(#[source] std::io::Error),
    #[error("Controller already running")]
    AlreadyExistController,
    #[error(transparent)]
    SemVer(#[from] semver::Error),
    #[cfg(unix)]
    #[error("Failed to bind UDS: {0}")]
    BindUdsError(#[source] std::io::Error),
    #[cfg(unix)]
    #[error("Failed to watch UDS: {0}")]
    WatchUdsError(#[source] notify::Error),
    #[cfg(unix)]
    #[error("Failed to get fd from systemd: {0}")]
    GetFd(#[from] crate::unix_utils::GetFdError),
    #[cfg(unix)]
    #[error("Request failed: {0}")]
    Request(#[from] crate::unix_utils::GetRequestError),
    #[cfg(unix)]
    #[error("Failed to get meta uds path")]
    PathConvention,
}

pub trait RuntimeInfoStorage: std::fmt::Debug {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError>;
    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>;
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    pub version: String,
}

#[trait_variant::make(Send)]
pub trait RuntimeManager: RuntimeManagerWithoutAsync {
    async fn get_version(&self) -> Result<Version, RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    self_pid: u32,
    file_handler: H,
    process_manager: P,
    uds_path: PathBuf,
    meta_uds_path: PathBuf,
    state_sender: watch::Sender<State>,
}
impl<H, P> RuntimeManager for RuntimeManagerImpl<H, P>
where 
    H: RuntimeInfoStorage + Sync + Send,
    P: ProcessManager + Sync + Send,
{
    async fn get_version(&self) -> Result<Version, RuntimeError> {
        #[cfg(unix)]
        let version_response: VersionResponse =
            crate::unix_utils::get_request(&self.uds_path, "/internal/version/get").await?;
        #[cfg(windows)]
        let version_response = VersionResponse {
            version: "9.9.9".to_string(),
        };
        Ok(Version::parse(&version_response.version)?)
    }
}