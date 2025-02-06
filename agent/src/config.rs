use std::env;

#[derive(Debug)]
pub struct ServerConfig {
    did_http_endpoint: String,
    did_attachment_link: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerConfig {
    pub fn new() -> ServerConfig {
        let did_endpoint =
            env::var("MIAX_DID_HTTP_ENDPOINT").unwrap_or("https://did.miacross.io".to_string());
        let link =
            env::var("MIAX_DID_ATTACHMENT_LINK").unwrap_or("https://did.miacross.io".to_string());

        ServerConfig {
            did_http_endpoint: did_endpoint,
            did_attachment_link: link,
        }
    }

    pub fn did_http_endpoint(&self) -> String {
        self.did_http_endpoint.clone()
    }
    pub fn did_attachment_link(&self) -> String {
        self.did_attachment_link.clone()
    }
}

pub fn server_config() -> ServerConfig {
    ServerConfig::new()
}
