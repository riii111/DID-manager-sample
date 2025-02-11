use crate::controllers;
use axum::{routing::get, routing::post, Router};

pub fn make_router() -> Router {
    Router::new()
        .route(
            "/create_identifier",
            post(controllers::public::miax_create_identifier::handler),
        )
        .route(
            "/identifiers/:did",
            get(controllers::public::miax_find_identifier::handler),
        )
}
