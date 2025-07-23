use crate::bodies::StringBody;
use hyper::body::Body;
use hyper::{Response, StatusCode};
use std::env;

// Re-export the BoxBody type for other handler modules
pub type BoxBody = Box<dyn Body<Data = hyper::body::Bytes, Error = std::io::Error> + Send + Unpin>;

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
        .status(302)
        .header("Location", "/login")
        .body(Box::new(StringBody::new("".to_string())) as BoxBody)
        .unwrap()
}

pub fn html_response(html: String) -> Response<BoxBody> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new(html)) as BoxBody)
        .unwrap()
}

pub fn not_found() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new("404 Not Found".to_string())) as BoxBody)
        .unwrap())
}

// Authentication helper function
use crate::auth::AuthManager;
use hyper::{Request, body::Incoming};

pub fn is_authenticated(req: &Request<Incoming>, auth_manager: &AuthManager) -> bool {
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_id=") {
                    let session_id = &cookie[11..]; // Remove "session_id="
                    return auth_manager.validate_session(session_id);
                }
            }
        }
    }
    false
}