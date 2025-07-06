use std::fs;
use std::io::Write;
use std::env;
use std::path::Path;

pub async fn handle_upload(body_bytes: Vec<u8>) -> String {
    // Simple file upload handler - in a real application, you'd parse multipart data
    // For now, this is a basic implementation
    
    if body_bytes.is_empty() {
        return "No file data received".to_string();
    }
    
    // Extract filename from multipart data (simplified)
    let body_str = String::from_utf8_lossy(&body_bytes);
    let filename = extract_filename(&body_str).unwrap_or("uploaded_file.txt".to_string());
    
    // Create upload directory in $HOME\Desktop
    let upload_dir = get_upload_directory();
    if !Path::new(&upload_dir).exists() {
        if let Err(e) = fs::create_dir_all(&upload_dir) {
            return format!("Failed to create upload directory: {}", e);
        }
    }
    
    let file_path = format!("{}/{}", upload_dir, filename);
    
    // Extract file content from multipart data (simplified)
    let file_content = extract_file_content(&body_bytes);
    
    match fs::File::create(&file_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&file_content) {
                format!("Failed to write file: {}", e)
            } else {
                format!("File '{}' uploaded successfully to {}!", filename, upload_dir)
            }
        }
        Err(e) => format!("Failed to create file: {}", e),
    }
}

fn get_upload_directory() -> String {
    // Get the home directory and append Desktop/uploads
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    
    format!("{}/Desktop/uploads", home_dir)
}

fn extract_filename(body_str: &str) -> Option<String> {
    // Simple filename extraction from multipart data
    if let Some(start) = body_str.find("filename=\"") {
        let start = start + 10; // Length of "filename=\""
        if let Some(end) = body_str[start..].find('"') {
            return Some(body_str[start..start + end].to_string());
        }
    }
    None
}

fn extract_file_content(body_bytes: &[u8]) -> Vec<u8> {
    // Simple file content extraction from multipart data
    // Look for the double CRLF that separates headers from content
    let separator = b"\r\n\r\n";
    
    for i in 0..body_bytes.len().saturating_sub(separator.len()) {
        if &body_bytes[i..i + separator.len()] == separator {
            let content_start = i + separator.len();
            // Find the end boundary (simplified)
            let content = &body_bytes[content_start..];
            
            // Look for the closing boundary
            if let Some(boundary_pos) = find_boundary_end(content) {
                return content[..boundary_pos].to_vec();
            }
            
            return content.to_vec();
        }
    }
    
    // If no proper multipart structure found, return the whole body
    body_bytes.to_vec()
}

fn find_boundary_end(content: &[u8]) -> Option<usize> {
    // Look for CRLF followed by boundary
    let boundary_marker = b"\r\n--";
    
    for i in 0..content.len().saturating_sub(boundary_marker.len()) {
        if &content[i..i + boundary_marker.len()] == boundary_marker {
            return Some(i);
        }
    }
    None
}