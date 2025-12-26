/*!
Resource Pool Leasing System

This crate provides core logic for a resource leasing system, where clients can request exclusive access to resources
(such as network-connected equipment test benches) based on attributes and location. The system supports matching
requests to available resources, handling pool attributes, resource attributes, and location constraints.

Main Components:
- Resource: Represents an individual entity with attributes and properties.
- Pool: A collection of resources, with its own attributes and location.
- Inventory: Holds all pools and manages resource allocation.
- ResourceRequest: Describes a client's requirements for resource allocation.
- PoolLease: Represents a successful lease of a pool, including resource pairing.
- InventoryResourceRequest: Trait for handling resource requests and matching logic.

Matching Logic:
- ResourceRequests are matched against pools and resources using attribute and location constraints.
- Resource matching uses a simple subset check; assignment problem logic can be extended for optimal pairing.

Unit tests are provided for core matching scenarios.

See README.md for usage, roadmap, and further details.
*/
pub mod config;
pub mod respo {

    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::{Arc, Weak};
    use tokio::sync::{Mutex, Notify};
    use tokio::time::{Duration, Instant, sleep_until};

    const DEFAULT_LEASE_TIME: Duration = Duration::from_secs(1234); // TODO: read default lease time from config file

    type AttributeSet = Vec<String>; // TODO:  Use BTreeSet
    type AttributeMatch = Vec<(AttributeSet, Resource)>;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Resource {
        pub attributes: AttributeSet,
        pub properties: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pool {
        pub name: String,
        pub attributes: AttributeSet,
        pub location: String,
        pub resources: Vec<Resource>,
        #[serde(skip_serializing, skip_deserializing)]
        pub user: Weak<Mutex<InnerClient>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct InnerInventory {
        pools: Vec<Pool>,
    }
    #[derive(Debug, Clone)]
    pub struct Inventory(pub Arc<Mutex<InnerInventory>>);

    #[derive(Debug, Clone, Default, Serialize)]
    pub struct ResourceRequest {
        pub location: Option<String>,
        pub pool_attributes: Option<Vec<String>>, // TODO:  Use btreeset
        pub resource_attributes: Option<Vec<AttributeSet>>,
        pub timeout: Option<Duration>,
        pub by_name: Option<String>, // This will be used to take a pool offline for maintenance
    }

    #[derive(Debug, Serialize)]
    pub enum ResourceRequestError {
        Impossible,
        InUse,
        TimeOut,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct PoolLease {
        leasetime: Duration,
        pool: Pool,
        pairing: Option<AttributeMatch>,
        #[serde(skip_serializing, skip_deserializing)]
        notify: Arc<Notify>,
    }

    impl Drop for PoolLease {
        fn drop(&mut self) {
            println!("notifying waiters");
            self.notify.notify_waiters()
        }
    }
    impl Inventory {
        pub fn new(pools: Vec<Pool>) -> Inventory {
            Inventory(Arc::new(Mutex::new(InnerInventory { pools })))
        }
    }
    //#[async_trait]
    pub trait InventoryResourceRequest {
        fn request(
            &mut self,
            request: &ResourceRequest,
            client: &Arc<tokio::sync::Mutex<InnerClient>>,
            client_notify: &Arc<Notify>,
        ) -> impl std::future::Future<Output = Result<PoolLease, ResourceRequestError>> + Send;
    }

    fn matches(subset: &[String], superset: &[String]) -> bool {
        subset.iter().all(|x| superset.contains(x))
    }

    fn solve_resource_matches(
        pool: &Pool,
        requested_resources_spec: &Vec<AttributeSet>,
    ) -> Option<AttributeMatch> {
        let mut matchlist: AttributeMatch = Vec::new();

        // TODO: properly implement the assignment problem
        for resource_spec in requested_resources_spec {
            let matching_resources: Vec<&Resource> = pool
                .resources
                .iter()
                .filter(|y: &&Resource| matches(resource_spec, &y.attributes))
                .collect();

            if matching_resources.iter().len() > 0 {
                let first_match = matching_resources[0].clone();
                matchlist.push((resource_spec.clone(), first_match));
            } else {
                return None;
            }
        }
        Some(matchlist)
    }

    //#[async_trait]
    impl InventoryResourceRequest for Inventory {
        async fn request(
            &mut self,
            request: &ResourceRequest,
            client: &Arc<tokio::sync::Mutex<InnerClient>>,
            client_notify: &Arc<Notify>,
        ) -> Result<PoolLease, ResourceRequestError> {
            let mut inventory = self.0.lock().await;
            let mut ultimate_failure: ResourceRequestError = ResourceRequestError::Impossible;
            for potential_pool in &mut inventory.pools {
                // skip if request.pool_attributes not a subset of potential_pool.attributes
                if let Some(wanted_pool_attributes) = &request.pool_attributes
                    && !wanted_pool_attributes.iter().all(|requested_attribute| {
                        potential_pool.attributes.contains(requested_attribute)
                    })
                {
                    continue;
                }
                if let Some(wanted_location) = &request.location
                    && *wanted_location != potential_pool.location
                {
                    continue;
                }
                if let Some(requested_resources_spec) = &request.resource_attributes {
                    if let Some(match_) =
                        solve_resource_matches(potential_pool, requested_resources_spec)
                    {
                        return Ok(PoolLease {
                            leasetime: DEFAULT_LEASE_TIME,
                            pool: potential_pool.clone(),
                            pairing: Some(match_),
                            notify: Arc::clone(client_notify),
                        });
                    } else {
                        continue;
                    }
                }
                if let Some(requested_pool_name) = &request.by_name
                    && *requested_pool_name != potential_pool.name
                {
                    continue;
                }
                if potential_pool.user.upgrade().is_none() {
                    println!("claiming item");
                    potential_pool.user = Arc::downgrade(client);
                    return Ok(PoolLease {
                        leasetime: DEFAULT_LEASE_TIME,
                        pool: potential_pool.clone(),
                        pairing: None,
                        notify: Arc::clone(client_notify),
                    });
                } else {
                    println!("item is in use");
                    ultimate_failure = ResourceRequestError::InUse;
                }
            }
            Err(ultimate_failure)
        }
    }

    #[derive(Debug)]
    pub struct InnerClient {
        pub name: String,
        pub inventory: Inventory, // needed to make a request
        pub notify: Arc<Notify>,  // needed to retry a request
    }
    #[derive(Debug)]
    pub struct Client(Arc<Mutex<InnerClient>>);

    //#[async_trait]
    pub trait ClientResourceRequest {
        fn request(
            &mut self,
            request: &ResourceRequest,
        ) -> impl std::future::Future<Output = Result<PoolLease, ResourceRequestError>> + Send;
    }

    //#[async_trait]
    impl ClientResourceRequest for Client {
        async fn request(
            &mut self,
            request: &ResourceRequest,
        ) -> Result<PoolLease, ResourceRequestError> {
            let deadline = request.timeout.map(|timeout| Instant::now() + timeout);
            println!("trying to claim {:?} until {:?}", &request, &deadline);
            loop {
                let mut client = self.0.lock().await;
                let notify = client.notify.clone();
                match client.inventory.request(request, &self.0, &notify).await {
                    Ok(lease) => return Ok(lease),
                    Err(ResourceRequestError::InUse) => {
                        drop(client);
                        if let Some(deadline) = &deadline {
                            tokio::select! {
                                _ = notify.notified() => {
                                    // fall through to retry
                                },
                                _ = sleep_until(*deadline) =>  {
                                    return Err(ResourceRequestError::TimeOut);
                                },
                            }
                        } else {
                            return Err(ResourceRequestError::InUse);
                        }
                    }
                    Err(other) => {
                        return Err(other);
                    }
                }
            }
        }
    }

    pub struct RespoClientFactory {
        inventory: Inventory,
        notify: Arc<Notify>,
    }
    impl Client {
        fn new(inner: InnerClient) -> Client {
            Client(Arc::new(Mutex::new(inner)))
        }
    }

    impl RespoClientFactory {
        pub fn new(inventory: Inventory) -> RespoClientFactory {
            Self {
                inventory: inventory.clone(),
                notify: Arc::new(Notify::new()),
            }
        }
        pub fn create(&mut self, name: String) -> Client {
            Client::new(InnerClient {
                name,
                inventory: self.inventory.clone(),
                notify: self.notify.clone(),
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use respo::{
        Client, ClientResourceRequest, Inventory, Pool, Resource, ResourceRequest,
        ResourceRequestError, RespoClientFactory,
    };
    use std::collections::HashMap;
    use std::sync::Weak;
    use tokio::join;
    use tokio::time::{Duration, sleep};

    fn build_simple_inventory() -> Inventory {
        Inventory::new(vec![Pool {
            name: "pool1".into(),
            attributes: vec!["attr1".into(), "attr2".into()],
            location: "location1".into(),
            resources: vec![
                Resource {
                    attributes: vec!["RA1".into(), "RA2".into()],
                    properties: HashMap::new(),
                },
                Resource {
                    attributes: vec!["RB1".into(), "RB2".into()],
                    properties: HashMap::new(),
                },
            ],
            user: Weak::new(),
        }])
    }
    fn build_simple_clientfactory() -> RespoClientFactory {
        let inventory = build_simple_inventory();
        RespoClientFactory::new(inventory)
    }
    fn build_ok_request() -> ResourceRequest {
        ResourceRequest {
            pool_attributes: Some(vec!["attr1".into()]),
            ..Default::default()
        }
    }
    fn build_simple_client() -> Client {
        let mut clientfactory = build_simple_clientfactory();
        clientfactory.create("client_a".into())
    }
    #[tokio::test]
    async fn test_by_name_positive() {
        let mut client = build_simple_client();
        let ok_request = ResourceRequest {
            by_name: Some("pool1".into()),
            ..Default::default()
        };
        assert!(client.request(&ok_request).await.is_ok());
    }
    #[tokio::test]
    async fn test_by_name_negative() {
        let mut client = build_simple_client();
        let nok_request = ResourceRequest {
            by_name: Some("pool_not_there".into()),
            ..Default::default()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }

    #[tokio::test]
    async fn test_ok_pool_attributes() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        assert!(client.request(&ok_request).await.is_ok());
    }

    #[tokio::test]
    async fn test_nok_pool_attributes() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let nok_request = ResourceRequest {
            pool_attributes: Some(vec!["attr3".into()]),
            ..ok_request.clone()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }

    #[tokio::test]
    async fn test_nok_location() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let nok_request = ResourceRequest {
            location: Some("abroad".into()),
            ..ok_request.clone()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }
    #[tokio::test]
    async fn test_resource_attributes_match() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let ra_ok_request = ResourceRequest {
            resource_attributes: Some(vec![vec!["RA1".into()]]),
            ..ok_request.clone()
        };
        assert!(client.request(&ra_ok_request.clone()).await.is_ok());
    }
    #[tokio::test]
    async fn test_resource_attributes_mismatch() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        // Failure case
        let nok_request = ResourceRequest {
            resource_attributes: Some(vec![vec!["RA3".into()]]),
            ..ok_request.clone()
        };
        let result = client.request(&nok_request).await;
        assert!(
            matches!(result.as_ref(), Err(ResourceRequestError::Impossible)),
            "Unexpected error: {:?}",
            result.as_ref().unwrap_err()
        );
    }
    #[tokio::test]
    async fn test_concurrent_usage_returns_error() {
        let mut clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async {
                assert!(client_a.request(&ok_request.clone()).await.is_ok());
                sleep(Duration::from_secs(1)).await;
            },
            async {
                sleep(Duration::from_millis(100)).await;
                assert!(matches!(
                    client_b.request(&ok_request.clone()).await,
                    Err(ResourceRequestError::InUse)
                ));
            }
        );
    }
    #[tokio::test]
    async fn test_concurrent_timeout() {
        let mut clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();
        let ok_with_timeout = ResourceRequest {
            timeout: Some(Duration::from_millis(500)),
            ..ok_request.clone()
        };

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async move {
                assert!(client_a.request(&ok_request).await.is_ok());
                sleep(Duration::from_secs(1)).await;
            },
            async move {
                sleep(Duration::from_millis(100)).await;
                assert!(matches!(
                    client_b.request(&ok_with_timeout).await,
                    Err(ResourceRequestError::TimeOut)
                ));
            }
        );
    }

    #[tokio::test]
    async fn test_concurrent_becomes_available() {
        // FIXME: test should be using process time instead of walltime
        let mut clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();
        let ok_with_timeout = ResourceRequest {
            timeout: Some(Duration::from_millis(1000)),
            ..ok_request.clone()
        };

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async move {
                let lease = client_a.request(&ok_request).await;
                assert!(lease.is_ok());
                sleep(Duration::from_millis(100)).await;
            },
            async move {
                sleep(Duration::from_millis(100)).await;
                let result = client_b.request(&ok_with_timeout).await;
                assert!(
                    result.is_ok(),
                    "Unexpected error: {:?}",
                    result.unwrap_err()
                );
            }
        );
    }
}
