pub mod http;
use crate::client::http::{build_query, test};

use crate::inventory::{ClientResourceRequest, PoolLease, ResourceRequest, ResourceRequestError};

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

impl ClientResourceRequest for RemoteRespoClient {
    async fn request(
        &mut self,
        request: &ResourceRequest,
    ) -> Result<PoolLease, ResourceRequestError> {
        println!("request: {:?}", request);
        let request = build_query(Some(self.name.clone()), request);
        test(format!("{}?{}", self.url, request)).await.unwrap();
        // TODO: launch a thread keeping the connection alive.
        // shutdown the thread in the drop() of the lease.
        Err(ResourceRequestError::Impossible)
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
