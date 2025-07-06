use crate::auth::AuthManager;
use crate::bodies::{BytesBody, StringBody};
use crate::file_browser::{generate_directory_html, get_directory_entries};
use crate::upload::handle_upload;
use crate::utils::collect_body_bytes;
use hyper::body::Body;
use hyper::{Request, Response, StatusCode, Method};
use hyper::body::Incoming;
use std::path::Path;
use std::fs;
use std::env;
use std::sync::Arc;

pub type BoxBody = Box<dyn Body<Data = hyper::body::Bytes, Error = std::io::Error> + Send + Unpin>;

// System files and folders to exclude
const SYSTEM_EXCLUSIONS: &[&str] = &[
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

fn is_system_file_or_folder(name: &str, is_hidden: bool) -> bool {
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

pub async fn handle_request(
    req: Request<Incoming>,
    auth_manager: Arc<AuthManager>,
) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let method = req.method().clone();
    let path = req.uri().path();
    
    // Check if user is authenticated (except for login routes)
    if path != "/login" && path != "/static" && !path.starts_with("/static/") {
        if !is_authenticated(&req, &auth_manager) {
            return Ok(redirect_to_login());
        }
    }
    
    match (method, path) {
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
        (Method::GET, "/upload") => upload_page(),
        (Method::POST, "/upload") => handle_upload_request(req).await,
        _ => not_found(),
    }
}

fn get_home_directory() -> String {
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

fn home_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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
            <h1>Web-based File Manager</h1>
            <p class="welcome-text">Your personal file server built with Rust</p>
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

fn url_decode(s: &str) -> String {
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

async fn browse_directory(path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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

async fn serve_file(path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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

fn upload_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Upload Files</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }
            .container { max-width: 600px; margin: 0 auto; background: rgba(255, 255, 255, 0.95); padding: 40px; border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); }
            h1 { color: #1f2937; text-align: center; margin-bottom: 30px; font-size: 32px; background: linear-gradient(135deg, #f97316, #2563eb); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
            .form-group { margin-bottom: 20px; }
            label { display: block; margin-bottom: 10px; font-weight: 600; color: #374151; font-size: 16px; }
            input[type="file"] { width: 100%; padding: 15px; border: 2px dashed #d1d5db; border-radius: 12px; background: rgba(255, 255, 255, 0.8); font-size: 14px; transition: all 0.3s ease; }
            input[type="file"]:hover { border-color: #f97316; background: rgba(249, 115, 22, 0.05); }
            input[type="file"]:focus { outline: none; border-color: #2563eb; background: rgba(37, 99, 235, 0.05); box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1); }
            .upload-btn { background: linear-gradient(135deg, #f97316, #ea580c); color: white; padding: 15px 30px; border: none; border-radius: 12px; cursor: pointer; font-size: 16px; font-weight: 600; width: 100%; margin-top: 10px; transition: all 0.3s ease; box-shadow: 0 4px 15px rgba(249, 115, 22, 0.3); }
            .upload-btn:hover { background: linear-gradient(135deg, #ea580c, #dc2626); transform: translateY(-2px); box-shadow: 0 8px 25px rgba(249, 115, 22, 0.4); }
            .upload-btn:active { transform: translateY(0); }
            .progress-container { margin-top: 20px; display: none; }
            .progress-bar { width: 100%; height: 24px; background: rgba(255, 255, 255, 0.8); border-radius: 12px; overflow: hidden; border: 1px solid rgba(255, 255, 255, 0.3); }
            .progress-fill { height: 100%; background: linear-gradient(135deg, #10b981, #059669); width: 0%; transition: width 0.3s ease; border-radius: 12px; position: relative; }
            .progress-fill::after { content: ''; position: absolute; top: 0; left: 0; right: 0; bottom: 0; background: linear-gradient(45deg, rgba(255,255,255,0.2) 25%, transparent 25%, transparent 50%, rgba(255,255,255,0.2) 50%, rgba(255,255,255,0.2) 75%, transparent 75%); background-size: 20px 20px; animation: progress-stripes 1s linear infinite; }
            @keyframes progress-stripes { 0% { background-position: 0 0; } 100% { background-position: 20px 0; } }
            .upload-status { margin-top: 15px; text-align: center; font-weight: 600; color: #374151; }
            .nav-footer { display: flex; justify-content: space-between; margin-top: 30px; padding-top: 20px; border-top: 1px solid rgba(255, 255, 255, 0.3); }
            .nav-link { color: #2563eb; text-decoration: none; font-size: 14px; font-weight: 500; padding: 8px 16px; border-radius: 8px; transition: all 0.2s ease; }
            .nav-link:hover { background: rgba(37, 99, 235, 0.1); text-decoration: underline; }
            .upload-zone { border: 2px dashed #d1d5db; border-radius: 12px; padding: 40px; text-align: center; background: rgba(255, 255, 255, 0.8); transition: all 0.3s ease; margin-bottom: 20px; }
            .upload-zone:hover { border-color: #f97316; background: rgba(249, 115, 22, 0.05); }
            .upload-zone.drag-over { border-color: #2563eb; background: rgba(37, 99, 235, 0.05); }
            .upload-icon { font-size: 48px; margin-bottom: 15px; color: #6b7280; }
            .upload-text { color: #6b7280; font-size: 16px; margin-bottom: 10px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>Upload Files</h1>
            
            <form id="uploadForm" method="post" enctype="multipart/form-data">
                <div class="upload-zone" id="uploadZone">
                    <div class="upload-icon">FILES</div>
                    <div class="upload-text">Drop files here or click to browse</div>
                    <input type="file" id="file" name="file" required multiple style="display: none;">
                </div>
                <div class="form-group">
                    <label for="file">Selected files will appear here</label>
                    <div id="fileList" style="color: #6b7280; font-size: 14px; min-height: 20px;">No files selected</div>
                </div>
                <button type="submit" class="upload-btn">Upload File(s)</button>
            </form>
            
            <div class="progress-container" id="progressContainer">
                <div class="progress-bar">
                    <div class="progress-fill" id="progressFill"></div>
                </div>
                <div class="upload-status" id="uploadStatus">Uploading...</div>
            </div>
            
            <div class="nav-footer">
                <a href="javascript:history.back()" class="nav-link">Back</a>
                <a href="/" class="nav-link">Home</a>
            </div>
        </div>
        
        <script>
            const uploadZone = document.getElementById('uploadZone');
            const fileInput = document.getElementById('file');
            const fileList = document.getElementById('fileList');
            
            uploadZone.addEventListener('click', () => fileInput.click());
            
            uploadZone.addEventListener('dragover', (e) => {
                e.preventDefault();
                uploadZone.classList.add('drag-over');
            });
            
            uploadZone.addEventListener('dragleave', () => {
                uploadZone.classList.remove('drag-over');
            });
            
            uploadZone.addEventListener('drop', (e) => {
                e.preventDefault();
                uploadZone.classList.remove('drag-over');
                fileInput.files = e.dataTransfer.files;
                updateFileList();
            });
            
            fileInput.addEventListener('change', updateFileList);
            
            function updateFileList() {
                const files = Array.from(fileInput.files);
                if (files.length > 0) {
                    fileList.innerHTML = files.map(file => `${file.name} (${formatFileSize(file.size)})`).join('<br>');
                } else {
                    fileList.innerHTML = 'No files selected';
                }
            }
            
            function formatFileSize(bytes) {
                if (bytes === 0) return '0 Bytes';
                const k = 1024;
                const sizes = ['Bytes', 'KB', 'MB', 'GB'];
                const i = Math.floor(Math.log(bytes) / Math.log(k));
                return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
            }
            
            document.getElementById('uploadForm').addEventListener('submit', function(e) {
                e.preventDefault();
                
                const formData = new FormData(this);
                const progressContainer = document.getElementById('progressContainer');
                const progressFill = document.getElementById('progressFill');
                const uploadStatus = document.getElementById('uploadStatus');
                
                progressContainer.style.display = 'block';
                progressFill.style.width = '0%';
                uploadStatus.textContent = 'Uploading...';
                
                const xhr = new XMLHttpRequest();
                
                xhr.upload.addEventListener('progress', function(e) {
                    if (e.lengthComputable) {
                        const percentComplete = (e.loaded / e.total) * 100;
                        progressFill.style.width = percentComplete + '%';
                        uploadStatus.textContent = `Uploading... ${Math.round(percentComplete)}%`;
                    }
                });
                
                xhr.onreadystatechange = function() {
                    if (xhr.readyState === 4) {
                        if (xhr.status === 200) {
                            progressFill.style.width = '100%';
                            uploadStatus.textContent = 'Upload completed successfully!';
                            uploadStatus.style.color = '#059669';
                            
                            setTimeout(() => {
                                document.getElementById('uploadForm').reset();
                                progressContainer.style.display = 'none';
                                uploadStatus.style.color = '#374151';
                                updateFileList();
                            }, 2000);
                        } else {
                            uploadStatus.textContent = 'Upload failed. Please try again.';
                            uploadStatus.style.color = '#dc2626';
                        }
                    }
                };
                
                xhr.open('POST', '/upload');
                xhr.send(formData);
            });
        </script>
    </body>
    </html>
    "#;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new(html.to_string())) as BoxBody)
        .unwrap())
}

async fn handle_upload_request(req: Request<Incoming>) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    match collect_body_bytes(req.into_body()).await {
        Ok(body_bytes) => {
            let response = handle_upload(body_bytes).await;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Box::new(StringBody::new(response)) as BoxBody)
                .unwrap())
        }
        Err(_) => {
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Box::new(StringBody::new("Error processing upload".to_string())) as BoxBody)
                .unwrap())
        }
    }
}

fn is_authenticated(req: &Request<Incoming>, auth_manager: &AuthManager) -> bool {
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

async fn handle_login(
    req: Request<Incoming>,
    auth_manager: Arc<AuthManager>,
) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let body = collect_body_bytes(req.into_body()).await?; // Use your existing function
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
            .header("Location", "/browse")
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

async fn handle_logout(
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

fn redirect_to_login() -> Response<BoxBody> {
    Response::builder()
        .status(302)
        .header("Location", "/login")
        .body(Box::new(StringBody::new("".to_string())) as BoxBody)
        .unwrap()
}

fn html_response(html: String) -> Response<BoxBody> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new(html)) as BoxBody)
        .unwrap()
}

fn not_found() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new("404 Not Found".to_string())) as BoxBody)
        .unwrap())
}