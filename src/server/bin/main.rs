use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use rp::inventory::{
    Client, ClientResourceRequest, Inventory, Pool, Resource, ResourceRequest, RespoClientFactory,
};

//use rp::config::InventoryLoader;
use std::sync::Weak;
use std::{env, path::PathBuf};
use tokio::net::TcpListener;
use tokio::time::Duration;
use url::Url;

use clap::{Parser, Subcommand};

fn get_default_config_path() -> PathBuf {
    let mut path = env::current_dir().unwrap();
    path.push("respod.yaml");
    path
}

/// Resource pool client tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, default_value=get_default_config_path().into_os_string())]
    /// configuration file (default respod.yaml)
    config_path: PathBuf,
    #[arg(short, long)]
    /// logfile path
    log: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Locks a pool for maintenance
    Serve,
}

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
fn build_simple_client() -> Client {
    let mut clientfactory = build_simple_clientfactory();
    clientfactory.create("client_a".into())
}

async fn handle_request(
    _inventory: Inventory,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let uri_string = request.uri().to_string();
    let request_url = Url::parse("http://localhost")
        .unwrap()
        .join(&uri_string)
        .unwrap();
    let params = request_url.query_pairs();
    if params.count() == 0 {
        return Ok(Response::new(Full::new(Bytes::from("No value specified"))));
    }
    let mut request = ResourceRequest::default();

    for (key, value) in params {
        match key {
            Cow::Borrowed("location") => request.location = Some(String::from(value)),
            Cow::Borrowed("by_name") => request.by_name = Some(String::from(value)),
            Cow::Borrowed("pool_attributes") => {
                let attribute_list: Vec<String> = value.split(",").map(String::from).collect();
                request.pool_attributes = Some(attribute_list);
            }
            Cow::Borrowed("resource_attributes") => {
                let resource_attributes: Vec<String> = value.split(",").map(String::from).collect();
                match &mut request.resource_attributes {
                    None => {
                        request.resource_attributes = Some(vec![resource_attributes]);
                    }
                    Some(existing_list) => {
                        existing_list.push(resource_attributes);
                    }
                }
            }
            Cow::Borrowed("timeout") => {
                let value = value.parse::<u64>();
                match value {
                    Ok(value) => {
                        let value = Duration::new(value, 0);
                        request.timeout = Some(value);
                    }
                    Err(e) => {
                        return Ok(Response::new(Full::new(Bytes::from(format!(
                            "parse error: {:?}",
                            e
                        )))));
                    }
                }
            }
            _ => {
                todo!("error");
            }
        }
    }
    dbg!(&request);

    let mut client_a = build_simple_client();
    let lease = client_a.request(&request).await;
    match lease {
        Ok(lease) => {
            let json = serde_json::to_string_pretty(&lease);
            match json {
                Ok(json) => Ok(Response::new(Full::new(Bytes::from(json)))),
                Err(x) => Ok(Response::new(Full::new(Bytes::from(format!(
                    "got an error: {:?}",
                    x
                ))))),
            }
        }
        Err(x) => Ok(Response::new(Full::new(Bytes::from(format!(
            "got an error: {:?}",
            x
        ))))),
    }
}

async fn http_serve(inventory: Inventory) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // TODO: make configurable

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

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
                .serve_connection(
                    io,
                    service_fn(move |req: Request<hyper::body::Incoming>| {
                        let onion2 = onion1.clone();
                        async move { handle_request(onion2, req).await }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();

    match args.command {
        Commands::Serve => {
            args.config_path
                .try_exists()
                .expect("Can't check existence of file or config does not exist");
        }
    }

    //let inventory = InventoryLoader::load(args.config_path);
    let inventory = build_simple_inventory();
    http_serve(inventory).await
}
