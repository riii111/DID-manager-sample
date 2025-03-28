use std::time::Duration;

use crate::{
    miax::utils::did_accessor::{DidAccessor, DidAccessorImpl},
    services::{
        miax::MiaX,
        studio::{MessageResponse, Studio},
    },
};
use anyhow::anyhow;
use controller::validator::network::can_connect_to_download_server;
use protocol::didcomm::encrypted::DidCommEncryptedService;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
enum OperationType {
    UpdateAgent,
    UpdateNetworkJson,
}

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

    async fn handle_invalid_json(
        &self,
        m: &MessageResponse,
        e: serde_json::Error,
    ) -> Result<(), anyhow::Error> {
        self.studio
            .ack_message(&self.project_did, m.id.clone(), false)
            .await?;
        Err(anyhow::anyhow!("Invalid Json: {:?}", e))
    }

    pub async fn receive_message(&self) -> anyhow::Result<()> {
        for m in self.studio.get_message(&self.project_did).await? {
            let json_message = match serde_json::from_str(&m.raw_message) {
                Ok(msg) => msg,
                Err(e) => return self.handle_invalid_json(&m, e).await,
            };
            log::info!("Receive message, message_id = {:?}", m.id);
            match DidCommEncryptedService::verify(
                self.agent.did_repository(),
                &DidAccessorImpl {}.get_my_keyring(),
                &json_message,
            )
            .await
            {
                Ok(verified) => {
                    log::info!(
                        "Verify success. message_id = {}, from = {}",
                        m.id,
                        verified.message.issuer.id
                    );
                    self.studio
                        .ack_message(&self.project_did, m.id, true)
                        .await?;
                    if verified.message.issuer.id == self.project_did {
                        let container = verified.message.credential_subject.container;
                        let operation_type = container["operation"].clone();
                        match serde_json::from_value::<OperationType>(operation_type) {
                            Ok(OperationType::UpdateAgent) => {
                                let binary_url = container["binary_url"]
                                    .as_str()
                                    .ok_or(anyhow!("the container doesn't have binary_url"))?;
                                if !can_connect_to_download_server("https://github.com").await {
                                    log::error!("Not connected to be Internet");
                                } else if !binary_url.starts_with(
                                    "https://github.com/nodecross/nodex/releases/download/",
                                ) {
                                    log::error!("Invalid url");
                                    anyhow::bail!("Invalid url");
                                }
                                self.agent.update_version(binary_url).await?;
                            }
                            Ok(OperationType::UpdateNetworkJson) => {
                                self.studio.network().await?;
                            }
                            Err(e) => {
                                log::error!("Json Parse Error: {:?}", e);
                            }
                        }
                        continue;
                    } else {
                        log::error!("Not supported")
                    }
                }
                Err(_) => {
                    log::error!("Verify failed : message_id = {}", m.id);
                    self.studio
                        .ack_message(&self.project_did, m.id, false)
                        .await?;
                    continue;
                }
            }
        }

        Ok(())
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
