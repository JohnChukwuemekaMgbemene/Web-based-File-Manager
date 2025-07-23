use crate::utils::format_file_size;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
}

// Add URL encoding function
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '%' => "%25".to_string(),
            '#' => "%23".to_string(),
            '?' => "%3F".to_string(),
            '&' => "%26".to_string(),
            '+' => "%2B".to_string(),
            '=' => "%3D".to_string(),
            _ if c.is_ascii_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

pub fn get_directory_entries(dir_path: &Path) -> Result<Vec<FileEntry>, io::Error> {
    let entries = fs::read_dir(dir_path)?;
    let mut file_entries = Vec::new();
    
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };
        
        file_entries.push(FileEntry {
            name,
            path,
            is_dir,
            size,
        });
    }
    
    file_entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    Ok(file_entries)
}

// Add this function to check if file is viewable in browser
fn is_viewable_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    // Images
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || 
    lower.ends_with(".png") || lower.ends_with(".gif") || 
    lower.ends_with(".bmp") || lower.ends_with(".webp") ||
    lower.ends_with(".svg") ||
    // Videos
    lower.ends_with(".mp4") || lower.ends_with(".webm") || 
    lower.ends_with(".ogg") || lower.ends_with(".mov") ||
    // Text files
    lower.ends_with(".txt") || lower.ends_with(".html") || 
    lower.ends_with(".css") || lower.ends_with(".js") ||
    lower.ends_with(".json") || lower.ends_with(".xml") ||
    lower.ends_with(".md") || lower.ends_with(".csv") ||
    // PDF files
    lower.ends_with(".pdf") ||
    // Log files
    lower.ends_with(".log") ||
    // Python and rust files
    lower.ends_with(".rs") ||
    lower.ends_with(".py")
}

// Update the generate_directory_html function

