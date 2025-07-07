use std::io::{BufReader, BufWriter};
use std::fs::File;

pub struct PerformanceOptimizations;

impl PerformanceOptimizations {
    // Buffered file operations
    pub fn create_buffered_reader(file: File) -> BufReader<File> {
        BufReader::with_capacity(64 * 1024, file) // 64KB buffer
    }
    
    pub fn create_buffered_writer(file: File) -> BufWriter<File> {
        BufWriter::with_capacity(64 * 1024, file) // 64KB buffer
    }
    
    // Compression for text files
    pub fn should_compress(content_type: &str) -> bool {
        matches!(content_type, 
            "text/plain" | "text/html" | "text/css" | "text/javascript" |
            "application/json" | "application/xml" | "text/xml"
        )
    }
}