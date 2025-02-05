use axum::{http::StatusCode, response::Json};
use chrono;
use serde::Serialize;

#[derive(Serialize)]
pub struct CreateIdentifierResponse {
    did: String,
    created_at: String,
}

pub async fn handler() -> Result<Json<CreateIdentifierResponse>, StatusCode> {
    let response = CreateIdentifierResponse {
        did: "did:miax:test123".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}
