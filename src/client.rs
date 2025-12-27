pub mod client {
    use crate::inventory::{
        ResourceRequest
    };

    pub trait RespoClient {
        pub async fn request(&self, request: &ResourceRequest) -> Result<PoolLease, ResourceRequestError>;
    }

    pub struct RespoClientFactory
    {
        url: String
    }

    impl RespoClientFactory {
        pub fn new(url: String) -> RespoClientFactory {
            RespoClientFactory { url }
        }
    }

}
