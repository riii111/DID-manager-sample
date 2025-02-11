use crate::{controllers::errors::MiaXErrorCode, services::miax::MiaX};
use axum::{http::StatusCode, response::Json};
use protocol::did::sidetree::payload::MiaxDidResponse;

pub async fn handler() -> Result<Json<MiaxDidResponse>, StatusCode> {
    let service = MiaX::new();
    match service.create_identifier().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(MiaXErrorCode::CreateIdentifierInternal)?
        }
    }
}
