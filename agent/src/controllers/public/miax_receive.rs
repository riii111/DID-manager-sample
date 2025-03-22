use std::time::Duration;

use crate::services::{miax::MiaX, studio::Studio};
use tokio_util::sync::CancellationToken;

struct MessageReceiveUsecase {
    studio: Studio,
    agent: MiaX,
    project_did: String,
}

impl MessageReceiveUsecase {
    pub fn new() -> Self {
        let network = crate::network_config();
        let network = network.lock();
        let project_did = if let Some(v) = network.get_project_did() {
            v
        } else {
            panic!("Failed to read project_did")
        };
        drop(network);

        Self {
            studio: Studio::new(),
            agent: MiaX::new(),
            project_did,
        }
    }

    pub async fn receive_message(&self) -> anyhow::Result<()> {
        unimplemented!("receive message")
    }
}

pub async fn polling_task(shutdown_token: CancellationToken) {
    log::info!("Polling task is started");

    let usecase = MessageReceiveUsecase::new();

    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                match usecase.receive_message().await {
                    Ok(_) => {},
                    Err(e) => log::error!("Error: {:?}", e),
                }
            }
            _ = shutdown_token.cancelled() => {
                break;
            }
        }
    }

    log::info!("Polling task is stopped")
}
