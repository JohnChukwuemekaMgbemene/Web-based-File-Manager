use crate::auth::AuthManager;
use hyper::{Request, Response, Method, StatusCode};
use std::sync::Arc;
use std::convert::Infallible;

mod utils;
mod auth_handlers;
mod file_handlers;
mod upload_handlers;

use utils::{
    BoxBody, 
    StringBody,
    redirect_to_login,
    html_response,
    not_found,
    is_authenticated,
};

use auth_handlers::{handle_login, handle_logout};
use file_handlers::{home_page, browse_directory, serve_file, serve_download};
use upload_handlers::{
    upload_page, 
    handle_upload_request, 
    start_resumable_upload, 
    upload_chunk, 
    complete_upload, 
    get_upload_status
};

// Helper function to create error responses
fn create_error_response(status: StatusCode, message: &str) -> Response<BoxBody> {
    Response::builder()
        .status(status)
        .body(Box::new(StringBody::new(message.to_string())) as BoxBody)
        .unwrap()
}

pub async fn handle_request(
    req: Request<hyper::body::Incoming>, 
    auth_manager: Arc<AuthManager>
) -> Result<Response<BoxBody>, Infallible> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start_time = std::time::Instant::now();
    
    // Check if user is authenticated (except for login routes)
    if uri.path() != "/login" && uri.path() != "/static" && !uri.path().starts_with("/static/") {
        if !is_authenticated(&req, &auth_manager) {
            return Ok(redirect_to_login());
        }
    }
    
    let result = match (method.clone(), uri.path()) {
        (Method::GET, "/login") => {
            html_response(crate::auth::generate_login_html())
        }
        (Method::POST, "/login") => {
            match handle_login(req, auth_manager).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Login error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                }
            }
        }
        (Method::GET, "/logout") => {
            match handle_logout(req, auth_manager).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Logout error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                }
            }
        }
        (Method::GET, "/") => {
            match home_page() {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Home page error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                }
            }
        }
        (Method::GET, path) if path.starts_with("/browse") => {
            match browse_directory(path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Browse error for {}: {:?}", path, e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Browse Error")
                }
            }
        }
        (Method::GET, path) if path.starts_with("/file") => {
            match serve_file(path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ File serve error for {}: {:?}", path, e);
                    create_error_response(StatusCode::NOT_FOUND, "File Not Found")
                }
            }
        }
        (Method::GET, path) if path.starts_with("/download") => {
            match serve_download(path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Download error for {}: {:?}", path, e);
                    create_error_response(StatusCode::NOT_FOUND, "File Not Found")
                }
            }
        }
        (Method::GET, "/upload") => {
            match upload_page() {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload page error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Upload Page Error")
                }
            }
        }
        (Method::POST, "/upload") => {
            match handle_upload_request(req).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Upload Failed")
                }
            }
        }
        (Method::POST, "/upload/start") => {
            match start_resumable_upload(req).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload start error: {:?}", e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Upload Start Failed")
                }
            }
        }
        (Method::POST, path) if path.starts_with("/upload/chunk/") => {
            match upload_chunk(req, path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload chunk error for {}: {:?}", path, e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Upload Chunk Failed")
                }
            }
        }
        (Method::POST, path) if path.starts_with("/upload/complete/") => {
            match complete_upload(req, path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload complete error for {}: {:?}", path, e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Upload Complete Failed")
                }
            }
        }
        (Method::GET, path) if path.starts_with("/upload/status/") => {
            match get_upload_status(req, path).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("❌ Upload status error for {}: {:?}", path, e);
                    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Status Check Failed")
                }
            }
        }
        _ => {
            match not_found() {
                Ok(response) => response,
                Err(_) => create_error_response(StatusCode::NOT_FOUND, "Not Found")
            }
        }
    };
    
    // Log request with emojis and timing
    let duration = start_time.elapsed();
    let status = result.status().as_u16();
    
    let emoji = match status {
        200..=299 => "✅",
        300..=399 => "🔄",
        400..=499 => "⚠️",
        500..=599 => "❌",
        _ => "❓",
    };
    
    println!("{} {} {} {} - {} ({:.1}ms)", 
        emoji,
        chrono::Utc::now().format("%H:%M:%S"),
        method, 
        uri.path(),
        status, 
        duration.as_secs_f64() * 1000.0
    );
    
    Ok(result)
}