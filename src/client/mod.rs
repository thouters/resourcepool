pub mod http;
use crate::client::http::test;

use crate::inventory::{ClientResourceRequest, PoolLease, ResourceRequest, ResourceRequestError};

pub struct RemoteRespoClientFactory {
    url: String,
}

pub enum RemoteRespoClient {
    RespoHttpClient { name: String, url: String },
}

impl ClientResourceRequest for RemoteRespoClient {
    async fn request(
        &mut self,
        request: &ResourceRequest,
    ) -> Result<PoolLease, ResourceRequestError> {
        println!("request: {:?}", request);
        test().await.unwrap();
        Err(ResourceRequestError::Impossible)
    }
}

impl RemoteRespoClientFactory {
    pub fn new(url: String) -> RemoteRespoClientFactory {
        RemoteRespoClientFactory { url }
    }

    pub fn create(&mut self, name: String) -> RemoteRespoClient {
        RemoteRespoClient::RespoHttpClient {
            name,
            url: self.url.clone(),
        }
    }
}
