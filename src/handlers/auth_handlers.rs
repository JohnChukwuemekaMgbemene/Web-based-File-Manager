use crate::auth::AuthManager;
use crate::utils::collect_body_bytes;
use hyper::{Request, Response};
use hyper::body::Incoming;
use std::sync::Arc;

use super::utils::{BoxBody, html_response};
use crate::bodies::StringBody;

pub async fn handle_login(
    req: Request<Incoming>,
    auth_manager: Arc<AuthManager>,
) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let body = collect_body_bytes(req.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;
    
    // Parse form data
    let mut username = String::new();
    let mut password = String::new();
    
    for pair in body_str.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let decoded_value = urlencoding::decode(value)?.to_string();
            match key {
                "username" => username = decoded_value,
                "password" => password = decoded_value,
                _ => {}
            }
        }
    }
    
    if let Some(session_id) = auth_manager.authenticate(&username, &password) {
        let response = Response::builder()
            .status(302)
            .header("Location", "/")
            .header("Set-Cookie", format!("session_id={}; HttpOnly; Path=/", session_id))
            .body(Box::new(StringBody::new("".to_string())) as BoxBody)?;
        
        Ok(response)
    } else {
        // Return login page with error
        let error_html = crate::auth::generate_login_html().replace(
            "</form>",
            r#"</form>
            <div class="error">Invalid username or password</div>"#
        );
        Ok(html_response(error_html))
    }
}

pub async fn handle_logout(
    req: Request<Incoming>,
    auth_manager: Arc<AuthManager>,
) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    // Extract session ID and logout
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_id=") {
                    let session_id = &cookie[11..];
                    auth_manager.logout(session_id);
                    break;
                }
            }
        }
    }
    
    let response = Response::builder()
        .status(302)
        .header("Location", "/login")
        .header("Set-Cookie", "session_id=; HttpOnly; Path=/; Max-Age=0")
        .body(Box::new(StringBody::new("".to_string())) as BoxBody)?;
    
    Ok(response)
}