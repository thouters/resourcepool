/*!
Resource Pool Leasing System

This crate provides core logic for a resource leasing system, where clients can request exclusive access to resources
(such as network-connected equipment test benches) based on attributes and location. The system supports matching
requests to available resources, handling pool attributes, resource attributes, and location constraints.

Main Components:
- Resource: Represents an individual entity with attributes and properties.
- Pool: A collection of resources, with its own attributes and location.
- Inventory: Holds all pools and manages resource allocation.
- Request: Describes a client's requirements for resource allocation.
- PoolLease: Represents a successful lease of a pool, including resource pairing.
- Requestable: Trait for handling resource requests and matching logic.

Matching Logic:
- Requests are matched against pools and resources using attribute and location constraints.
- Resource matching uses a simple subset check; assignment problem logic can be extended for optimal pairing.

Unit tests are provided for core matching scenarios.

See README.md for usage, roadmap, and further details.
*/

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::time::{sleep, Sleep, Duration};
use tokio::join;




type AttributeSet = Vec<String>;
type AttributeMatch = Vec<(AttributeSet, Resource)>;

#[derive(Debug, Clone)]
struct Resource {
    attributes: Vec<String>, // TODO:  Use HashSet
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct Pool {
    name: String,
    attributes: Vec<String>, // TODO:  Use btreeset
    location: String,
    resources: Vec<Resource>,
    user: Option<bool>
}
// respod manages a pool
#[derive(Debug)]
struct Inventory {
    // holding the resources and usage info
    pools: Vec<Pool>,
}

#[derive(Debug, Clone)]
struct Request {
    location: Option<String>,
    pool_attributes: Option<Vec<String>>, // TODO:  Use btreeset
    resource_attributes: Option<Vec<AttributeSet>>,
    timeout: Option<Duration>
}

#[derive(Debug)]
enum RequestError {
    Impossible,
    InUse,
    NotReady,
    TimeOut
}

#[derive(Debug, Clone)]
struct PoolLease {
    leasetime: u32,
    pool: Pool,
    pairing: Option<AttributeMatch>,
}

#[async_trait]
trait Requestable {
    async fn request(&mut self, request: &Request) -> Result<PoolLease, RequestError>;
}

fn matches(subset: &Vec<String>, superset: &Vec<String>) -> bool {
    subset.iter().all(|x| superset.contains(x))
}

fn solve_resource_matches(
    pool: &Pool,
    requested_resources_spec: &Vec<AttributeSet>,
) -> Option<AttributeMatch> {
    // pair up all items from the list so that they match.
    let mut matchlist: AttributeMatch = Vec::new();

    // FIXME: properly implement the assignment problem
    for resource_spec in requested_resources_spec {
        let matching_resources: Vec<&Resource> = pool
            .resources
            .iter()
            .filter(|y: &&Resource| matches(resource_spec, &y.attributes))
            .collect();
        if matching_resources.iter().count() > 0 {
            matchlist.push((resource_spec.clone(), matching_resources[0].clone()));
        } else {
            return None;
        }
    }
    Some(matchlist)
}

#[async_trait]
impl Requestable for Inventory {
    async fn request(&mut self, request: &Request) -> Result<PoolLease, RequestError> {
        let mut impossible = true;
        for pool in &mut self.pools {
            // TODO: there should be a more functional way to express this
            // skip if request.pool_attributes not a subset of pool.attributes
            if let Some(wanted_attributes) = &request.pool_attributes {
                if !wanted_attributes
                    .iter()
                    .all(|requested_attribute| pool.attributes.contains(requested_attribute))
                {
                    continue;
                }
            }
            // TODO: check there is an unique resource per set of resource_attributes
            if let Some(wanted_location) = &request.location {
                if *wanted_location != pool.location {
                    continue;
                }
            }
            if let Some(requested_resources_spec) = &request.resource_attributes {
                if let Some(match_) = solve_resource_matches(pool, requested_resources_spec) {
                    return Ok(PoolLease {
                        leasetime: 1234, // TODO: read default lease time from config file
                        pool: pool.clone(),
                        pairing: Some(match_),
                    });
                } else {
                    continue;
                }
            }
            if let Some(_user) = pool.user {
                impossible = false;
            } else {
                //FIXME make self mutable and set this
                pool.user = Some(true);
                return Ok(PoolLease {
                    leasetime: 1234, // TODO: read default lease time from config file
                    pool: pool.clone(),
                    pairing: None,
                });
            }
        }
        match impossible {
            true => { Err(RequestError::Impossible) }
            false => { Err(RequestError::InUse) }
        }
    }
}

struct LocalClient {
    name: String,
    inventory: Arc<Mutex<Inventory>>,
    notify: Arc<Notify>
}
impl LocalClient {
    async fn request(&self, request: Request) -> Result<PoolLease, RequestError> {
        let mut timeout_timer: Option<Sleep> = None;
        if let Some(timeout) = request.timeout.clone() {
            timeout_timer = Some(sleep(timeout));
        }
        loop {
            match self.inventory.lock().unwrap().request(&request).await {
                Ok(lease) => { return Ok(lease) }
                Err(RequestError::InUse) => {
                    if let Some(timeout_timer) = timeout_timer {
                        tokio::select! {
                            _ = self.notify.notified() => { 
                                // fall through to retry
                            },
                            _ = timeout_timer =>  {
                                return Err(RequestError::TimeOut);
                            },
                        }
                        //sleeping on an event that gets notified of changes to the usage
                    } else {
                        return Err(RequestError::InUse);
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
    inventory: Arc<Mutex<Inventory>>,
    notify: Arc<Notify>
}

impl RespoClientFactory {
    fn new(inventory: Inventory) -> RespoClientFactory  {
        Self {
            inventory: Arc::new(Mutex::new(inventory)),
            notify: Arc::new(Notify::new())
        }
    }
    fn create(&mut self, name: String) -> LocalClient {
        LocalClient {
            name: name,
            inventory:  Arc::clone(&self.inventory),
            notify:  Arc::clone(&self.notify)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_inventory() -> Inventory {
        Inventory {
            pools: vec![Pool {
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
                user: None
            }],
        }
    }
    fn build_simple_clientfactory() -> RespoClientFactory {
        let inventory = build_simple_inventory();
        RespoClientFactory::new(inventory)
    }
    fn build_ok_request() -> Request {
        Request {
            location: None,
            pool_attributes: Some(vec!["attr1".into()]),
            resource_attributes: None,
            timeout: None
        }
    }
    #[tokio::test]
    async fn test_ok_pool_attributes() {
        let mut inventory = build_simple_inventory();
        let ok_request = build_ok_request();
        assert!(inventory.request(&ok_request).await.is_ok());
    }

    #[tokio::test]
    async fn test_nok_pool_attributes() {
        let mut inventory = build_simple_inventory();
        let ok_request = build_ok_request();
        let nok_request = Request {
            pool_attributes: Some(vec!["attr3".into()]),
            ..ok_request.clone()
        };
        assert!(inventory.request(&nok_request).await.is_err());
    }

    #[tokio::test]
    async fn test_nok_location() {
        let mut inventory = build_simple_inventory();
        let ok_request = build_ok_request();
        let nok_request = Request {
            location: Some("abroad".into()),
            ..ok_request.clone()
        };
        assert!(inventory.request(&nok_request).await.is_err());
    }
    #[tokio::test]
    async fn test_resource_attributes_match() {
        let mut inventory = build_simple_inventory();
        let ok_request = build_ok_request();
        let ra_ok_request = Request {
            resource_attributes: Some(vec![vec!["RA1".into()]]),
            ..ok_request.clone()
        };
        assert!(inventory.request(&ra_ok_request.clone()).await.is_ok());
    }
    #[tokio::test]
    async fn test_resource_attributes_mismatch() {
        let mut inventory = build_simple_inventory();
        let ok_request = build_ok_request();
        // Failure case
        let nok_request = Request {
            resource_attributes: Some(vec![vec!["RA3".into()]]),
            ..ok_request.clone()
        };
        assert!(inventory.request(&nok_request).await.is_err());
    }
    #[tokio::test]
    async fn test_concurrent_usage_returns_error() {
        let mut clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();

        let client_a = clientfactory.create("client_a".into());
        let client_b = clientfactory.create("client_b".into());

        let a = async {
            println!("'a' started.");
            assert!(client_a.request(ok_request.clone()).await.is_ok());
            sleep(Duration::from_secs(1)).await;
            println!("'a' finished.");
        };
        let b = async {
            println!("'b' started.");
            sleep(Duration::from_millis(100)).await;
            assert!(client_b.request(ok_request.clone()).await.is_err());
        };
        join!(a, b);
    }
}
