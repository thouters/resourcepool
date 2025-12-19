
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::{Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use url::Url;
use rp::respo::{Inventory, Pool, Resource, Client, ResourceRequest, RespoClientFactory, ResourceRequestError, ClientResourceRequest};
use std::sync::{Arc, Weak};

fn build_simple_inventory() -> Inventory {
	Inventory::new( vec![Pool {
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
			user: Weak::new()
		}]
	)
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

async fn handle_request(_inventory: Inventory, request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
	let uri_string = request.uri().to_string();
	let request_url = Url::parse("http://localhost").unwrap().join(&uri_string).unwrap();
	let params = request_url.query_pairs();
	let poolname = params.filter_map(|(poolname, b)| {
	 	if poolname=="poolname" {
			Some(b)
		} else {None}
	}).next();
	dbg!(&poolname);
	match poolname {
		Some(value) => {
            let mut client_a = build_simple_client();
            let request = ResourceRequest {
                by_name: Some(String::from(value)),
                ..Default::default()
            };

            let lease = client_a.request(&request).await;
            match lease {
                Ok(lease) => {
                    Ok(Response::new(Full::new(Bytes::from(format!("got {:?}",lease)))))
                }
                Err(x) => {
                    Ok(Response::new(Full::new(Bytes::from(format!("got an error: {:?}",x)))))
                }
            }
		}
		None => {
			Ok(Response::new(Full::new(Bytes::from("No value specified"))))
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    let inventory = build_simple_inventory();
    // We start a loop to continuously accept incoming connections
    loop {
        let onion1 = inventory.clone();
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(
                move |req: Request<hyper::body::Incoming>| {
                let onion2 = onion1.clone();
                async move {
                    handle_request(onion2, req).await
                }
                }
                ))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

