use std::convert::Infallible;
use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{env, path::PathBuf};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;

use tokio::net::TcpListener;
use tokio::time::Duration;
use url::Url;

use clap::{Parser, Subcommand};

use rp::config::InventoryLoader;
use rp::inventory::{
    ClientResourceRequest, Inventory, InventoryManager, LocalRespoClient, LocalRespoClientFactory,
    ResourceRequest,
};

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

async fn handle_request(
    client_factory: Arc<LocalRespoClientFactory>,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let uri_string = request.uri().to_string();
    let request_url = Url::parse(&uri_string).unwrap();
    let params = request_url.query_pairs();
    if params.count() == 0 {
        let mut resp = Response::new(Full::new(Bytes::from("No value specified")));
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(resp);
    }
    let mut request = ResourceRequest::default();
    let mut client_name: Option<String> = None;

    for (key, value) in params {
        match &*key {
            "client_name" => client_name = Some(String::from(value)),
            "location" => request.location = Some(String::from(value)),
            "by_name" => request.by_name = Some(String::from(value)),
            "pool_attributes" => {
                let attribute_list: Vec<String> = value.split(",").map(String::from).collect();
                request.pool_attributes = Some(attribute_list);
            }
            "resource_attributes" => {
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
            "timeout" => {
                let value = value.parse::<u64>();
                match value {
                    Ok(value) => {
                        let value = Duration::new(value, 0);
                        request.timeout = Some(value);
                    }
                    Err(e) => {
                        let mut resp =
                            Response::new(Full::new(Bytes::from(format!("parse error: {:?}", e))));
                        *resp.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(resp);
                    }
                }
            }
            _ => {
                let mut resp = Response::new(Full::new(Bytes::from(format!(
                    "key not recognised: {:?}",
                    key
                ))));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(resp);
            }
        }
    }
    dbg!(&request);

    let mut client_a: LocalRespoClient =
        client_factory.create(client_name.unwrap_or("no-name".into()));
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

async fn http_serve(
    client_factory: LocalRespoClientFactory,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // TODO: make configurable

    let listener = TcpListener::bind(addr).await?;
    let client_factory = Arc::new(client_factory);

    loop {
        let onion1 = client_factory.clone();
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
            let f = File::open(args.config_path).unwrap();
            let parsed: Inventory = InventoryLoader::load(f);
            let manager = InventoryManager::new(parsed);
            let client_factory = LocalRespoClientFactory::new(manager);
            http_serve(client_factory).await
        }
    }
}
