use http_body_util::Empty;
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::inventory::ResourceRequest;

pub fn build_query(client_name: Option<String>, request: &ResourceRequest) -> String {
    let mut query: Vec<String> = Vec::new();
    if let Some(client_name) = client_name {
        let mut client_name_part = String::from("client_name=");
        client_name_part.push_str(client_name.as_str());
        query.push(client_name_part);
    }
    if let Some(location) = request.location.clone() {
        let mut location_part = String::from("location=");
        location_part.push_str(location.as_str());
        query.push(location_part);
    }
    if let Some(attribute_list) = request.pool_attributes.clone() {
        let mut attribute_part = String::from("pool_attributes=");
        attribute_part.push_str(attribute_list.join(",").as_str());
    }
    if let Some(pool_name) = request.by_name.clone() {
        let mut pool_name_part = String::from("by_name=");
        pool_name_part.push_str(pool_name.as_str());
        query.push(pool_name_part);
    }
    query.join("&")
}

pub async fn test(url: String) -> Result<(), Box<dyn std::error::Error>> {
    // Parse our URL...
    let url = url.parse::<hyper::Uri>()?;
    dbg!(&url);

    // Get the host and the port
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address)
        .await
        .expect("unable to connect");

    // Use an adapter to access something implementing `tokio::io` traits as if they implement
    // `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .expect("create client failed");

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    // The authority of our URL will be the hostname of the httpbin remote
    let authority = url.authority().unwrap().clone();

    // Create an HTTP request with an empty body and a HOST header
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())
        .expect("failed to create request");

    // Await the response...
    let res = sender.send_request(req).await.expect("response error");

    println!("Response status: {}", res.status());
    Ok(())
}
