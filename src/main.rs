use hyper::server::conn::http1;
use hyper::service::service_fn;
use rust_web_server::handlers::handle_request;
use rust_web_server::auth::AuthManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth_manager = Arc::new(AuthManager::new());
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on http://{}", addr);
    println!("Default login - Username: admin, Password: admin123");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = hyper_util::rt::TokioIo::new(stream);
        let auth_manager = auth_manager.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let auth_manager = auth_manager.clone();
                    handle_request(req, auth_manager)
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}