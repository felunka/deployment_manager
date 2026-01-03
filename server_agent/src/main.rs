use std::net::SocketAddr;
use std::env;
use std::fs;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod router;
mod util;
mod services;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr_string = match env::args().nth(1) {
        Some(addr) => addr,
        None => "127.0.0.1:8080".to_string(),
    };
    let key_file_path = match env::args().nth(2) {
        Some(file_path) => file_path,
        None => "/home/felunka/deployment_manager/server_agent/.key.hash".to_string(),
    };
    let key_hash = match fs::read_to_string(key_file_path) {
        Ok(key_hash) => key_hash,
        Err(e) => {
            println!("Key could not be read! Setting empty key...");
            println!("{}", e);
            String::new()
        },
    };

    let addr = addr_string.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let key_hash = key_hash.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| router::router(req, key_hash.clone())))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
