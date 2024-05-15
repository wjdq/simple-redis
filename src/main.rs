use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, warn};

use simple_redis::backend::Backend;
use simple_redis::network;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6378";

    info!("simple-redis-server is listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    let backend = Backend::new();
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        info!("accepted connection from {}", remote_addr);
        let backend_cloned = backend.clone();
        tokio::spawn(async move {
            match network::stream_handler(stream, backend_cloned).await {
                Ok(_) => info!("connection closed for {}", remote_addr),
                Err(e) => warn!("handle connection error for {} : {}", remote_addr, e),
            }
        });
    }
}
