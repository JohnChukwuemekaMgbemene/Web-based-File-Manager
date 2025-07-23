use crate::auth::AuthManager;
use hyper::{Request, Response, Method};
use hyper::body::Incoming;
use std::sync::Arc;

mod utils;
mod auth_handlers;
mod file_handlers;
mod upload_handlers;

use utils::{
    BoxBody, 
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

pub async fn handle_request(
    req: Request<Incoming>,
    auth_manager: Arc<AuthManager>,
) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    
    // Check if user is authenticated (except for login routes)
    if path != "/login" && path != "/static" && !path.starts_with("/static/") {
        if !is_authenticated(&req, &auth_manager) {
            return Ok(redirect_to_login());
        }
    }
    
    match (method, path.as_str()) {
        (Method::GET, "/login") => {
            Ok(html_response(crate::auth::generate_login_html()))
        }
        (Method::POST, "/login") => {
            handle_login(req, auth_manager).await
        }
        (Method::GET, "/logout") => {
            handle_logout(req, auth_manager).await
        }
        (Method::GET, "/") => home_page(),
        (Method::GET, path) if path.starts_with("/browse") => browse_directory(path).await,
        (Method::GET, path) if path.starts_with("/file") => serve_file(path).await,
        (Method::GET, path) if path.starts_with("/download") => serve_download(path).await,
        (Method::GET, "/upload") => upload_page(),
        (Method::POST, "/upload") => handle_upload_request(req).await,
        (Method::POST, "/upload/start") => start_resumable_upload(req).await,
        (Method::POST, path) if path.starts_with("/upload/chunk/") => upload_chunk(req, path).await,
        (Method::POST, path) if path.starts_with("/upload/complete/") => complete_upload(req, path).await,
        (Method::GET, path) if path.starts_with("/upload/status/") => get_upload_status(req, path).await,
        _ => not_found(),
    }
}