use crate::bodies::{BytesBody, StringBody};
use crate::file_browser::{generate_directory_html, get_directory_entries};
use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use std::path::Path;
use std::fs;

use super::utils::{BoxBody, is_system_file_or_folder, get_home_directory, url_decode};

pub fn home_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Rust Web Server</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }
            .container { max-width: 800px; margin: 0 auto; background: rgba(255, 255, 255, 0.95); padding: 40px; border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); }
            h1 { color: #1f2937; text-align: center; font-size: 36px; margin-bottom: 20px; background: linear-gradient(135deg, #f97316, #2563eb); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
            .welcome-text { text-align: center; color: #6b7280; font-size: 18px; margin-bottom: 40px; }
            .nav { text-align: center; margin-top: 30px; }
            .nav a { display: inline-block; margin: 0 20px; padding: 15px 30px; color: white; text-decoration: none; border-radius: 12px; font-size: 16px; font-weight: 600; transition: all 0.3s ease; box-shadow: 0 4px 15px rgba(0,0,0,0.1); position: relative; overflow: hidden; }
            .nav a:first-child { background: linear-gradient(135deg, #2563eb, #1d4ed8); }
            .nav a:first-child:hover { background: linear-gradient(135deg, #1d4ed8, #1e40af); transform: translateY(-2px); box-shadow: 0 8px 25px rgba(37, 99, 235, 0.3); }
            .nav a:last-child { background: linear-gradient(135deg, #f97316, #ea580c); }
            .nav a:last-child:hover { background: linear-gradient(135deg, #ea580c, #dc2626); transform: translateY(-2px); box-shadow: 0 8px 25px rgba(249, 115, 22, 0.3); }
            .nav a::before { content: ''; position: absolute; top: 0; left: -100%; width: 100%; height: 100%; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent); transition: left 0.5s; }
            .nav a:hover::before { left: 100%; }
            .features { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-top: 40px; }
            .feature { background: rgba(138, 158, 250, 0.6); padding: 20px; border-radius: 12px; text-align: center; border: 1px solid rgba(255, 255, 255, 0.3); }
            .feature h3 { color: #1f2937; margin-bottom: 10px; }
            .feature p { color: #6b7280; font-size: 14px; }
            .feature-icon { font-size: 32px; margin-bottom: 15px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>Web-based File Browser</h1>
            <p class="welcome-text">Your personal file browser built for easy file sharing and transfer</p>
            <div class="nav">
                <a href="/browse">Browse Files</a>
                <a href="/upload">Upload Files</a>
            </div>
            <div class="features">
                <div class="feature">
                    <div class="feature-icon">FAST</div>
                    <h3>Fast & Efficient</h3>
                    <p>Built with Rust for maximum performance and safety</p>
                </div>
                <div class="feature">
                    <div class="feature-icon">SECURE</div>
                    <h3>Secure</h3>
                    <p>System files are automatically filtered and protected</p>
                </div>
                <div class="feature">
                    <div class="feature-icon">WEB</div>
                    <h3>Web-Based</h3>
                    <p>Access your files from any device with a web browser</p>
                </div>
            </div>
        </div>
    </body>
    </html>
    "#;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new(html.to_string())) as BoxBody)
        .unwrap())
}

pub async fn browse_directory(path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let home_dir = get_home_directory();
    let fs_path = path.strip_prefix("/browse").unwrap_or("/");
    
    let fs_path = if fs_path.is_empty() || fs_path == "/" {
        // Start from home directory
        home_dir.clone()
    } else {
        // Decode the URL-encoded path and construct path relative to home directory
        let relative_path = &fs_path[1..]; // Remove leading slash
        let decoded_relative_path = url_decode(relative_path);
        format!("{}/{}", home_dir, decoded_relative_path.replace("/", "\\"))
    };
    
    let dir_path = Path::new(&fs_path);
    
    if dir_path.exists() && dir_path.is_dir() {
        match get_directory_entries(dir_path) {
            Ok(entries) => {
                // Filter out system files and folders
                let filtered_entries: Vec<_> = entries.into_iter()
                    .filter(|entry| {
                        // Check if file has hidden attribute on Windows
                        #[cfg(windows)]
                        let is_hidden = {
                            use std::os::windows::fs::MetadataExt;
                            if let Ok(metadata) = entry.path.metadata() {
                                const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
                                const FILE_ATTRIBUTE_SYSTEM: u32 = 0x04;
                                let attrs = metadata.file_attributes();
                                (attrs & FILE_ATTRIBUTE_HIDDEN) != 0 || (attrs & FILE_ATTRIBUTE_SYSTEM) != 0
                            } else {
                                false
                            }
                        };
                        
                        #[cfg(not(windows))]
                        let is_hidden = false;
                        
                        !is_system_file_or_folder(&entry.name, is_hidden)
                    })
                    .collect();
                
                let html = generate_directory_html(&filtered_entries, path);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Box::new(StringBody::new(html)) as BoxBody)
                    .unwrap())
            }
            Err(_) => {
                Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Box::new(StringBody::new("Access denied".to_string())) as BoxBody)
                    .unwrap())
            }
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Box::new(StringBody::new("Directory not found".to_string())) as BoxBody)
            .unwrap())
    }
}

pub async fn serve_file(path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let home_dir = get_home_directory();
    let fs_path = path.strip_prefix("/file").unwrap_or("/");
    let fs_path = if fs_path.starts_with('/') { &fs_path[1..] } else { fs_path };
    
    // Check if download parameter is present
    let (fs_path, force_download) = if let Some(pos) = fs_path.find('?') {
        let query = &fs_path[pos+1..];
        let path_only = &fs_path[..pos];
        (path_only, query.contains("download=true"))
    } else {
        (fs_path, false)
    };
    
    // Decode the URL-encoded path
    let decoded_fs_path = url_decode(fs_path);
    
    let full_path = format!("{}/{}", home_dir, decoded_fs_path.replace("/", "\\"));
    let file_path = Path::new(&full_path);
    
    // Check if trying to access system file
    if let Some(file_name) = file_path.file_name() {
        if let Some(name_str) = file_name.to_str() {
            #[cfg(windows)]
            let is_hidden = {
                use std::os::windows::fs::MetadataExt;
                if let Ok(metadata) = file_path.metadata() {
                    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
                    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x04;
                    let attrs = metadata.file_attributes();
                    (attrs & FILE_ATTRIBUTE_HIDDEN) != 0 || (attrs & FILE_ATTRIBUTE_SYSTEM) != 0
                } else {
                    false
                }
            };
            
            #[cfg(not(windows))]
            let is_hidden = false;
            
            if is_system_file_or_folder(name_str, is_hidden) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Box::new(StringBody::new("Access to system files is not allowed".to_string())) as BoxBody)
                    .unwrap());
            }
        }
    }
    
    if file_path.exists() && file_path.is_file() {
        let contents = fs::read(file_path)?;
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream();
        
        let mut response_builder = Response::builder()
            .header("Content-Type", mime_type.as_ref());
        
        // Add download headers if requested
        if force_download {
            if let Some(filename) = file_path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    response_builder = response_builder
                        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename_str));
                }
            }
        }
        
        Ok(response_builder
            .body(Box::new(BytesBody::new(contents)) as BoxBody)
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Box::new(StringBody::new("File not found".to_string())) as BoxBody)
            .unwrap())
    }
}

