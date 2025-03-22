use crate::validator::process::{is_manage_by_systemd, is_manage_socket_activation};
use chrono::{DateTime, FixedOffset, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    os::unix::process,
    path::{Path, PathBuf},
};
use tokio::sync::watch;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RuntimeInfo {
    pub state: State,
    pub process_infos: [Option<ProcessInfo>; 4],
    pub exec_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Idle,
    Update,
    Rollback,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub executed_at: DateTime<FixedOffset>,
    pub version: Version,
    pub feat_type: FeatType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum FeatType {
    Agent,
    Controller,
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

pub trait RuntimeManagerWithoutAsync {
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError>;

    fn launch_controller(
        &mut self,
        new_controller_path: impl AsRef<Path>,
    ) -> Result<(), RuntimeError>;

    fn get_runtime_info(&mut self) -> Result<RuntimeInfo, RuntimeError>;

    fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError>;

    fn update_state(&mut self, state: State) -> Result<(), RuntimeError>;

    fn kill_process(&mut self, process_info: &ProcessInfo) -> Result<(), RuntimeError>;

    fn kill_other_agents(&mut self, target: u32) -> Result<(), RuntimeError>;
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

impl<H, P> RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.add_process_info(process_info))
    }

    fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        self.file_handler
            .apply_with_lock(|runtime_info| runtime_info.remove_process_info(process_id))
    }

    pub fn cleanup_all(&mut self) -> Result<(), RuntimeError> {
        #[cfg(unix)]
        {
            crate::unix_utils::remove_file_if_exists(&self.uds_path);
            crate::unix_utils::remove_file_if_exists(&self.meta_uds_path);
        }
        let process_manager = &self.process_manager;
        self.file_handler.apply_with_lock(move |runtime_info| {
            let mut errs = vec![];
            for info in runtime_info.process_infos.iter_mut() {
                if let Some(info) = info {
                    if let Err(err) =
                        process_manager.kill_process(info.process_id, MiaxSignal::Terminate)
                    {
                        errs.push(RuntimeError::Kill(err));
                    }
                }
                *info = None;
            }
            runtime_info.state = State::Idle;
            if !errs.is_empty() {
                return Err(RuntimeError::Kills(errs));
            } else {
                Err(RuntimeError::Kills(errs))
            }
        })
    }

    pub fn cleanup(&mut self) -> Result<(), RuntimeError> {
        self.remove_process_info(self.self_pid)
    }
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

impl<H, P> RuntimeManagerWithoutAsync for RuntimeManagerImpl<H, P>
where
    H: RuntimeInfoStorage,
    P: ProcessManager,
{
    fn launch_agent(&mut self, is_first: bool) -> Result<ProcessInfo, RuntimeError> {
        #[cfg(unix)]
        if is_first {
            if !is_manage_socket_activation() && self.uds_path.exists() {
                log::warn!("UDS file already exists, removing: {:?}", self.uds_path);
                std::fs::remove_file(&self.uds_path);
            }
            if self.meta_uds_path.exists() {
                log::warn!(
                    "UDS file already exists, removing: {:?}",
                    self.meta_uds_path
                );
                std::fs::remove_file(&self.meta_uds_path);
            }
        }
        let current_exe = &self.get_runtime_info()?.exec_path;
        let child = self
            .process_manager
            .spawn_process(current_exe, &["controlled"])
            .map_err(|e| RuntimeError::Fork)?;

        #[cfg(unix)]
        if is_first {
            let listener = if is_manage_by_systemd() && is_manage_socket_activation() {
                Some(crate::unix_utils::get_fd_from_systemd()?)
            } else {
                None
            };
            let () = crate::unix_utils::wait_until_file_created(&self.meta_uds_path)
                .map_err(RuntimeError::WatchUdsError)?;
            let stream = loop {
                match std::os::unix::net::UnixStream::connect(&self.meta_uds_path) {
                    Ok(stream) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::ConnectionRefused => {
                        // wait for bind
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        continue;
                    }
                    Err(err) => return Err(RuntimeError::BindUdsError(err)),
                }
            };
            let stream = std::os::unix::io::AsRawFd::as_raw_fd(&stream);
            crate::unix_utils::send_fd(stream, listener)
                .map_err(|e| RuntimeError::BindUdsError(e.into()))?;
        }
        let process_info = ProcessInfo::new(child, FeatType::Agent);
        self.add_process_info(process_info.clone())?;
        Ok(process_info)
    }
}

impl ProcessInfo {
    pub fn new(process_id: u32, feat_type: FeatType) -> Self {
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
        let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        ProcessInfo {
            process_id,
            executed_at: now,
            version,
            feat_type,
        }
    }
}
impl RuntimeInfo {
    pub fn add_process_info(&mut self, process_info: ProcessInfo) -> Result<(), RuntimeError> {
        for info in self.process_infos.iter_mut() {
            if info.is_none() {
                *info = Some(process_info);
                return Ok(());
            }
        }
        Err(RuntimeError::FileWrite(std::io::Error::new(
            std::io::ErrorKind::StorageFull,
            "Failed to add process_info",
        )))
    }

    pub fn remove_process_info(&mut self, process_id: u32) -> Result<(), RuntimeError> {
        let pid = process_id;
        let mut i = None;
        for (j, info) in self.process_infos.iter_mut().enumerate() {
            match info.as_ref() {
                Some(ProcessInfo { process_id, .. }) if pid == *process_id => {
                    *info = None;
                    i = Some(j);
                    break;
                }
                _ => continue,
            }
        }
        if let Some(i) = i {
            self.process_infos[i..].rotate_left(1);
            Ok(())
        } else {
            Err(RuntimeError::FileWrite(std::io::Error::new(
                std::io::ErrorKind::StorageFull,
                "Failed to remove process_info",
            )))
        }
    }
}
