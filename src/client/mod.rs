pub mod http;
use crate::client::http::{build_query, try_request};

use crate::inventory::{PoolLease, ResourceRequest, ResourceRequestError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientResourceRequestError {
    #[error("Unable to claim lease: {0}")]
    InventoryError(ResourceRequestError),
    #[error("URL error: {0}")]
    InvalidUriError(#[from] ::http::uri::InvalidUri),
    #[error("HTTP Communication error: {0}")]
    HTTPCommunicationError(#[from] ::http::Error),
    #[error("JSON Parsing error: {0}")]
    JsonParsingError(#[from] serde_json::Error),
    #[error("Hyper error: {0}")]
    HyperError(#[from] hyper::Error),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}
pub struct RemoteRespoClientFactory {
    url: String,
}

pub struct RemoteRespoClient {
    name: String,
    url: String,
}
pub fn create_client_name() -> String {
    format!(
        "{}@{}",
        std::env::var("USER").unwrap_or(String::from("user")),
        std::env::var("HOSTNAME").unwrap_or(String::from("host"))
    )
}

impl RemoteRespoClient {
    pub async fn request(
        &mut self,
        request: &ResourceRequest,
    ) -> Result<PoolLease, ClientResourceRequestError> {
        println!("request: {:?}", request);
        let request = build_query(Some(self.name.clone()), request);
        try_request(format!("{}?{}", self.url, request)).await
        // TODO: launch a thread keeping the connection alive.
        // shutdown the thread in the drop() of the lease.
    }
}

impl RemoteRespoClientFactory {
    pub fn new(url: String) -> RemoteRespoClientFactory {
        RemoteRespoClientFactory { url }
    }

    pub fn create(&mut self, name: String) -> RemoteRespoClient {
        RemoteRespoClient {
            name,
            url: self.url.clone(),
        }
    }
}
