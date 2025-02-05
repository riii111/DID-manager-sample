use crate::controllers;
use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new().route(
        "/create_identifier",
        post(post(controllers::public::miax_create_identifier::handler)),
    )
}
