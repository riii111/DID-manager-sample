use crate::config::get_config;
use crate::managers::runtime::{ProcessManager, RuntimeInfoStorage, RuntimeManagerImpl};
use crate::state::handler::handle_state;
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
pub mod managers;
pub mod state;
pub mod validator;

#[tokio::main]
pub async fn run() -> std::io::Result<()> {
    #[cfg(unix)]
    let handler = crate::managers::mmap_storage::MmapHandler::new("miax_runtime_info")
        .expect("Failed to create MmapHanlder");
    #[cfg(windows)]
    let handler = {
        let path = get_config()
            .lock()
            .unwrap()
            .runtime_dir
            .join("runtime_info.json");
        crate::managers::file_storage::FileHandler::new(path).expect("Failed to create FileHandler")
    };
    let uds_path = get_config().lock().unwrap().uds_path.clone();
    let (runtime_manager, mut state_rx) =
        RuntimeManagerImpl::new_by_controller(handler, ProcessManagerImpl {}, uds_path)
            .expect("Failed to create RuntimeManager");

    let runtime_manager = Arc::new(Mutex::new(runtime_manager));
    let shutdown_handle = tokio::spawn(handle_signals(runtime_manager.clone()));

    tokio::spawn(async move {
        let mut description = "Initial state";
        while {
            let current_state = *state_rx.borrow();
            log::info!("Worker: {}: {:?}", description, current_state);
            {
                let mut _runtime_manager = runtime_manager.lock().await;
                if let Err(e) = handle_state(current_state, &mut *_runtime_manager).await {
                    log::error!("Worker: Failed to handle {}: {}", description, e);
                }
            }
            description = "State change";
            state_rx.changed().await.is_ok()
        } {}
    });

    let _ = shutdown_handle.await;
    log::info!("Shutdown handler completed successfully.");

    Ok(())
}

#[cfg(unix)]
pub async fn handle_signals<H, P>(runtime_manager: Arc<Mutex<RuntimeManagerImpl<H, P>>>)
where
    H: RuntimeInfoStorage + Send + Sync + 'static,
    P: ProcessManager + Send + Sync + 'static,
{
    unimplemented()
}
