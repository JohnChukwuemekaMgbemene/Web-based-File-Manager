use crate::bodies::{BytesBody, StringBody};
use crate::file_browser::{generate_directory_html, get_directory_entries};
use hyper::{Response, StatusCode};
use std::path::Path;
use std::fs;

use super::utils::{BoxBody, is_system_file_or_folder, get_home_directory, url_decode};

// Replace the home_page function:

pub fn home_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Rust Web Server</title>
        <meta charset="UTF-8">
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }
            .container { max-width: 800px; margin: 0 auto; background: rgba(255, 255, 255, 0.95); padding: 40px; border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); position: relative; transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1); min-height: 500px; overflow: hidden; }
            
            .hamburger-menu { position: absolute; top: 20px; left: 20px; z-index: 9999; }
            .hamburger-btn { background: rgba(37, 99, 235, 0.1); border: 1px solid rgba(37, 99, 235, 0.2); cursor: pointer; padding: 12px; border-radius: 12px; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2); backdrop-filter: blur(10px); }
            .hamburger-btn:hover { background: rgba(37, 99, 235, 0.2); transform: translateY(-2px); box-shadow: 0 8px 25px rgba(37, 99, 235, 0.3); }
            .hamburger-icon { width: 22px; height: 16px; position: relative; transform: rotate(0deg); transition: .3s ease-in-out; }
            .hamburger-icon span { display: block; position: absolute; height: 2px; width: 100%; background: #2563eb; border-radius: 2px; opacity: 1; left: 0; transform: rotate(0deg); transition: .3s cubic-bezier(0.4, 0, 0.2, 1); }
            .hamburger-icon span:nth-child(1) { top: 0px; }
            .hamburger-icon span:nth-child(2) { top: 7px; }
            .hamburger-icon span:nth-child(3) { top: 14px; }
            .hamburger-btn.active { background: rgba(37, 99, 235, 0.2); box-shadow: 0 8px 25px rgba(37, 99, 235, 0.4); }
            .hamburger-btn.active .hamburger-icon span:nth-child(1) { top: 7px; transform: rotate(45deg); }
            .hamburger-btn.active .hamburger-icon span:nth-child(2) { opacity: 0; transform: translateX(-20px); }
            .hamburger-btn.active .hamburger-icon span:nth-child(3) { top: 7px; transform: rotate(-45deg); }
            
            .menu-container { position: absolute; top: 0; left: 0; width: 380px; height: 100%; background: linear-gradient(145deg, rgba(255, 255, 255, 0.98), rgba(240, 245, 255, 0.95)); backdrop-filter: blur(20px); border-radius: 16px; border: 1px solid rgba(255, 255, 255, 0.3); box-shadow: 0 12px 40px rgba(0,0,0,0.2); opacity: 0; visibility: hidden; transform: translateX(-100%); transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1); z-index: 1000; display: flex; flex-direction: column; }
            .menu-container.show { opacity: 1; visibility: visible; transform: translateX(0); }
            
            .menu-header { background: linear-gradient(135deg, #f97316, #2563eb); color: white; padding: 20px 24px; text-align: center; position: relative; flex-shrink: 0; border-radius: 16px 16px 0 0; }
            .menu-title { font-size: 22px; font-weight: 700; margin: 0; display: flex; align-items: center; justify-content: center; gap: 10px; }
            .menu-subtitle { font-size: 13px; opacity: 0.9; margin: 6px 0 0; font-weight: 400; }
            
            .menu-list { flex: 1; padding: 0; margin: 0; list-style: none; background: transparent; overflow-y: auto; }
            .menu-item { border-bottom: 1px solid rgba(255, 255, 255, 0.2); transition: all 0.3s ease; cursor: pointer; position: relative; overflow: hidden; }
            .menu-item:last-child { border-bottom: none; }
            .menu-item:hover { background: rgba(255, 255, 255, 0.8); transform: translateX(6px); }
            .menu-item::before { content: ''; position: absolute; left: 0; top: 0; width: 4px; height: 100%; background: transparent; transition: all 0.3s ease; }
            .menu-item:hover::before { background: linear-gradient(135deg, #f97316, #2563eb); }
            .menu-item.primary::before { background: rgba(37, 99, 235, 0.3); }
            .menu-item.secondary::before { background: rgba(249, 115, 22, 0.3); }
            .menu-item.danger::before { background: rgba(220, 38, 38, 0.3); }
            
            .menu-link { display: flex; align-items: center; padding: 16px 20px; text-decoration: none; color: #1f2937; transition: all 0.3s ease; }
            .menu-icon { font-size: 20px; margin-right: 14px; flex-shrink: 0; filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1)); }
            .menu-text { flex: 1; }
            .menu-title-text { font-size: 15px; font-weight: 600; margin-bottom: 3px; }
            .menu-desc { font-size: 11px; color: #6b7280; }
            .menu-shortcut { background: rgba(107, 114, 128, 0.1); color: #6b7280; padding: 3px 7px; border-radius: 5px; font-size: 10px; font-weight: 600; letter-spacing: 0.3px; }
            
            .content-wrapper { transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1); position: relative; z-index: 1; }
            .content-wrapper.menu-open { filter: blur(3px); opacity: 0.7; transform: scale(0.98); pointer-events: none; }
            
            .main-content { transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1); opacity: 1; transform: scale(1); }
            
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
            
            @media (max-width: 768px) { .container { padding: 20px; margin: 10px; } .menu-container { width: 320px; } .menu-header { padding: 16px 20px; } .menu-title { font-size: 20px; } .menu-subtitle { font-size: 12px; } .menu-link { padding: 14px 16px; } .menu-icon { font-size: 18px; margin-right: 12px; } .menu-title-text { font-size: 14px; } .menu-desc { font-size: 10px; } .menu-shortcut { font-size: 9px; padding: 2px 5px; } .hamburger-menu { top: 15px; left: 15px; } }
            @media (max-width: 480px) { .container { padding: 15px; margin: 5px; } .menu-container { width: 280px; } .menu-header { padding: 14px 18px; } .menu-title { font-size: 18px; gap: 8px; } .menu-subtitle { font-size: 11px; } .menu-link { padding: 12px 14px; } .menu-icon { font-size: 16px; margin-right: 10px; } .menu-title-text { font-size: 13px; } .menu-desc { font-size: 9px; } .menu-shortcut { font-size: 8px; padding: 2px 4px; } .hamburger-menu { top: 12px; left: 12px; } }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="hamburger-menu">
                <button class="hamburger-btn" id="hamburgerBtn" onclick="toggleMenu()">
                    <div class="hamburger-icon">
                        <span></span>
                        <span></span>
                        <span></span>
                    </div>
                </button>
            </div>
            
            <div class="menu-container" id="menuContainer">
                <div class="menu-header">
                    <h2 class="menu-title">🦀 File Manager</h2>
                    <p class="menu-subtitle">Choose an action</p>
                </div>
                
                <ul class="menu-list">
                    <li class="menu-item primary" onclick="browseFolders()">
                        <div class="menu-link">
                            <span class="menu-icon">📁</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Browse Files</div>
                                <div class="menu-desc">Navigate through your files and folders</div>
                            </div>
                            <span class="menu-shortcut">Ctrl+B</span>
                        </div>
                    </li>
                    
                    <li class="menu-item secondary" onclick="showUploadDialog()">
                        <div class="menu-link">
                            <span class="menu-icon">📤</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Upload Files</div>
                                <div class="menu-desc">Upload new files to your storage</div>
                            </div>
                            <span class="menu-shortcut">Ctrl+U</span>
                        </div>
                    </li>
                    
                    <li class="menu-item" onclick="refreshPage()">
                        <div class="menu-link">
                            <span class="menu-icon">🔄</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Refresh</div>
                                <div class="menu-desc">Reload the current page</div>
                            </div>
                            <span class="menu-shortcut">F5</span>
                        </div>
                    </li>
                    
                    <li class="menu-item" onclick="showSettings()">
                        <div class="menu-link">
                            <span class="menu-icon">⚙️</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Settings</div>
                                <div class="menu-desc">Configure your preferences</div>
                            </div>
                            <span class="menu-shortcut">Ctrl+,</span>
                        </div>
                    </li>
                    
                    <li class="menu-item" onclick="showHelp()">
                        <div class="menu-link">
                            <span class="menu-icon">❓</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Help & About</div>
                                <div class="menu-desc">Get help and app information</div>
                            </div>
                            <span class="menu-shortcut">F1</span>
                        </div>
                    </li>
                    
                    <li class="menu-item danger" onclick="logout()">
                        <div class="menu-link">
                            <span class="menu-icon">🚪</span>
                            <div class="menu-text">
                                <div class="menu-title-text">Logout</div>
                                <div class="menu-desc">End your current session</div>
                            </div>
                            <span class="menu-shortcut">Ctrl+L</span>
                        </div>
                    </li>
                </ul>
            </div>
            
            <div class="content-wrapper" id="contentWrapper">
                <div class="main-content">
                    <h1>Web-based File Browser</h1>
                    <p class="welcome-text">Your personal file browser built for easy file sharing and transfer</p>
                    <div class="nav">
                        <a href="/browse">Browse Files</a>
                        <a href="/upload">Upload Files</a>
                    </div>
                    <div class="features">
                        <div class="feature">
                            <div class="feature-icon">⚡</div>
                            <h3>Fast & Efficient</h3>
                            <p>Built with Rust for maximum performance and safety</p>
                        </div>
                        <div class="feature">
                            <div class="feature-icon">🔒</div>
                            <h3>Secure</h3>
                            <p>System files are automatically filtered and protected</p>
                        </div>
                        <div class="feature">
                            <div class="feature-icon">🌐</div>
                            <h3>Web-Based</h3>
                            <p>Access your files from any device with a web browser</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <script>
            let isMenuOpen = false;
            
            function toggleMenu() {
                const hamburgerBtn = document.getElementById('hamburgerBtn');
                const menuContainer = document.getElementById('menuContainer');
                const contentWrapper = document.getElementById('contentWrapper');
                
                isMenuOpen = !isMenuOpen;
                
                if (isMenuOpen) {
                    hamburgerBtn.classList.add('active');
                    menuContainer.classList.add('show');
                    contentWrapper.classList.add('menu-open');
                    document.body.style.overflow = 'hidden';
                } else {
                    hamburgerBtn.classList.remove('active');
                    menuContainer.classList.remove('show');
                    contentWrapper.classList.remove('menu-open');
                    document.body.style.overflow = 'auto';
                }
            }
            
            function closeMenu() { if (isMenuOpen) toggleMenu(); }
            function refreshPage() { window.location.reload(); closeMenu(); }
            function showUploadDialog() { window.location.href = '/upload'; closeMenu(); }
            function browseFolders() { window.location.href = '/browse'; closeMenu(); }
            function showSettings() { alert('Settings panel coming soon!\n\nPlanned features:\n• Theme customization\n• File view preferences\n• Upload settings\n• Security options'); closeMenu(); }
            function showHelp() { alert('🦀 Web-based File Manager v1.0\n\nKeyboard Shortcuts:\n• Ctrl+B: Browse Files\n• Ctrl+U: Upload Files\n• F5: Refresh\n• Ctrl+,: Settings\n• F1: Help\n• Ctrl+L: Logout\n• Esc: Close Menu\n\nBuilt with Rust for maximum performance and security.\nSystem files are automatically filtered for your protection.\n\nFirxTTech Solutions © 2025'); closeMenu(); }
            function logout() { if (confirm('Are you sure you want to logout?\nThis will end your current session.')) window.location.href = '/logout'; closeMenu(); }
            
            document.addEventListener('keydown', function(e) {
                if (e.ctrlKey || e.metaKey) {
                    switch(e.key) {
                        case 'u': e.preventDefault(); showUploadDialog(); break;
                        case 'b': e.preventDefault(); browseFolders(); break;
                        case ',': e.preventDefault(); showSettings(); break;
                        case 'l': e.preventDefault(); logout(); break;
                    }
                } else if (e.key === 'F5') {
                    e.preventDefault(); refreshPage();
                } else if (e.key === 'Escape') {
                    closeMenu();
                }
            });
            
            // Close menu when clicking outside
            document.addEventListener('click', function(e) {
                if (isMenuOpen && !document.getElementById('menuContainer').contains(e.target) && !document.getElementById('hamburgerBtn').contains(e.target)) {
                    closeMenu();
                }
            });
        </script>
    </body>
    </html>
    "#;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
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