pub fn generate_directory_html(entries: &[FileEntry], url_path: &str) -> String {
    let mut html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>FirxTTech Solutions - {}</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }}
        .header {{ background: rgba(255, 255, 255, 0.95); padding: 20px; border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); margin-bottom: 20px; position: relative; backdrop-filter: blur(10px); z-index: 10; }}
        
        /* Hamburger Menu Styles - Much Higher z-index */
        .hamburger-menu {{ position: absolute; top: 20px; left: 20px; z-index: 9999; }}
        .hamburger-btn {{ background: none; border: none; cursor: pointer; padding: 8px; border-radius: 8px; transition: all 0.3s ease; position: relative; z-index: 10000; }}
        .hamburger-btn:hover {{ background: rgba(37, 99, 235, 0.1); }}
        .hamburger-icon {{ width: 24px; height: 20px; position: relative; transform: rotate(0deg); transition: .5s ease-in-out; }}
        .hamburger-icon span {{ display: block; position: absolute; height: 3px; width: 100%; background: #2563eb; border-radius: 9px; opacity: 1; left: 0; transform: rotate(0deg); transition: .25s ease-in-out; }}
        .hamburger-icon span:nth-child(1) {{ top: 0px; }}
        .hamburger-icon span:nth-child(2) {{ top: 8px; }}
        .hamburger-icon span:nth-child(3) {{ top: 16px; }}
        
        /* Hamburger Animation */
        .hamburger-btn.active .hamburger-icon span:nth-child(1) {{ top: 8px; transform: rotate(135deg); }}
        .hamburger-btn.active .hamburger-icon span:nth-child(2) {{ opacity: 0; left: -60px; }}
        .hamburger-btn.active .hamburger-icon span:nth-child(3) {{ top: 8px; transform: rotate(-135deg); }}
        
        /* Dropdown Menu - Highest z-index */
        .dropdown-menu {{ 
            position: fixed; 
            top: 80px; 
            left: 40px; 
            background: rgba(255, 255, 255, 0.98); 
            border-radius: 12px; 
            box-shadow: 0 8px 32px rgba(0,0,0,0.25); 
            backdrop-filter: blur(15px); 
            border: 1px solid rgba(255, 255, 255, 0.3); 
            min-width: 220px; 
            opacity: 0; 
            visibility: hidden; 
            transform: translateY(-10px); 
            transition: all 0.3s ease; 
            z-index: 9999;
            max-height: 80vh;
            overflow-y: auto;
        }}
        .dropdown-menu.show {{ opacity: 1; visibility: visible; transform: translateY(0); }}
        .dropdown-menu::before {{ 
            content: ''; 
            position: absolute; 
            top: -8px; 
            left: 20px; 
            width: 0; 
            height: 0; 
            border-left: 8px solid transparent; 
            border-right: 8px solid transparent; 
            border-bottom: 8px solid rgba(255, 255, 255, 0.98); 
            z-index: 10000;
        }}
        
        .menu-item {{ 
            padding: 14px 20px; 
            border-bottom: 1px solid rgba(0,0,0,0.08); 
            cursor: pointer; 
            transition: all 0.2s ease; 
            display: flex; 
            align-items: center; 
            gap: 12px; 
            color: #374151; 
            background: rgba(255, 255, 255, 0.9);
            position: relative;
            z-index: 9999;
        }}
        .menu-item:last-child {{ border-bottom: none; }}
        .menu-item:hover {{ 
            background: rgba(37, 99, 235, 0.1); 
            color: #2563eb; 
            transform: translateX(2px);
        }}
        .menu-item:first-child {{ border-radius: 12px 12px 0 0; }}
        .menu-item:last-child {{ border-radius: 0 0 12px 12px; }}
        
        .menu-icon {{ font-size: 16px; width: 20px; text-align: center; }}
        .menu-text {{ font-weight: 500; flex: 1; }}
        .menu-shortcut {{ font-size: 11px; color: #9ca3af; background: rgba(156, 163, 175, 0.1); padding: 2px 6px; border-radius: 4px; }}
        
        .menu-separator {{ height: 1px; background: rgba(0,0,0,0.1); margin: 6px 0; }}
        
        .breadcrumb {{ margin-bottom: 10px; font-size: 14px; color: #374151; margin-left: 60px; min-height: 20px; display: flex; align-items: center; }}
        .breadcrumb a {{ color: #2563eb; text-decoration: none; font-weight: 500; }}
        .breadcrumb a:hover {{ text-decoration: underline; color: #1d4ed8; }}
        .breadcrumb-separator {{ margin: 0 8px; color: #9ca3af; }}
        .upload-btn {{ position: absolute; top: 30px; right: 20px; background: linear-gradient(135deg, #f97316, #ea580c); color: white; padding: 10px 20px; border: none; border-radius: 8px; text-decoration: none; font-size: 14px; cursor: pointer; transition: all 0.3s ease; box-shadow: 0 2px 10px rgba(249, 115, 22, 0.3); z-index: 100; }}
        .upload-btn:hover {{ background: linear-gradient(135deg, #ea580c, #dc2626); transform: translateY(-2px); box-shadow: 0 4px 20px rgba(249, 115, 22, 0.4); }}
        h1 {{ color: #1f2937; margin: 0; text-align: center; font-size: 28px; }}
        
        /* Grid Container - Force lower z-index */
        .grid-container {{ 
            display: grid; 
            grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); 
            gap: 16px; 
            padding: 0; 
            position: relative;
            z-index: 1;
        }}
        
        /* Grid Items - Force to stay below menu */
        .grid-item {{ 
            background: rgba(113, 152, 224, 0.9); 
            border-radius: 12px; 
            padding: 16px; 
            text-align: center; 
            box-shadow: 0 2px 8px rgba(0,0,0,0.1); 
            transition: all 0.3s ease; 
            cursor: pointer; 
            backdrop-filter: blur(5px); 
            border: 1px solid rgba(255, 255, 255, 0.2);
            position: relative;
            z-index: 1 !important;
            isolation: isolate;
        }}
        .grid-item:hover {{ 
            transform: translateY(-4px); 
            box-shadow: 0 8px 25px rgba(0,0,0,0.15); 
            background: rgba(255, 255, 255, 0.95); 
            z-index: 2 !important;
        }}
        
        .folder-icon {{ width: 52px; height: 52px; background: linear-gradient(135deg, #f97316, #ea580c); border-radius: 8px; margin: 0 auto 12px; position: relative; box-shadow: 0 2px 8px rgba(249, 115, 22, 0.3); }}
        .folder-icon::before {{ content: '📁'; position: absolute; top: 10px; left: 10px; right: 10px; bottom: 10px; background: linear-gradient(135deg, #fed7aa, #fdba74); border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 18px; }}
        .file-icon {{ width: 52px; height: 52px; background: linear-gradient(135deg, #2563eb, #1d4ed8); border-radius: 8px; margin: 0 auto 12px; position: relative; box-shadow: 0 2px 8px rgba(37, 99, 235, 0.3); }}
        .file-icon::before {{ content: '📄'; position: absolute; top: 10px; left: 10px; right: 10px; bottom: 10px; background: linear-gradient(135deg, #dbeafe, #bfdbfe); border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 18px; }}
        .parent-item {{ background: rgba(37, 99, 235, 0.15); border: 2px solid rgba(37, 99, 235, 0.4); }}
        .parent-item:hover {{ background: rgba(37, 99, 235, 0.25); transform: translateY(-4px); box-shadow: 0 8px 25px rgba(37, 99, 235, 0.2); }}
        .parent-item .folder-icon {{ background: linear-gradient(135deg, #2563eb, #1d4ed8); }}
        .parent-item .folder-icon::before {{ background: linear-gradient(135deg, #dbeafe, #bfdbfe); }}
        .parent-item .item-name {{ color: #2563eb; font-weight: 700; }}
        .parent-item .item-info {{ color: #1d4ed8; opacity: 0.9; }}
        .item-name {{ font-weight: 600; margin-bottom: 8px; word-wrap: break-word; font-size: 16px; }}
        .item-info {{ font-size: 12px; opacity: 0.8; }}
        .file-actions {{ margin-top: 12px; display: flex; gap: 8px; justify-content: center; }}
        .action-btn {{ font-size: 11px; padding: 4px 8px; border: 1px solid #2563eb; border-radius: 6px; text-decoration: none; color: #2563eb; background: rgba(255, 255, 255, 0.9); font-weight: 500; transition: all 0.2s ease; }}
        .action-btn:hover {{ background: #2563eb; color: white; transform: translateY(-1px); box-shadow: 0 2px 8px rgba(37, 99, 235, 0.3); }}
        .download-btn {{ border-color: #f97316; color: #f97316; }}
        .download-btn:hover {{ background: #f97316; color: white; box-shadow: 0 2px 8px rgba(249, 115, 22, 0.3); }}
        
        /* Enhanced icons for different file types */
        .image-file .file-icon {{ background: linear-gradient(135deg, #10b981, #059669); }}
        .image-file .file-icon::before {{ content: '🖼️'; background: linear-gradient(135deg, #d1fae5, #a7f3d0); }}
        .video-file .file-icon {{ background: linear-gradient(135deg, #8b5cf6, #7c3aed); }}
        .video-file .file-icon::before {{ content: '🎥'; background: linear-gradient(135deg, #ede9fe, #ddd6fe); }}
        .text-file .file-icon {{ background: linear-gradient(135deg, #f59e0b, #d97706); }}
        .text-file .file-icon::before {{ content: '📝'; background: linear-gradient(135deg, #fef3c7, #fde68a); }}
        .pdf-file .file-icon {{ background: linear-gradient(135deg, #dc2626, #b91c1c); }}
        .pdf-file .file-icon::before {{ content: '📄'; background: linear-gradient(135deg, #fecaca, #fca5a5); }}
        
        /* Overlay for closing menu - Very high z-index */
        .menu-overlay {{ 
            position: fixed; 
            top: 0; 
            left: 0; 
            width: 100vw; 
            height: 100vh; 
            background: rgba(0, 0, 0, 0.2); 
            z-index: 9998; 
            display: none;
            backdrop-filter: blur(2px);
        }}
        .menu-overlay.show {{ display: block; }}
        
        /* Force stacking context for content */
        .content-wrapper {{ 
            position: relative; 
            z-index: 1; 
            isolation: isolate; 
        }}
    </style>
</head>
<body>
    <div class="header">
        <!-- Hamburger Menu -->
        <div class="hamburger-menu">
            <button class="hamburger-btn" id="hamburgerBtn" onclick="toggleMenu()">
                <div class="hamburger-icon">
                    <span></span>
                    <span></span>
                    <span></span>
                </div>
            </button>
            
            <div class="dropdown-menu" id="dropdownMenu">
                <div class="menu-item" onclick="goHome()">
                    <span class="menu-icon">🏠</span>
                    <span class="menu-text">Home</span>
                    <span class="menu-shortcut">Ctrl+H</span>
                </div>
                <div class="menu-item" onclick="refreshPage()">
                    <span class="menu-icon">🔄</span>
                    <span class="menu-text">Refresh</span>
                    <span class="menu-shortcut">F5</span>
                </div>
                <div class="menu-separator"></div>
                <div class="menu-item" onclick="showUploadDialog()">
                    <span class="menu-icon">📤</span>
                    <span class="menu-text">Upload Files</span>
                    <span class="menu-shortcut">Ctrl+U</span>
                </div>
                <div class="menu-item" onclick="createNewFolder()">
                    <span class="menu-icon">📁</span>
                    <span class="menu-text">New Folder</span>
                    <span class="menu-shortcut">Ctrl+N</span>
                </div>
                <div class="menu-separator"></div>
                <div class="menu-item" onclick="showSettings()">
                    <span class="menu-icon">⚙️</span>
                    <span class="menu-text">Settings</span>
                    <span class="menu-shortcut">Ctrl+,</span>
                </div>
                <div class="menu-item" onclick="showHelp()">
                    <span class="menu-icon">❓</span>
                    <span class="menu-text">Help & About</span>
                    <span class="menu-shortcut">F1</span>
                </div>
                <div class="menu-separator"></div>
                <div class="menu-item" onclick="logout()">
                    <span class="menu-icon">🚪</span>
                    <span class="menu-text">Logout</span>
                    <span class="menu-shortcut">Ctrl+L</span>
                </div>
            </div>
        </div>
        
        <!-- Overlay for closing menu -->
        <div class="menu-overlay" id="menuOverlay" onclick="closeMenu()"></div>
        
        <div class="breadcrumb">
            {}
        </div>
        <h1>File Manager</h1>
        <a href="/upload" class="upload-btn">Upload</a>
    </div>
    
    <div class="content-wrapper">
        <div class="grid-container">

    <script>
        let isMenuOpen = false;
        
        function toggleMenu() {{
            const hamburgerBtn = document.getElementById('hamburgerBtn');
            const dropdownMenu = document.getElementById('dropdownMenu');
            const menuOverlay = document.getElementById('menuOverlay');
            
            isMenuOpen = !isMenuOpen;
            
            if (isMenuOpen) {{
                hamburgerBtn.classList.add('active');
                dropdownMenu.classList.add('show');
                menuOverlay.classList.add('show');
                // Prevent body scroll when menu is open
                document.body.style.overflow = 'hidden';
            }} else {{
                hamburgerBtn.classList.remove('active');
                dropdownMenu.classList.remove('show');
                menuOverlay.classList.remove('show');
                // Restore body scroll
                document.body.style.overflow = 'auto';
            }}
        }}
        
        function closeMenu() {{
            if (isMenuOpen) {{
                toggleMenu();
            }}
        }}
        
        // Menu action functions
        function goHome() {{
            window.location.href = '/';
            closeMenu();
        }}
        
        function refreshPage() {{
            window.location.reload();
            closeMenu();
        }}
        
        function showUploadDialog() {{
            window.location.href = '/upload';
            closeMenu();
        }}
        
        function createNewFolder() {{
            const folderName = prompt('Enter folder name:');
            if (folderName) {{
                // TODO: Implement create folder functionality
                alert('Create folder functionality will be implemented soon!');
            }}
            closeMenu();
        }}
        
        function showSettings() {{
            // TODO: Implement settings modal/page
            alert('Settings panel will be implemented soon!');
            closeMenu();
        }}
        
        function showHelp() {{
            const helpText = `Web-based File Manager v1.0

Keyboard Shortcuts:
• Ctrl+H: Go to Home
• F5: Refresh current page
• Ctrl+U: Upload files
• Ctrl+N: New folder
• Ctrl+,: Settings
• F1: Help
• Ctrl+L: Logout

Built with Rust 🦀 for maximum performance and security.
System files are automatically filtered for your protection.

FirxTTech Solutions © 2025`;
            alert(helpText);
            closeMenu();
        }}
        
        function logout() {{
            if (confirm('Are you sure you want to logout?')) {{
                window.location.href = '/logout';
            }}
            closeMenu();
        }}
        
        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {{
            if (e.ctrlKey || e.metaKey) {{
                switch(e.key) {{
                    case 'h':
                        e.preventDefault();
                        goHome();
                        break;
                    case 'u':
                        e.preventDefault();
                        showUploadDialog();
                        break;
                    case 'n':
                        e.preventDefault();
                        createNewFolder();
                        break;
                    case ',':
                        e.preventDefault();
                        showSettings();
                        break;
                    case 'l':
                        e.preventDefault();
                        logout();
                        break;
                }}
            }} else if (e.key === 'F5') {{
                e.preventDefault();
                refreshPage();
            }} else if (e.key === 'F1') {{
                e.preventDefault();
                showHelp();
            }} else if (e.key === 'Escape') {{
                closeMenu();
            }}
        }});
        
        // Close menu when clicking outside
        document.addEventListener('click', function(e) {{
            const hamburgerMenu = document.querySelector('.hamburger-menu');
            if (!hamburgerMenu.contains(e.target) && isMenuOpen) {{
                closeMenu();
            }}
        }});
        
        // Prevent menu from closing when clicking inside dropdown
        document.getElementById('dropdownMenu').addEventListener('click', function(e) {{
            e.stopPropagation();
        }});
    </script>
"#, url_path, 
    // Generate breadcrumb content separately to avoid borrowing issues
    {
        if url_path != "/" { 
            let replaced = url_path.replace("/browse", "");
            let trimmed = replaced.trim_matches('/');
            if trimmed.is_empty() {
                String::new()
            } else {
                format!("📁 {}", trimmed.replace("/", " / "))
            }
        } else { 
            String::new() 
        }
    });
    
    if url_path != "/browse" && url_path != "/browse/" {
        let parent_url = get_parent_url(url_path);
        html.push_str(&format!(r#"
        <div class="grid-item parent-item" onclick="location.href='{}'">
            <div class="folder-icon"></div>
            <div class="item-name">Parent Directory</div>
            <div class="item-info">Go up one level</div>
        </div>
        "#, parent_url));
    }
    
    for entry in entries {
        if entry.is_dir {
            let folder_url = format!("{}/{}", url_path.trim_end_matches('/'), url_encode(&entry.name));
            html.push_str(&format!(r#"
            <div class="grid-item" onclick="location.href='{}'">
                <div class="folder-icon"></div>
                <div class="item-name">{}</div>
                <div class="item-info">Folder</div>
            </div>
            "#, folder_url, entry.name));
        } else {
            let file_url = {
                let path_part = url_path.trim_start_matches("/browse");
                if path_part.is_empty() || path_part == "/" {
                    format!("/file/{}", url_encode(&entry.name))
                } else {
                    format!("/file{}/{}", path_part, url_encode(&entry.name))
                }
            };
            
            let download_url = format!("/download{}", 
                if url_path.trim_start_matches("/browse").is_empty() || url_path.trim_start_matches("/browse") == "/" {
                    format!("/{}", url_encode(&entry.name))
                } else {
                    format!("{}/{}", url_path.trim_start_matches("/browse"), url_encode(&entry.name))
                }
            );
            
            let size_str = entry.size.map(format_file_size).unwrap_or_else(|| "Unknown".to_string());
            let file_type_class = get_file_type_class(&entry.name);
            
            if is_viewable_file(&entry.name) {
                html.push_str(&format!(r#"
                <div class="grid-item {}">
                    <div class="file-icon"></div>
                    <div class="item-name">{}</div>
                    <div class="item-info">{}</div>
                    <div class="file-actions">
                        <a href="{}" class="action-btn">View</a>
                        <a href="{}" class="action-btn download-btn">Download</a>
                    </div>
                </div>
                "#, file_type_class, entry.name, size_str, file_url, download_url));
            } else {
                html.push_str(&format!(r#"
                <div class="grid-item {}" onclick="location.href='{}'">
                    <div class="file-icon"></div>
                    <div class="item-name">{}</div>
                    <div class="item-info">{}</div>
                </div>
                "#, file_type_class, download_url, entry.name, size_str));
            }
        }
    }
    
    html.push_str(r#"
        </div>
    </div>
</body>
</html>
    "#);
    
    html
}

// Add this function to determine file type class for styling
fn get_file_type_class(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || 
       lower.ends_with(".png") || lower.ends_with(".gif") || 
       lower.ends_with(".bmp") || lower.ends_with(".webp") ||
       lower.ends_with(".svg") {
        "image-file"
    } else if lower.ends_with(".mp4") || lower.ends_with(".webm") || 
              lower.ends_with(".ogg") || lower.ends_with(".mov") {
        "video-file"
    } else if lower.ends_with(".txt") || lower.ends_with(".html") || 
              lower.ends_with(".css") || lower.ends_with(".js") ||
              lower.ends_with(".json") || lower.ends_with(".xml") ||
              lower.ends_with(".md") || lower.ends_with(".csv") ||
              lower.ends_with(".log") || lower.ends_with(".rs") ||
              lower.ends_with(".py") {
        "text-file"
    } else if lower.ends_with(".pdf") {
        "pdf-file"
    } else {
        ""
    }
}

fn get_parent_url(url_path: &str) -> &str {
    let parent_path = if url_path.ends_with('/') {
        &url_path[..url_path.len()-1]
    } else {
        url_path
    };
    
    if let Some(pos) = parent_path.rfind('/') {
        if pos == 0 || parent_path[..pos].ends_with("/browse") {
            "/browse"
        } else {
            &parent_path[..pos]
        }
    } else {
        "/browse"
    }
}