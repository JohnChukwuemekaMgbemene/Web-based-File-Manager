use hyper::{Response, StatusCode, header};
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper::body::{Body, Bytes, Frame};
use std::io::Error as IoError;

// Re-export the BoxBody type for other handler modules
pub type BoxBody = Box<dyn Body<Data = Bytes, Error = IoError> + Send + Unpin>;

pub struct StringBody {
    data: Option<Bytes>,
}

impl StringBody {
    pub fn new(data: String) -> Self {
        Self {
            data: Some(Bytes::from(data)),
        }
    }
}

impl Body for StringBody {
    type Data = Bytes;
    type Error = IoError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if let Some(data) = self.data.take() {
            Poll::Ready(Some(Ok(Frame::data(data))))
        } else {
            Poll::Ready(None)
        }
    }
}

// System files and folders to exclude
pub const SYSTEM_EXCLUSIONS: &[&str] = &[
    // Windows system directories
    "System Volume Information",
    "$Recycle.Bin",
    "$RECYCLE.BIN",
    "Config.Msi",
    "PerfLogs",
    "Recovery",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
    "AppData",
    "Users/All Users",
    "Users/Default",
    "Users/Default User",
    "Users/Public",
    
    // Hidden/system files
    "pagefile.sys",
    "hiberfil.sys",
    "swapfile.sys",
    "bootmgr",
    "BOOTNXT",
    "BOOTSECT.BAK",
    "ntldr",
    "NTDETECT.COM",
    "boot.ini",
    "Desktop.ini",
    
    // Mac system files
    ".DS_Store",
    ".Trash",
    ".fseventsd",
    ".Spotlight-V100",
    ".TemporaryItems",
    
    // Linux system files
    ".bash_history",
    ".bashrc",
    ".profile",
    
    // Common system/hidden patterns
    "desktop.ini",
    "thumbs.db",
    "Thumbs.db",
    ".tmp",
    ".temp",
];

pub fn is_system_file_or_folder(name: &str, is_hidden: bool) -> bool {
    // Check if it's a hidden file/folder (starts with .)
    if is_hidden || name.starts_with('.') {
        return true;
    }
    
    // Check against system exclusions list
    for exclusion in SYSTEM_EXCLUSIONS {
        if name.eq_ignore_ascii_case(exclusion) {
            return true;
        }
    }
    
    // Check for temporary files
    if name.ends_with(".tmp") || name.ends_with(".temp") || name.ends_with("~") {
        return true;
    }
    
    // Check for .ink files (Windows shortcuts)
    if name.ends_with(".lnk") {
        return true;
    }
    
    false
}

pub fn get_home_directory() -> String {
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

pub fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex1 = chars.next().unwrap_or('0');
            let hex2 = chars.next().unwrap_or('0');
            let hex_str = format!("{}{}", hex1, hex2);
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                result.push(byte as char);
            } else {
                result.push(ch);
                result.push(hex1);
                result.push(hex2);
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

pub fn redirect_to_login() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/login")
        .body(Box::new(StringBody::new(String::new())) as BoxBody)
        .unwrap()
}

pub fn html_response(html: String) -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Box::new(StringBody::new(html)) as BoxBody)
        .unwrap()
}

pub fn not_found() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>404 - Not Found</title>
        <meta charset="UTF-8">
        <style>
            body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; margin: 0; }
            .container { background: white; padding: 40px; border-radius: 12px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); max-width: 500px; margin: 0 auto; }
            h1 { color: #ef4444; font-size: 48px; margin: 0; }
            p { color: #6b7280; font-size: 18px; margin: 20px 0; }
            a { color: #2563eb; text-decoration: none; font-weight: 600; }
            a:hover { text-decoration: underline; }
            .emoji { font-size: 64px; margin-bottom: 20px; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="emoji">🔍</div>
            <h1>404</h1>
            <p>The page you're looking for could not be found.</p>
            <p><a href="/">← Go back home</a></p>
        </div>
    </body>
    </html>
    "#;
    
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Box::new(StringBody::new(html.to_string())) as BoxBody)?)
}

// Authentication helper function
use crate::auth::AuthManager;
use hyper::Request;
use std::sync::Arc;

pub fn is_authenticated(req: &Request<hyper::body::Incoming>, auth_manager: &Arc<AuthManager>) -> bool {
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("session_id=") {
                    return auth_manager.is_valid_token(token);
                }
            }
        }
    }
    false
}