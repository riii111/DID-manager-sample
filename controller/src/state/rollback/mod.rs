use crate::managers::{
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{RuntimeError, RuntimeManager},
};

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("failed to kill process: {0}")]
    FailedKillOwnProcess(String),
    #[error("Failed to get current executable path: {0}")]
    CurrentExecutablePathError(#[source] std::io::Error),
}

pub async fn execute<'a, R, T>(
    resource_manager: &'a R,
    runtime_manager: &'a mut T,
) -> Result<(), RollbackError>
where
    R: ResourceManagerTrait,
    T: RuntimeManager,
{
    log::info!("Starting rollback");

    let latest_backup = resource_manager.get_latest_backup();
    match latest_backup {
        Some(backup_file) => {
            let agent_path = runtime_manager.get_runtime_info()?.exec_path;
            log::info!("Found backup: {}", backup_file.display());
            resource_manager.rollback(&backup_file)?;
            if let Err(err) = resource_manager.remove() {
                log::error!("Failed to remove files {}", err);
            }
            runtime_manager.update_state_without_send(crate::managers::runtime::State::Idle)?;
            runtime_manager.launch_controller(agent_path)?;
            log::info!("Rollback completed");

            #[cfg(not(test))]
            {
                log::info!("Restarting controller by SIGTERM");
                let runtime_info = runtime_manager.get_runtime_info()?;
                let self_info = runtime_info.find_process_info(std::process::id()).ok_or(
                    RollbackError::FailedKillOwnProcess("Failed to find self info".into()),
                )?;
                runtime_manager.kill_process(self_info)?;
            }
            Ok(())
        }
        None => Err(RollbackError::BackupNotFound),
    }
}
