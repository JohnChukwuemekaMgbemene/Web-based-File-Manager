use crate::bodies::StringBody;
use crate::upload::handle_upload;
use crate::utils::collect_body_bytes;
use crate::resumable_upload::UPLOAD_MANAGER;
use hyper::{Request, Response};
use hyper::body::Incoming;
use std::path::Path;
use serde_json;

use super::utils::{BoxBody, get_home_directory};

pub fn upload_page() -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Upload Files - File Browser</title>
    <meta charset="UTF-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: linear-gradient(135deg, #f97316 0%, #2563eb 100%); min-height: 100vh; }
        .container { max-width: 800px; margin: 0 auto; background: rgba(255, 255, 255, 0.95); padding: 40px; border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.15); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.2); }
        h1 { color: #1f2937; text-align: center; margin-bottom: 30px; font-size: 32px; background: linear-gradient(135deg, #f97316, #2563eb); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
        .select-files-link { display: none; color: #2563eb; text-decoration: underline; cursor: pointer; margin-bottom: 20px; font-weight: 600; }
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
        .upload-btn { background: #28a745; color: white; width: 100%; border: none; padding: 20px 60px; border-radius: 12px; cursor: pointer; font-size: 18px; font-weight: 600; transition: all 0.3s ease; box-shadow: 0 4px 15px rgba(40, 167, 69, 0.3); }
        .upload-btn:hover { background: #218838; transform: translateY(-2px); box-shadow: 0 6px 20px rgba(40, 167, 69, 0.4); }
        .upload-btn:disabled { background: #6c757d; cursor: not-allowed; transform: none; box-shadow: none; }
        .upload-progress { width: 100%; height: 20px; background-color: #f0f0f0; border-radius: 10px; overflow: hidden; margin: 10px 0; }
        .progress-bar { height: 100%; background: linear-gradient(90deg, #4caf50, #45a049); transition: width 0.3s ease; }
        .upload-speed { font-size: 14px; color: #666; margin: 5px 0; }
        .upload-item { border: 1px solid #ddd; border-radius: 8px; padding: 15px; margin: 10px 0; background: #f9f9f9; }
        .upload-item.completed { background: #e8f5e8; border-color: #4caf50; }
        .upload-item.failed { background: #ffe8e8; border-color: #f44336; }
        .pause-resume-btn, .cancel-btn { background: #ff9800; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer; margin: 0 5px; }
        .cancel-btn { background: #f44336; }
        .upload-actions.hidden { display: none; }
        .navigation { display: flex; justify-content: space-between; align-items: center; margin-top: 30px; }
        .nav-button { background: #2563eb; color: white; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer; font-size: 16px; font-weight: 600; transition: all 0.3s ease; text-decoration: none; display: inline-flex; align-items: center; gap: 8px; }
        .nav-button:hover { background: #1d4ed8; transform: translateY(-2px); box-shadow: 0 4px 15px rgba(37, 99, 235, 0.3); }
        .nav-button.home { background: #f97316; }
        .nav-button.home:hover { background: #ea580c; box-shadow: 0 4px 15px rgba(249, 115, 22, 0.3); }
        .empty-state { text-align: center; color: #6c757d; font-style: italic; margin: 20px 0; }
        .hidden { display: none !important; }
    </style>
</head>
<body>
    <div class="container">
        <h1>📁 Upload Files</h1>
        
        <div class="select-files-link" id="selectFilesLink" onclick="showUploadArea()">+ Select Files</div>
        
        <div class="upload-area" id="uploadArea">
            <p>Drag & drop files here or click to select</p>
            <input type="file" id="fileInput" multiple style="display: none;">
        </div>
        
        <div class="file-list" id="fileList">
            <div class="empty-state" id="emptyState">No files selected</div>
        </div>
        
        <div class="upload-controls">
            <button class="upload-btn" id="uploadBtn" disabled onclick="startUpload()">
                Upload Files
            </button>
        </div>
        
        <div id="uploadList"></div>
        
        <div class="navigation">
            <button class="nav-button" onclick="goBack()">← Back</button>
            <a href="/" class="nav-button home">Home</a>
        </div>
    </div>

    <script>
        function goBack() {
            if (window.history.length > 1) {
                window.history.back();
            } else {
                window.location.href = '/browse';
            }
        }

        function showUploadArea() {
            document.getElementById('uploadArea').classList.remove('hidden');
            document.getElementById('selectFilesLink').classList.add('hidden');
        }

        function hideUploadArea() {
            document.getElementById('uploadArea').classList.add('hidden');
            document.getElementById('selectFilesLink').classList.remove('hidden');
        }

        let selectedFiles = [];
        let uploader = null;
        let allUploadsCompleted = false;
        
        class ResumableUploader {
            constructor() {
                this.uploads = new Map();
                this.chunkSize = 1024 * 1024; // 1MB chunks
                this.maxConcurrent = 3;
                this.activeUploads = 0;
                this.completedUploads = 0;
                this.failedUploads = 0;
                this.totalUploads = 0;
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
                    completed: false,
                    failed: false,
                    startTime: Date.now(),
                    element: this.createUploadElement(file.name, uploadId)
                };
                
                this.uploads.set(uploadId, upload);
                
                try {
                    console.log('Starting upload session for:', file.name);
                    const sessionResponse = await fetch('/upload/start', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            filename: file.name,
                            totalSize: file.size
                        })
                    });
                    
                    if (!sessionResponse.ok) {
                        const errorText = await sessionResponse.text();
                        throw new Error(`Failed to start session: ${sessionResponse.status} - ${errorText}`);
                    }
                    
                    const session = await sessionResponse.json();
                    upload.sessionId = session.sessionId;
                    upload.element.querySelector('.upload-status').textContent = 'Uploading...';
                    
                    await this.uploadChunks(upload);
                    
                } catch (error) {
                    console.error('Upload failed:', error);
                    upload.failed = true;
                    upload.element.querySelector('.upload-status').textContent = 'Failed: ' + error.message;
                    upload.element.classList.add('failed');
                    upload.element.querySelector('.upload-actions').classList.add('hidden');
                    this.failedUploads++;
                    this.activeUploads--;
                    this.checkAllUploadsComplete();
                }
            }
            
            async uploadChunks(upload) {
                while (upload.uploadedSize < upload.totalSize && !upload.cancelled && !upload.failed) {
                    if (upload.paused) {
                        await new Promise(resolve => setTimeout(resolve, 100));
                        continue;
                    }
                    
                    const start = upload.uploadedSize;
                    const end = Math.min(start + upload.chunkSize, upload.totalSize);
                    const chunk = upload.file.slice(start, end);
                    
                    try {
                        const response = await fetch(`/upload/chunk/${upload.sessionId}`, {
                            method: 'POST',
                            headers: {
                                'Content-Range': `bytes ${start}-${end-1}/${upload.totalSize}`,
                                'Content-Type': 'application/octet-stream'
                            },
                            body: chunk
                        });
                        
                        if (!response.ok) {
                            const errorText = await response.text();
                            throw new Error(`Chunk upload failed: ${response.status} - ${errorText}`);
                        }
                        
                        const result = await response.json();
                        upload.uploadedSize = result.uploadedSize;
                        this.updateProgress(upload);
                        
                    } catch (error) {
                        upload.failed = true;
                        upload.element.querySelector('.upload-status').textContent = 'Error: ' + error.message;
                        upload.element.classList.add('failed');
                        upload.element.querySelector('.upload-actions').classList.add('hidden');
                        this.failedUploads++;
                        this.activeUploads--;
                        this.checkAllUploadsComplete();
                        return;
                    }
                }
                
                if (upload.uploadedSize >= upload.totalSize && !upload.cancelled && !upload.failed) {
                    await this.completeUpload(upload);
                }
                
                this.activeUploads--;
                this.checkAllUploadsComplete();
            }
            
            async completeUpload(upload) {
                try {
                    const response = await fetch(`/upload/complete/${upload.sessionId}`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            filename: upload.file.name,
                            finalPath: `/uploads/${upload.file.name}`
                        })
                    });
                    
                    if (response.ok) {
                        upload.completed = true;
                        upload.element.querySelector('.upload-status').textContent = 'Completed ✓';
                        upload.element.classList.add('completed');
                        upload.element.querySelector('.upload-actions').classList.add('hidden');
                        
                        this.completedUploads++;
                        this.removeFromQueue(upload.file);
                    } else {
                        const errorText = await response.text();
                        throw new Error(`Complete upload failed: ${response.status} - ${errorText}`);
                    }
                } catch (error) {
                    upload.failed = true;
                    upload.element.querySelector('.upload-status').textContent = 'Completion failed: ' + error.message;
                    upload.element.classList.add('failed');
                    upload.element.querySelector('.upload-actions').classList.add('hidden');
                    this.failedUploads++;
                    this.checkAllUploadsComplete();
                }
            }
            
            checkAllUploadsComplete() {
                const totalFinished = this.completedUploads + this.failedUploads;
                
                if (totalFinished >= this.totalUploads && this.activeUploads === 0) {
                    allUploadsCompleted = true;
                    console.log('All uploads completed. Showing upload area.');
                    
                    setTimeout(() => {
                        showUploadArea();
                        document.getElementById('uploadBtn').disabled = true;
                    }, 10);
                }
            }
            
            removeFromQueue(file) {
                const index = selectedFiles.findIndex(f => f.name === file.name && f.size === file.size);
                if (index !== -1) {
                    selectedFiles.splice(index, 1);
                    updateFileList();
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
                if (upload && !upload.completed && !upload.failed) {
                    upload.paused = !upload.paused;
                    const btn = upload.element.querySelector('.pause-resume-btn');
                    btn.textContent = upload.paused ? 'Resume' : 'Pause';
                }
            }
            
            cancelUpload(uploadId) {
                const upload = this.uploads.get(uploadId);
                if (upload && !upload.completed) {
                    upload.cancelled = true;
                    upload.element.classList.add('failed');
                    upload.element.querySelector('.upload-status').textContent = 'Cancelled';
                    upload.element.querySelector('.upload-actions').classList.add('hidden');
                    this.failedUploads++;
                    this.activeUploads--;
                    this.checkAllUploadsComplete();
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
            
            if (selectedFiles.length > 0) {
                console.log('Clearing completed uploads and starting fresh.');
                document.getElementById('uploadList').innerHTML = '';
                allUploadsCompleted = false;
                hideUploadArea();
            }
        }
        
        function removeFile(index) {
            selectedFiles.splice(index, 1);
            updateFileList();
            
            if (selectedFiles.length === 0) {
                showUploadArea();
            }
        }
        
        function updateFileList() {
            const fileList = document.getElementById('fileList');
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
            uploader.totalUploads = selectedFiles.length;
            uploader.completedUploads = 0;
            uploader.failedUploads = 0;
            document.getElementById('uploadBtn').disabled = true;
            allUploadsCompleted = false;
            
            selectedFiles.forEach(file => {
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

pub async fn handle_upload_request(req: Request<Incoming>) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    use hyper::StatusCode;
    
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

pub async fn start_resumable_upload(req: Request<Incoming>) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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

pub async fn upload_chunk(req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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

pub async fn complete_upload(req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id = path.strip_prefix("/upload/complete/").unwrap();
    
    let body = collect_body_bytes(req.into_body()).await?;
    let body_str = String::from_utf8(body.to_vec())?;
    
    // Parse JSON request
    let complete_request: serde_json::Value = serde_json::from_str(&body_str)?;
    let filename = complete_request["filename"].as_str().unwrap();
    let _final_path = complete_request["finalPath"].as_str().unwrap();
    
    // Get home directory and construct Desktop\Uploads path
    let home_dir = get_home_directory();
    let final_path = format!("{}\\Desktop\\Uploads\\{}", home_dir, filename);
    
    // Ensure Desktop\Uploads directory exists
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

pub async fn get_upload_status(_req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, Box<dyn std::error::Error + Send + Sync>> {
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