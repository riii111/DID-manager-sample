use crate::controllers;
use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new().route(
        "/miax/create_identifier",
        post(controllers::create_identifier::handle_create_identifier),
    )
}
