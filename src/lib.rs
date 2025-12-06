use std::collections::HashMap;

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
}
// respod manages a pool
#[derive(Debug)]
struct Registry {
    // holding the resources and usage info
    pools: Vec<Pool>,
}
#[derive(Debug)]
struct Clients {
    // holding the clients
}

#[derive(Debug, Clone)]
struct Request {
    location: Option<String>,
    pool_attributes: Option<Vec<String>>, // TODO:  Use btreeset
    resource_attributes: Option<Vec<AttributeSet>>,
}

#[derive(Debug)]
enum RequestError {
    Impossible(String),
    InUse(String),
    NotReady(String),
}

#[derive(Debug, Clone)]
struct PoolLease {
    leasetime: u32,
    pool: Pool,
    pairing: Option<AttributeMatch>,
}

trait Requestable {
    fn request(&self, request: Request) -> Result<PoolLease, RequestError>;
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
impl Requestable for Registry {
    fn request(&self, request: Request) -> Result<PoolLease, RequestError> {
        for pool in &self.pools {
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
            return Ok(PoolLease {
                leasetime: 1234, // TODO: read default lease time from config file
                pool: pool.clone(),
                pairing: None,
            });
        }
        Err(RequestError::Impossible("No match".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_registry() -> Registry {
        Registry {
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
            }],
        }
    }
    fn build_ok_request() -> Request {
        Request {
            location: None,
            pool_attributes: Some(vec!["attr1".into()]),
            resource_attributes: None,
        }
    }
    #[test]
    fn test_ok_pool_attributes() {
        let r = build_simple_registry();
        let ok_request = build_ok_request();
        assert!(r.request(ok_request).is_ok());
    }

    #[test]
    fn test_nok_pool_attributes() {
        let r = build_simple_registry();
        let ok_request = build_ok_request();
        let nok_request = Request {
            pool_attributes: Some(vec!["attr3".into()]),
            ..ok_request.clone()
        };
        assert!(r.request(nok_request).is_err());
    }

    #[test]
    fn test_nok_location() {
        let r = build_simple_registry();
        let ok_request = build_ok_request();
        let nok_request = Request {
            location: Some("abroad".into()),
            ..ok_request.clone()
        };
        assert!(r.request(nok_request).is_err());
    }
    #[test]
    fn test_resource_attributes_match() {
        let r = build_simple_registry();
        let ok_request = build_ok_request();
        let ra_ok_request = Request {
            resource_attributes: Some(vec![vec!["RA1".into()]]),
            ..ok_request.clone()
        };
        assert!(r.request(ra_ok_request.clone()).is_ok());
    }
    #[test]
    fn test_resource_attributes_mismatch() {
        let r = build_simple_registry();
        let ok_request = build_ok_request();
        // Failure case
        let nok_request = Request {
            resource_attributes: Some(vec![vec!["RA3".into()]]),
            ..ok_request.clone()
        };
        assert!(r.request(nok_request).is_err());
    }
}
