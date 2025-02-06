use crate::{controllers::errors::MiaXErrorCode, services::miax::MiaX};
use axum::{http::StatusCode, response::Json};

pub struct MiaxDidResponse {
    pub did_document: DidDocument,
}

pub async fn handler() -> Result<Json<MiaxDidResponse>, StatusCode> {
    let service = MiaX::new();
    match service.create_identifier().await {
        Err(e) => {
            log::error!("ERROR: Failure to generate DID");
            Err(MiaXErrorCode::CreateIdentifierInternal)
        }
    }
}