pub async fn serve_download(path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let home_dir = get_home_directory();
    let fs_path = path.strip_prefix("/download").unwrap_or("/");
    let fs_path = if fs_path.starts_with('/') { &fs_path[1..] } else { fs_path };
    
    // Decode the URL-encoded path
    let decoded_fs_path = url_decode(fs_path);
    
    let full_path = format!("{}/{}", home_dir, decoded_fs_path.replace("/", "\\"));
    let file_path = Path::new(&full_path);
    
    // Check if trying to access system file
    if let Some(file_name) = file_path.file_name() {
        if let Some(name_str) = file_name.to_str() {
            #[cfg(windows)]
            let is_hidden = {
                use std::os::windows::fs::MetadataExt;
                if let Ok(metadata) = file_path.metadata() {
                    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;
                    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x04;
                    let attrs = metadata.file_attributes();
                    (attrs & FILE_ATTRIBUTE_HIDDEN) != 0 || (attrs & FILE_ATTRIBUTE_SYSTEM) != 0
                } else {
                    false
                }
            };
            
            #[cfg(not(windows))]
            let is_hidden = false;
            
            if is_system_file_or_folder(name_str, is_hidden) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Box::new(StringBody::new("Access to system files is not allowed".to_string())) as BoxBody)
                    .unwrap());
            }
        }
    }
    
    if file_path.exists() && file_path.is_file() {
        let contents = fs::read(file_path)?;
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream();
        
        let mut response_builder = Response::builder()
            .header("Content-Type", mime_type.as_ref());
        
        // Always add download headers for this route
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                response_builder = response_builder
                    .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename_str));
            }
        }
        
        Ok(response_builder
            .body(Box::new(BytesBody::new(contents)) as BoxBody)
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Box::new(StringBody::new("File not found".to_string())) as BoxBody)
            .unwrap())
    }
}