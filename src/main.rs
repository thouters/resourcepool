use std::collections::HashMap;
#[derive(Debug, Clone)]
struct Resource {
    attributes: Vec<String>, // TODO:  Use HashSet
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct Pool {
    name: String,
    attributes: Vec<String>, // TODO:  Use HashSet
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

#[derive(Debug)]
struct Request {
    location: Option<String>,
    pool_attributes: Option<Vec<String>>, // TODO:  Use HashSet
    resource_attributes: Option<Vec<Vec<String>>>,
}

#[derive(Debug)]
enum RequestError {
    Impossible(String),
    InUse(String),
    NotReady(String),
}

#[derive(Debug)]
struct PoolLease {
    leasetime: u32,
    pool: Pool,
}

trait Requestable {
    fn request(&self, request: Request) -> Result<PoolLease, RequestError>;
}

impl Requestable for Registry {
    fn request(&self, request: Request) -> Result<PoolLease, RequestError> {
        for pool in &self.pools {
            // TODO: there should be a more functional way to express this
            // skip if request.pool_attributes not a subset of pool.attributes
            if let Some(wanted_attributes) = &request.pool_attributes {
                if wanted_attributes
                    .iter()
                    .filter(
                        // QUESTION: making this a attribute_in_pool closure does not work!
                        |requested_attribute| {
                            pool.attributes
                                .iter()
                                .filter(|available_attribute| {
                                    requested_attribute == available_attribute
                                })
                                .count()
                                > 0
                        },
                    )
                    .count()
                    < request.pool_attributes.iter().count()
                {
                    continue;
                }
            }
            // TODO: check there is an unique resource per set of resource_attributes
            // TODO: test location
            return Ok(PoolLease {
                leasetime: 1234, // TODO: read default lease time from config file
                pool: pool.clone(),
            });
        }
        Err(RequestError::Impossible("No match".into()))
    }
}

fn main() {
    // TODO: move to unittest (-ENOTIME today)
    //hello world
    let r = Registry {
        pools: vec![Pool {
            name: "pool1".into(),
            attributes: vec!["attr1".into(), "attr2".into()],
            location: "location1".into(),
            resources: vec![],
        }],
    };
    println!("Given we have the registry with pools: {:?}", r);

    // sunny day test
    let ok_request = Request {
        location: None,
        pool_attributes: Some(vec!["attr1".into()]),
        resource_attributes: None,
    };
    println!("When I request {:?}", ok_request);
    let ok_result = r.request(ok_request);

    println!("I get poollease {:?}", ok_result);
    assert!(ok_result.is_ok());

    // Failure case test
    let nok_request = Request {
        location: None,
        pool_attributes: Some(vec!["attr3".into()]),
        resource_attributes: None,
    };
    println!("When I request {:?}", nok_request);
    let nok_result = r.request(nok_request);

    println!("I get an error {:?}", nok_result);
    assert!(!nok_result.is_ok());

    ()
}
