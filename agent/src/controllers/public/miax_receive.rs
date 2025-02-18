use crate::services::{miax::MiaX, studio::Studio};

struct MessageReceiveUsecase {
    studio: Studio,
    agent: MiaX,
    project_did: String,
}

impl MessageReceiveUsecase {
    pub fn new() -> Self {
        let network = crate::network_config();
        let network = network.lock();
        let project_did = if let Some(v) = network.getproject_did() {
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
