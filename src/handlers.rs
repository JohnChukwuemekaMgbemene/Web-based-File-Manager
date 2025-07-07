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
use serde_json;
use std::sync::Arc;
use crate::resumable_upload::UPLOAD_MANAGER;

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
    let path = req.uri().path().to_string(); // Convert to owned String
    
    // Check if user is authenticated (except for login routes)
    if path != "/login" && path != "/static" && !path.starts_with("/static/") {
        if !is_authenticated(&req, &auth_manager) {
            return Ok(redirect_to_login());
        }
    }
    
    match (method, path.as_str()) { // Use as_str() to get &str
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
        (Method::POST, "/upload/start") => start_resumable_upload(req).await,
        (Method::POST, path) if path.starts_with("/upload/chunk/") => upload_chunk(req, path).await,
        (Method::POST, path) if path.starts_with("/upload/complete/") => complete_upload(req, path).await,
        (Method::GET, path) if path.starts_with("/upload/status/") => get_upload_status(req, path).await,
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

// Update the upload_page function to use JavaScript history.back()
fn upload_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Upload Files - File Manager</title>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }
        .container { max-width: 800px; margin: 0 auto; background: rgba(255, 255, 255, 0.95); padding: 40px; border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); }
        h1 { color: #1f2937; text-align: center; margin-bottom: 30px; font-size: 32px; background: linear-gradient(135deg, #f97316, #2563eb); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
        .upload-area { border: 2px dashed #d1d5db; border-radius: 12px; padding: 40px; text-align: center; background: rgba(255, 255, 255, 0.8); transition: all 0.3s ease; margin-bottom: 20px; }
        .upload-area:hover { border-color: #f97316; background: rgba(249, 115, 22, 0.05); }
        .file-list { margin: 20px 0; }
        .file-item { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 8px; border: 1px solid #e9ecef; display: flex; justify-content: space-between; align-items: center; }
        .file-info { flex: 1; }
        .file-name { font-weight: 600; color: #495057; }
        .file-size { color: #6c757d; font-size: 14px; }
        .file-actions { display: flex; gap: 10px; }
        .remove-btn { background: #dc3545; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; }
        .upload-controls { text-align: center; margin: 20px 0; }
        .upload-btn { background: #28a745; color: white; border: none; padding: 20px 60px; border-radius: 12px; cursor: pointer; font-size: 18px; font-weight: 600; transition: all 0.3s ease; box-shadow: 0 4px 15px rgba(40, 167, 69, 0.3); }
        .upload-btn:hover { background: #218838; transform: translateY(-2px); box-shadow: 0 6px 20px rgba(40, 167, 69, 0.4); }
        .upload-btn:disabled { background: #6c757d; cursor: not-allowed; transform: none; box-shadow: none; }
        .upload-progress { width: 100%; height: 20px; background-color: #f0f0f0; border-radius: 10px; overflow: hidden; margin: 10px 0; }
        .progress-bar { height: 100%; background: linear-gradient(90deg, #4caf50, #45a049); transition: width 0.3s ease; }
        .upload-speed { font-size: 14px; color: #666; margin: 5px 0; }
        .upload-item { border: 1px solid #ddd; border-radius: 8px; padding: 15px; margin: 10px 0; background: #f9f9f9; }
        .pause-resume-btn { background: #ff9800; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; margin: 0 5px; }
        .cancel-btn { background: #f44336; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; }
        .navigation { display: flex; justify-content: space-between; align-items: center; margin-top: 30px; }
        .nav-button { background: #2563eb; color: white; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer; font-size: 16px; font-weight: 600; transition: all 0.3s ease; text-decoration: none; display: inline-flex; align-items: center; gap: 8px; }
        .nav-button:hover { background: #1d4ed8; transform: translateY(-2px); box-shadow: 0 4px 15px rgba(37, 99, 235, 0.3); }
        .nav-button.home { background: #f97316; }
        .nav-button.home:hover { background: #ea580c; box-shadow: 0 4px 15px rgba(249, 115, 22, 0.3); }
        .empty-state { text-align: center; color: #6c757d; font-style: italic; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="container">
        <h1>📁 Upload Files</h1>
        
        <div class="upload-area" id="uploadArea">
            <p>Drag & drop files here or click to select</p>
            <input type="file" id="fileInput" multiple style="display: none;">
        </div>
        
        <div class="file-list" id="fileList">
            <div class="empty-state" id="emptyState">No files selected</div>
        </div>
        
        <div class="upload-controls">
            <button class="upload-btn" id="uploadBtn" disabled onclick="startUpload()">
                📤 Upload Files
            </button>
        </div>
        
        <div id="uploadList"></div>
        
        <div class="navigation">
            <button class="nav-button" onclick="goBack()">← Back</button>
            <a href="/" class="nav-button home">🏠 Home</a>
        </div>
    </div>

    <script>
        // Add the goBack function
        function goBack() {
            if (window.history.length > 1) {
                window.history.back();
            } else {
                window.location.href = '/browse';
            }
        }

        let selectedFiles = [];
        let uploader = null;
        
        class ResumableUploader {
            constructor() {
                this.uploads = new Map();
                this.chunkSize = 1024 * 1024; // 1MB chunks
                this.maxConcurrent = 3;
                this.activeUploads = 0;
            }
            
            async uploadFile(file) {
                if (this.activeUploads >= this.maxConcurrent) {
                    setTimeout(() => this.uploadFile(file), 1000);
                    return;
                }
                
                this.activeUploads++;
                const uploadId = Date.now() + '-' + Math.random().toString(36).substr(2, 9);
                
                const upload = {
                    id: uploadId,
                    file: file,
                    sessionId: null,
                    totalSize: file.size,
                    uploadedSize: 0,
                    chunkSize: this.chunkSize,
                    paused: false,
                    cancelled: false,
                    startTime: Date.now(),
                    element: this.createUploadElement(file.name, uploadId)
                };
                
                this.uploads.set(uploadId, upload);
                
                try {
                    // Start upload session
                    console.log('Starting upload session for:', file.name);
                    const sessionResponse = await fetch('/upload/start', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            filename: file.name,
                            totalSize: file.size
                        })
                    });
                    
                    console.log('Session response status:', sessionResponse.status);
                    
                    if (!sessionResponse.ok) {
                        const errorText = await sessionResponse.text();
                        console.error('Session response error:', errorText);
                        throw new Error(`Failed to start session: ${sessionResponse.status} - ${errorText}`);
                    }
                    
                    const session = await sessionResponse.json();
                    console.log('Session started:', session);
                    
                    upload.sessionId = session.sessionId;
                    upload.element.querySelector('.upload-status').textContent = 'Uploading...';
                    
                    // Start uploading chunks
                    await this.uploadChunks(upload);
                    
                } catch (error) {
                    console.error('Upload failed:', error);
                    upload.element.querySelector('.upload-status').textContent = 'Failed: ' + error.message;
                    upload.element.style.backgroundColor = '#ffe8e8';
                    this.activeUploads--;
                }
            }
            
            async uploadChunks(upload) {
                console.log('Starting chunk upload process');
                
                while (upload.uploadedSize < upload.totalSize && !upload.cancelled) {
                    if (upload.paused) {
                        await new Promise(resolve => setTimeout(resolve, 100));
                        continue;
                    }
                    
                    const start = upload.uploadedSize;
                    const end = Math.min(start + upload.chunkSize, upload.totalSize);
                    const chunk = upload.file.slice(start, end);
                    
                    try {
                        console.log(`Uploading chunk ${start}-${end-1} of ${upload.totalSize}`);
                        
                        const response = await fetch(`/upload/chunk/${upload.sessionId}`, {
                            method: 'POST',
                            headers: {
                                'Content-Range': `bytes ${start}-${end-1}/${upload.totalSize}`,
                                'Content-Type': 'application/octet-stream'
                            },
                            body: chunk
                        });
                        
                        console.log('Chunk response status:', response.status);
                        
                        if (!response.ok) {
                            const errorText = await response.text();
                            console.error('Chunk response error:', errorText);
                            throw new Error(`Chunk upload failed: ${response.status} - ${errorText}`);
                        }
                        
                        const result = await response.json();
                        console.log('Chunk upload result:', result);
                        
                        upload.uploadedSize = result.uploadedSize;
                        
                        this.updateProgress(upload);
                        
                    } catch (error) {
                        console.error('Chunk upload error:', error);
                        upload.element.querySelector('.upload-status').textContent = 'Error: ' + error.message;
                        upload.element.style.backgroundColor = '#ffe8e8';
                        this.activeUploads--;
                        return;
                    }
                }
                
                if (upload.uploadedSize >= upload.totalSize && !upload.cancelled) {
                    console.log('All chunks uploaded, completing upload');
                    await this.completeUpload(upload);
                }
                
                this.activeUploads--;
            }
            
            async completeUpload(upload) {
                try {
                    console.log('Completing upload for:', upload.file.name);
                    const response = await fetch(`/upload/complete/${upload.sessionId}`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            filename: upload.file.name,
                            finalPath: `/uploads/${upload.file.name}`
                        })
                    });
                    
                    console.log('Complete response status:', response.status);
                    
                    if (response.ok) {
                        upload.element.querySelector('.upload-status').textContent = 'Completed ✓';
                        upload.element.style.backgroundColor = '#e8f5e8';
                        console.log('Upload completed successfully');
                    } else {
                        const errorText = await response.text();
                        console.error('Complete response error:', errorText);
                        throw new Error(`Complete upload failed: ${response.status} - ${errorText}`);
                    }
                } catch (error) {
                    console.error('Complete upload error:', error);
                    upload.element.querySelector('.upload-status').textContent = 'Completion failed: ' + error.message;
                    upload.element.style.backgroundColor = '#ffe8e8';
                }
            }
            
            createUploadElement(filename, uploadId) {
                const element = document.createElement('div');
                element.className = 'upload-item';
                element.innerHTML = `
                    <div class="upload-filename">${filename}</div>
                    <div class="upload-progress">
                        <div class="progress-bar" style="width: 0%"></div>
                    </div>
                    <div class="upload-speed">Starting...</div>
                    <div class="upload-status">Preparing...</div>
                    <div class="upload-actions">
                        <button class="pause-resume-btn" onclick="uploader.togglePause('${uploadId}')">Pause</button>
                        <button class="cancel-btn" onclick="uploader.cancelUpload('${uploadId}')">Cancel</button>
                    </div>
                `;
                
                document.getElementById('uploadList').appendChild(element);
                return element;
            }
            
            updateProgress(upload) {
                const progress = (upload.uploadedSize / upload.totalSize) * 100;
                const progressBar = upload.element.querySelector('.progress-bar');
                const speedElement = upload.element.querySelector('.upload-speed');
                
                progressBar.style.width = progress + '%';
                
                // Calculate speed
                const elapsed = (Date.now() - upload.startTime) / 1000;
                const speed = upload.uploadedSize / elapsed;
                const speedText = this.formatSpeed(speed);
                
                if (upload.uploadedSize < upload.totalSize) {
                    const eta = this.formatTime((upload.totalSize - upload.uploadedSize) / speed);
                    speedElement.textContent = `${speedText} - ${eta} remaining`;
                } else {
                    speedElement.textContent = `${speedText} - Complete`;
                }
            }
            
            formatSpeed(bytesPerSecond) {
                if (isNaN(bytesPerSecond) || bytesPerSecond === 0) return '0 B/s';
                if (bytesPerSecond < 1024) return bytesPerSecond.toFixed(0) + ' B/s';
                if (bytesPerSecond < 1024 * 1024) return (bytesPerSecond / 1024).toFixed(1) + ' KB/s';
                return (bytesPerSecond / (1024 * 1024)).toFixed(1) + ' MB/s';
            }
            
            formatTime(seconds) {
                if (isNaN(seconds) || seconds === Infinity) return '∞';
                if (seconds < 60) return Math.round(seconds) + 's';
                if (seconds < 3600) return Math.round(seconds / 60) + 'm';
                return Math.round(seconds / 3600) + 'h';
            }
            
            togglePause(uploadId) {
                const upload = this.uploads.get(uploadId);
                if (upload) {
                    upload.paused = !upload.paused;
                    const btn = upload.element.querySelector('.pause-resume-btn');
                    btn.textContent = upload.paused ? 'Resume' : 'Pause';
                }
            }
            
            cancelUpload(uploadId) {
                const upload = this.uploads.get(uploadId);
                if (upload) {
                    upload.cancelled = true;
                    upload.element.style.backgroundColor = '#ffe8e8';
                    upload.element.querySelector('.upload-status').textContent = 'Cancelled';
                    this.activeUploads--;
                }
            }
        }
        
        function addFiles(files) {
            Array.from(files).forEach(file => {
                if (!selectedFiles.find(f => f.name === file.name && f.size === file.size)) {
                    selectedFiles.push(file);
                }
            });
            updateFileList();
        }
        
        function removeFile(index) {
            selectedFiles.splice(index, 1);
            updateFileList();
        }
        
        function updateFileList() {
            const fileList = document.getElementById('fileList');
            const emptyState = document.getElementById('emptyState');
            const uploadBtn = document.getElementById('uploadBtn');
            
            if (selectedFiles.length === 0) {
                fileList.innerHTML = '<div class="empty-state">No files selected</div>';
                uploadBtn.disabled = true;
            } else {
                fileList.innerHTML = selectedFiles.map((file, index) => `
                    <div class="file-item">
                        <div class="file-info">
                            <div class="file-name">${file.name}</div>
                            <div class="file-size">${formatFileSize(file.size)}</div>
                        </div>
                        <div class="file-actions">
                            <button class="remove-btn" onclick="removeFile(${index})">Remove</button>
                        </div>
                    </div>
                `).join('');
                uploadBtn.disabled = false;
            }
        }
        
        function formatFileSize(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
        
        function startUpload() {
            if (selectedFiles.length === 0) return;
            
            uploader = new ResumableUploader();
            document.getElementById('uploadBtn').disabled = true;
            
            selectedFiles.forEach(file => {
                console.log('Starting upload for:', file.name, file.size, 'bytes');
                uploader.uploadFile(file);
            });
        }
        
        // File input handling
        const fileInput = document.getElementById('fileInput');
        const uploadArea = document.getElementById('uploadArea');
        
        uploadArea.addEventListener('click', () => fileInput.click());
        fileInput.addEventListener('change', (e) => {
            addFiles(e.target.files);
        });
        
        // Drag & drop
        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.style.backgroundColor = '#e8f4f8';
        });
        
        uploadArea.addEventListener('dragleave', () => {
            uploadArea.style.backgroundColor = '#f8f9fa';
        });
        
        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.style.backgroundColor = '#f8f9fa';
            addFiles(e.dataTransfer.files);
        });
        
        // Initialize
        updateFileList();
    </script>
</body>
</html>
    "#;

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Box::new(StringBody::new(html.to_string())) as BoxBody)?)
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

async fn start_resumable_upload(req: Request<Incoming>) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let body = collect_body_bytes(req.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;
    
    // Parse JSON request
    let upload_request: serde_json::Value = serde_json::from_str(&body_str)?;
    let filename = upload_request["filename"].as_str().unwrap();
    let total_size = upload_request["totalSize"].as_u64().unwrap();
    
    // Create upload session using global manager
    let session_id = UPLOAD_MANAGER.create_session(filename, total_size);
    
    let response = serde_json::json!({
        "sessionId": session_id,
        "chunkSize": 1024 * 1024, // 1MB chunks
        "uploadedSize": 0
    });
    
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
}

async fn upload_chunk(req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id = path.strip_prefix("/upload/chunk/").unwrap();
    
    // Get chunk offset from headers
    let offset = req.headers()
        .get("Content-Range")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            // Parse "bytes 0-1023/1024" format
            if let Some(bytes_part) = s.strip_prefix("bytes ") {
                if let Some(range_part) = bytes_part.split('/').next() {
                    if let Some(start_str) = range_part.split('-').next() {
                        return start_str.parse::<u64>().ok();
                    }
                }
            }
            None
        })
        .unwrap_or(0);
    
    let body = collect_body_bytes(req.into_body()).await?;
    
    // Use global upload manager
    match UPLOAD_MANAGER.upload_chunk(session_id, &body, offset) {
        Ok(uploaded_size) => {
            let response = serde_json::json!({
                "uploadedSize": uploaded_size,
                "status": "success"
            });
            
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": e,
                "status": "error"
            });
            
            Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
    }
}

// Update the complete_upload function to save to Desktop
async fn complete_upload(req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id = path.strip_prefix("/upload/complete/").unwrap();
    
    let body = collect_body_bytes(req.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;
    
    // Parse JSON request
    let complete_request: serde_json::Value = serde_json::from_str(&body_str)?;
    let filename = complete_request["filename"].as_str().unwrap();
    let _final_path = complete_request["finalPath"].as_str().unwrap();
    
    // Get home directory and construct Desktop path
    let home_dir = get_home_directory();
    let final_path = format!("{}\\Desktop\\{}", home_dir, filename);
    
    // Ensure Desktop directory exists (it should already exist)
    if let Some(parent) = Path::new(&final_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Use global upload manager
    match UPLOAD_MANAGER.complete_upload(session_id, &final_path) {
        Ok(()) => {
            let response = serde_json::json!({
                "status": "success",
                "message": "Upload completed successfully",
                "finalPath": final_path
            });
            
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
        Err(e) => {
            let response = serde_json::json!({
                "error": e,
                "status": "error"
            });
            
            Ok(Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
    }
}

async fn get_upload_status(_req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id = path.strip_prefix("/upload/status/").unwrap();
    
    // Use global upload manager
    match UPLOAD_MANAGER.get_session(session_id) {
        Some(session) => {
            let response = serde_json::json!({
                "status": "success",
                "sessionId": session.session_id,
                "totalSize": session.total_size,
                "uploadedSize": session.uploaded_size,
                "progress": (session.uploaded_size as f64 / session.total_size as f64) * 100.0
            });
            
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
        None => {
            let response = serde_json::json!({
                "error": "Session not found",
                "status": "error"
            });
            
            Ok(Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .body(Box::new(StringBody::new(response.to_string())) as BoxBody)?)
        }
    }
}