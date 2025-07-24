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
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 0; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }}
        
        /* Extended full-width header */
        .header {{ 
            background: rgba(255, 255, 255, 0.95); 
            backdrop-filter: blur(10px); 
            box-shadow: 0 4px 20px rgba(0,0,0,0.1); 
            position: sticky;
            top: 0;
            z-index: 100;
            width: 100%;
            margin: 0;
            padding: 20px 0;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        
        /* Hamburger menu integrated into header */
        .hamburger-menu {{ 
            padding-left: 20px;
            z-index: 1001; 
        }}
        .hamburger-btn {{ 
            background: rgba(37, 99, 235, 0.1); 
            backdrop-filter: blur(10px); 
            border: 1px solid rgba(37, 99, 235, 0.2); 
            border-radius: 12px; 
            padding: 12px; 
            cursor: pointer; 
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); 
            box-shadow: 0 4px 16px rgba(37, 99, 235, 0.2);
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .hamburger-btn:hover {{ 
            background: rgba(37, 99, 235, 0.2); 
            transform: translateY(-2px); 
            box-shadow: 0 8px 25px rgba(37, 99, 235, 0.3);
        }}
        
        /* Header title section */
        .header-title {{ 
            flex: 1;
            text-align: center;
            margin: 0 20px;
        }}
        .header-title h1 {{ 
            color: #1f2937; 
            margin: 0; 
            font-size: 28px; 
            font-weight: 700;
        }}
        
        /* Breadcrumb integrated into header */
        .breadcrumb {{ 
            font-size: 14px; 
            color: #6b7280; 
            margin-top: 5px;
            min-height: 16px; 
            display: flex; 
            align-items: center; 
            justify-content: center;
        }}
        .breadcrumb a {{ color: #2563eb; text-decoration: none; font-weight: 500; }}
        .breadcrumb a:hover {{ text-decoration: underline; color: #1d4ed8; }}
        .breadcrumb-separator {{ margin: 0 8px; color: #9ca3af; }}
        
        /* Upload button integrated into header */
        .header-actions {{ 
            padding-right: 20px;
        }}
        .upload-btn {{ 
            background: linear-gradient(135deg, #f97316, #ea580c); 
            color: white; 
            padding: 12px 24px; 
            border: none; 
            border-radius: 8px; 
            text-decoration: none; 
            font-size: 14px; 
            font-weight: 600;
            cursor: pointer; 
            transition: all 0.3s ease; 
            box-shadow: 0 4px 16px rgba(249, 115, 22, 0.3);
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .upload-btn:hover {{ 
            background: linear-gradient(135deg, #ea580c, #dc2626); 
            transform: translateY(-2px); 
            box-shadow: 0 8px 25px rgba(249, 115, 22, 0.4); 
        }}
        .upload-btn::before {{ 
            content: '📤'; 
            font-size: 16px; 
        }}
        
        /* Animated hamburger icon */
        .hamburger-icon {{ 
            width: 22px; 
            height: 16px; 
            position: relative; 
            transition: all 0.3s ease;
        }}
        .hamburger-icon span {{ 
            display: block; 
            position: absolute; 
            height: 2px; 
            width: 100%; 
            background: #2563eb; 
            border-radius: 2px; 
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        }}
        .hamburger-icon span:nth-child(1) {{ top: 0; }}
        .hamburger-icon span:nth-child(2) {{ top: 7px; }}
        .hamburger-icon span:nth-child(3) {{ top: 14px; }}
        
        /* Active state animation */
        .hamburger-btn.active {{ 
            background: rgba(37, 99, 235, 0.2);
            box-shadow: 0 8px 25px rgba(37, 99, 235, 0.4);
        }}
        .hamburger-btn.active .hamburger-icon span:nth-child(1) {{ 
            top: 7px; 
            transform: rotate(45deg); 
        }}
        .hamburger-btn.active .hamburger-icon span:nth-child(2) {{ 
            opacity: 0; 
            transform: translateX(-20px); 
        }}
        .hamburger-btn.active .hamburger-icon span:nth-child(3) {{ 
            top: 7px; 
            transform: rotate(-45deg); 
        }}
        
        /* Backdrop overlay */
        .menu-backdrop {{ 
            position: fixed; 
            top: 0; 
            left: 0; 
            width: 100vw; 
            height: 100vh; 
            background: rgba(0, 0, 0, 0.4); 
            backdrop-filter: blur(8px); 
            z-index: 999; 
            opacity: 0; 
            visibility: hidden; 
            transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        }}
        .menu-backdrop.show {{ opacity: 1; visibility: visible; }}
        
        /* Sliding sidebar */
        .sidebar {{ 
            position: fixed; 
            top: 0; 
            left: -400px; 
            width: 400px; 
            height: 100vh; 
            background: linear-gradient(145deg, rgba(255, 255, 255, 0.95), rgba(255, 255, 255, 0.85)); 
            backdrop-filter: blur(20px); 
            border-right: 1px solid rgba(255, 255, 255, 0.3); 
            z-index: 1000; 
            transition: left 0.4s cubic-bezier(0.4, 0, 0.2, 1); 
            box-shadow: 4px 0 50px rgba(0, 0, 0, 0.15);
            overflow-y: auto;
        }}
        .sidebar.show {{ left: 0; }}
        
        /* Sidebar header */
        .sidebar-header {{ 
            padding: 40px 30px 30px; 
            border-bottom: 1px solid rgba(0, 0, 0, 0.1); 
            background: linear-gradient(135deg, #f97316, #2563eb); 
            color: white; 
            position: relative;
        }}
        .sidebar-title {{ 
            font-size: 24px; 
            font-weight: 700; 
            margin: 0; 
            display: flex; 
            align-items: center; 
            gap: 12px;
        }}
        .sidebar-subtitle {{ 
            font-size: 14px; 
            opacity: 0.9; 
            margin: 8px 0 0; 
            font-weight: 400;
        }}
        
        /* Navigation sections */
        .nav-section {{ 
            padding: 20px 0; 
            border-bottom: 1px solid rgba(0, 0, 0, 0.05);
        }}
        .nav-section:last-child {{ border-bottom: none; }}
        
        .section-title {{ 
            font-size: 11px; 
            font-weight: 700; 
            text-transform: uppercase; 
            letter-spacing: 1.5px; 
            color: #6b7280; 
            margin: 0 30px 15px; 
            opacity: 0.8;
        }}
        
        .nav-item {{ 
            display: flex; 
            align-items: center; 
            padding: 16px 30px; 
            color: #374151; 
            text-decoration: none; 
            transition: all 0.2s ease; 
            border: none; 
            background: none; 
            width: 100%; 
            text-align: left; 
            cursor: pointer; 
            font-size: 15px; 
            font-weight: 500;
        }}
        .nav-item:hover {{ 
            background: linear-gradient(90deg, rgba(37, 99, 235, 0.1), transparent); 
            color: #2563eb; 
            transform: translateX(8px);
        }}
        
        .nav-icon {{ 
            width: 24px; 
            height: 24px; 
            margin-right: 16px; 
            font-size: 18px; 
            display: flex; 
            align-items: center; 
            justify-content: center; 
            border-radius: 8px; 
            background: rgba(37, 99, 235, 0.1); 
            transition: all 0.2s ease;
        }}
        .nav-item:hover .nav-icon {{ 
            background: #2563eb; 
            color: white; 
            transform: scale(1.1);
        }}
        
        .nav-shortcut {{ 
            margin-left: auto; 
            font-size: 11px; 
            font-weight: 600; 
            color: #9ca3af; 
            background: rgba(156, 163, 175, 0.15); 
            padding: 4px 8px; 
            border-radius: 6px; 
            opacity: 0.8;
        }}
        
        /* Special styling for logout */
        .nav-item.danger {{ color: #dc2626; }}
        .nav-item.danger:hover {{ 
            background: linear-gradient(90deg, rgba(220, 38, 38, 0.1), transparent); 
            color: #dc2626;
        }}
        .nav-item.danger .nav-icon {{ background: rgba(220, 38, 38, 0.1); }}
        .nav-item.danger:hover .nav-icon {{ background: #dc2626; color: white; }}
        
        /* Responsive design */
        @media (max-width: 768px) {{
            .sidebar {{ width: 320px; left: -320px; }}
            .header {{ padding: 15px 0; }}
            .header-title h1 {{ font-size: 24px; }}
            .hamburger-menu {{ padding-left: 15px; }}
            .header-actions {{ padding-right: 15px; }}
            .upload-btn {{ padding: 10px 16px; font-size: 13px; }}
        }}
        @media (max-width: 480px) {{
            .sidebar {{ width: 100%; left: -100%; }}
            .header-title h1 {{ font-size: 20px; }}
            .upload-btn {{ padding: 8px 12px; font-size: 12px; }}
            .upload-btn::before {{ font-size: 14px; }}
        }}

        /* Main content area with proper spacing */
        .content-wrapper {{ 
            position: relative; 
            z-index: 1; 
            isolation: isolate;
            padding: 20px;
        }}
        
        /* Grid Container */
        .grid-container {{ 
            display: grid; 
            grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); 
            gap: 16px; 
            padding: 0; 
            position: relative;
            z-index: 1;
        }}
        
        /* Grid Items */
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
            z-index: 1;
        }}
        .grid-item:hover {{ 
            transform: translateY(-4px); 
            box-shadow: 0 8px 25px rgba(0,0,0,0.15); 
            background: rgba(255, 255, 255, 0.95); 
            z-index: 2;
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
    </style>
</head>
<body>
    <!-- Menu backdrop -->
    <div class="menu-backdrop" id="menuBackdrop"></div>
    
    <!-- Sliding sidebar -->
    <div class="sidebar" id="sidebar">
        <div class="sidebar-header">
            <h2 class="sidebar-title">
                🦀 File Manager
            </h2>
            <p class="sidebar-subtitle">FirxTTech Solutions</p>
        </div>
        
        <div class="nav-section">
            <h3 class="section-title">Navigation</h3>
            <button class="nav-item" onclick="goHome()">
                <div class="nav-icon">🏠</div>
                Home
                <span class="nav-shortcut">Ctrl+H</span>
            </button>
            <button class="nav-item" onclick="refreshPage()">
                <div class="nav-icon">🔄</div>
                Refresh
                <span class="nav-shortcut">F5</span>
            </button>
        </div>
        
        <div class="nav-section">
            <h3 class="section-title">File Operations</h3>
            <button class="nav-item" onclick="showUploadDialog()">
                <div class="nav-icon">📤</div>
                Upload Files
                <span class="nav-shortcut">Ctrl+U</span>
            </button>
            <button class="nav-item" onclick="createNewFolder()">
                <div class="nav-icon">📁</div>
                New Folder
                <span class="nav-shortcut">Ctrl+N</span>
            </button>
        </div>
        
        <div class="nav-section">
            <h3 class="section-title">Settings</h3>
            <button class="nav-item" onclick="showSettings()">
                <div class="nav-icon">⚙️</div>
                Settings
                <span class="nav-shortcut">Ctrl+,</span>
            </button>
            <button class="nav-item" onclick="showHelp()">
                <div class="nav-icon">❓</div>
                Help & About
                <span class="nav-shortcut">F1</span>
            </button>
        </div>
        
        <div class="nav-section">
            <button class="nav-item danger" onclick="logout()">
                <div class="nav-icon">🚪</div>
                Logout
                <span class="nav-shortcut">Ctrl+L</span>
            </button>
        </div>
    </div>
    
    <!-- Full-width integrated header -->
    <div class="header">
        <!-- Left: Hamburger menu -->
        <div class="hamburger-menu">
            <button class="hamburger-btn" id="hamburgerBtn" onclick="toggleSidebar()">
                <div class="hamburger-icon">
                    <span></span>
                    <span></span>
                    <span></span>
                </div>
            </button>
        </div>
        
        <!-- Center: Title and breadcrumb -->
        <div class="header-title">
            <h1>File Manager</h1>
            <div class="breadcrumb">
                {}
            </div>
        </div>
        
        <!-- Right: Upload button -->
        <div class="header-actions">
            <a href="/upload" class="upload-btn">Upload</a>
        </div>
    </div>
    
    <div class="content-wrapper">
        <div class="grid-container">

    <script>
        let isSidebarOpen = false;
        
        function toggleSidebar() {{
            const sidebar = document.getElementById('sidebar');
            const backdrop = document.getElementById('menuBackdrop');
            const hamburgerBtn = document.getElementById('hamburgerBtn');
            
            isSidebarOpen = !isSidebarOpen;
            
            if (isSidebarOpen) {{
                sidebar.classList.add('show');
                backdrop.classList.add('show');
                hamburgerBtn.classList.add('active');
                document.body.style.overflow = 'hidden';
            }} else {{
                sidebar.classList.remove('show');
                backdrop.classList.remove('show');
                hamburgerBtn.classList.remove('active');
                document.body.style.overflow = '';
            }}
        }}
        
        function closeSidebar() {{
            if (isSidebarOpen) {{
                toggleSidebar();
            }}
        }}
        
        // Close on backdrop click
        document.getElementById('menuBackdrop').addEventListener('click', closeSidebar);
        
        // Menu action functions
        function goHome() {{
            window.location.href = '/';
            closeSidebar();
        }}
        
        function refreshPage() {{
            window.location.reload();
            closeSidebar();
        }}
        
        function showUploadDialog() {{
            window.location.href = '/upload';
            closeSidebar();
        }}
        
        function createNewFolder() {{
            const folderName = prompt('Enter folder name:');
            if (folderName) {{
                // TODO: Implement create folder functionality
                alert('Create folder functionality will be implemented soon!');
            }}
            closeSidebar();
        }}
        
        function showSettings() {{
            // TODO: Implement settings modal/page
            alert('Settings panel will be implemented soon!');
            closeSidebar();
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
            closeSidebar();
        }}
        
        function logout() {{
            if (confirm('Are you sure you want to logout?')) {{
                window.location.href = '/logout';
            }}
            closeSidebar();
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
                if (isSidebarOpen) {{
                    closeSidebar();
                }}
            }}
        }});
        
        // Prevent sidebar from closing when clicking inside it
        document.getElementById('sidebar').addEventListener('click', function(e) {{
            e.stopPropagation();
        }});
    </script>
"#, url_path, 
    // Generate breadcrumb content
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