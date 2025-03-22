use bytes::Bytes;
use http_body_util::{BodyExt, Full}; // BodyExt 拡張トレイト
use hyper::{body::Incoming, Response};
use hyper_util::client::legacy::{Client, Error as LegacyClientError};
use hyperlocal::{UnixClientExt, UnixConnector, Uri}; // UnixClientExt 拡張トレイト
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn remove_file_if_exists(path: impl AsRef<Path>) {
    if path.as_ref().exists() {
        std::fs::remove_file(path).unwrap();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetFdError {
    #[error("LISTEN_FDS not set or invalid")]
    ListenFdsError,
    #[error("LISTEN_PID not set or invalid")]
    ListenPidError,
    #[error("LISTEN_PID ({listen_pid}) does not match current process ID ({current_pid})")]
    ListenPidMismatch { listen_pid: i32, current_pid: i32 },
    #[error("No file descriptors passed by systemd.")]
    NoFileDescriptors,
}

#[derive(Debug, thiserror::Error)]
pub enum GetRequestError {
    #[error("Failed to collect body: {0}")]
    CollectBody(#[from] hyper::Error),
    #[error("Failed to convert body to string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Failed to parse JSON response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Request failed: {0}")]
    RequestFailed(#[from] LegacyClientError),
}

pub async fn parse_response_body<T>(response: Response<Incoming>) -> Result<T, GetRequestError>
where
    T: DeserializeOwned,
{
    let collected_body = response.into_body().collect().await?;
    let bytes = collected_body.to_bytes();
    let string_body = std::str::from_utf8(bytes.as_ref())?;
    Ok(serde_json::from_str(string_body)?)
}
pub async fn get_request<T>(
    uds_path: impl AsRef<Path>,
    endpoint: &str,
) -> Result<T, GetRequestError>
where
    T: DeserializeOwned + Send,
{
    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let uri = Uri::new(uds_path, endpoint).into();
    let response: Response<Incoming> = client.get(uri).await?;
    parse_response_body(response).await
}
