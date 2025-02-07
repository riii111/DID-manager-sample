use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiaXErrorCode {
    #[error("Internal Server Error")]
    CreateIdentifierInternal = 5004,
}

impl From<MiaXErrorCode> for StatusCode {
    fn from(code: MiaXErrorCode) -> Self {
        let code = code as u16;
        if (5000..6000).contains(&code) {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            unimplemented!("unimplemented error handling")
        }
    }
}
