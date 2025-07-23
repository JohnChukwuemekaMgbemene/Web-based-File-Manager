use hyper::server::conn::http1;
use hyper::service::service_fn;
use rust_web_server::handlers::handle_request;
use rust_web_server::auth::AuthManager;
use std::net::{SocketAddr, UdpSocket, IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::net::TcpListener;

// Function to get local IP address
fn get_local_ip() -> IpAddr {
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            match socket.connect("8.8.8.8:80") {
                Ok(_) => {
                    match socket.local_addr() {
                        Ok(addr) => addr.ip(),
                        Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    }
                },
                Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            }
        },
        Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth_manager = Arc::new(AuthManager::new());
    let port = 8000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let local_ip = get_local_ip();
    
    // Beautiful startup banner with emojis
    println!("\n🦀 Rust Web-based File Manager");
    println!("{}", "═".repeat(55));
    println!("🚀 Server Status: {} STARTING UP", "✅");
    println!();
    println!("📡 Network Information:");
    println!("   🏠 Local:      http://127.0.0.1:{}", port);
    println!("   🌐 Network:    http://{}:{}", local_ip, port);
    println!("   📱 LAN Access: http://{}:{}", local_ip, port);
    println!();
    println!("🔐 Authentication:");
    println!("   👤 Username: admin");
    println!("   🔑 Password: admin123");
    println!();
    println!("🎯 Quick Access:");
    println!("   📂 Browse Files:  /browse");
    println!("   📤 Upload Files:  /upload");
    println!("   🏠 Home Page:     /");
    println!();
    println!("📋 Features:");
    println!("   ✅ Secure file browsing");
    println!("   ✅ Multi-file uploads");
    println!("   ✅ System file protection");
    println!("   ✅ Cross-platform support");
    println!("   ✅ Mobile-friendly interface");
    println!();
    println!("🔧 Controls:");
    println!("   ⏹️  Stop Server: Ctrl+C");
    println!("   📊 View Logs:   Check terminal below");
    println!();
    println!("{}🎉 READY! Server is now accepting connections 🎉{}", "🎊 ", " 🎊");
    println!("{}", "═".repeat(55));
    println!();

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        
        // Log new connections with emojis
        println!("📱 {} New client connected: {}", 
            chrono::Utc::now().format("%H:%M:%S"), 
            client_addr
        );
        
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
                eprintln!("❌ {} Connection error from {}: {:?}", 
                    chrono::Utc::now().format("%H:%M:%S"),
                    client_addr, 
                    err
                );
            } else {
                println!("✅ {} Connection closed: {}", 
                    chrono::Utc::now().format("%H:%M:%S"),
                    client_addr
                );
            }
        });
    }
}