use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    pub session_id: String,
    pub file_path: String,
    pub total_size: u64,
    pub uploaded_size: u64,
    pub chunk_size: u64,
    pub file_hash: String,
    pub created_at: u64,
}

pub struct ResumableUploadManager {
    sessions: Arc<Mutex<HashMap<String, UploadSession>>>,
    temp_dir: String,
}

impl ResumableUploadManager {
    pub fn new(temp_dir: &str) -> Self {
        std::fs::create_dir_all(temp_dir).unwrap_or_default();
        ResumableUploadManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            temp_dir: temp_dir.to_string(),
        }
    }
    
    pub fn create_session(&self, _filename: &str, total_size: u64) -> String {
        let session_id = Uuid::new_v4().to_string();
        let file_path = format!("{}/{}.partial", self.temp_dir, session_id);
        
        let session = UploadSession {
            session_id: session_id.clone(),
            file_path: file_path.clone(),
            total_size,
            uploaded_size: 0,
            chunk_size: 1024 * 1024, // 1MB chunks
            file_hash: String::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Create partial file
        File::create(&file_path).unwrap_or_else(|e| {
            eprintln!("Failed to create partial file: {}", e);
            panic!("Failed to create partial file");
        });
        
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
        
        println!("Created session: {} for file size: {}", session_id, total_size);
        session_id
    }
    
    pub fn upload_chunk(&self, session_id: &str, chunk_data: &[u8], offset: u64) -> Result<u64, String> {
        let mut sessions = self.sessions.lock().unwrap();
        
        println!("Uploading chunk for session: {}, offset: {}, size: {}", session_id, offset, chunk_data.len());
        
        if let Some(session) = sessions.get_mut(session_id) {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&session.file_path)
                .map_err(|e| format!("Failed to open file: {}", e))?;
            
            file.seek(SeekFrom::Start(offset))
                .map_err(|e| format!("Failed to seek: {}", e))?;
            
            file.write_all(chunk_data)
                .map_err(|e| format!("Failed to write chunk: {}", e))?;
            
            file.flush().map_err(|e| format!("Failed to flush: {}", e))?;
            
            session.uploaded_size = std::cmp::max(session.uploaded_size, offset + chunk_data.len() as u64);
            
            println!("Chunk uploaded successfully. Total uploaded: {}", session.uploaded_size);
            Ok(session.uploaded_size)
        } else {
            println!("Session not found: {}", session_id);
            println!("Available sessions: {:?}", sessions.keys().collect::<Vec<_>>());
            Err("Session not found".to_string())
        }
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<UploadSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }
    
    pub fn complete_upload(&self, session_id: &str, final_path: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if let Some(session) = sessions.remove(session_id) {
            std::fs::rename(&session.file_path, final_path)
                .map_err(|e| format!("Failed to move file: {}", e))?;
            println!("Upload completed: {} -> {}", session.file_path, final_path);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
}

// Global upload manager instance
lazy_static::lazy_static! {
    pub static ref UPLOAD_MANAGER: ResumableUploadManager = ResumableUploadManager::new("./temp_uploads");
}