use http_body_util::Empty;
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

struct httpclient {

}

pub fn test () -> Result{
    // Parse our URL...
    let url = "http://localhost:3000/request".parse::<hyper::Uri>()?;

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
