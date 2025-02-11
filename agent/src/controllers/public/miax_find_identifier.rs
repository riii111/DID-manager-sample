use crate::controllers::errors::MiaXErrorCode;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
};
use protocol::did::sidetree::payload::MiaxDidResponse;

pub async fn handler(did: Path<String>) -> Result<Json<Option<MiaxDidResponse>>, StatusCode> {
    let service = crate::services::miax::MiaX::new();

    match service.find_identifier(&did).await {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("{:?}", e);
            Err(MiaXErrorCode::CreateIdentifierInternal)?
        }
    }
}
