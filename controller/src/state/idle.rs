use crate::managers::runtime::{RuntimeError, RuntimeManager};

#[derive(Debug, thiserror::Error)]
pub enum IdleError {
    #[error("failed to get runtime info: {0}")]
    RuntimeError(#[from] RuntimeError),
}

pub async fn execute<T: RuntimeManager>(runtime_manager: &mut T) -> Result<(), IdleError> {
    if !runtime_manager.get_runtime_info()?.is_agent_running() {
        let _process_info = runtime_manager.launch_agent(true)?;
    } else {
        log::error!("Agent already running");
    }
    log::info!("No state change required.");
    Ok(())
}
