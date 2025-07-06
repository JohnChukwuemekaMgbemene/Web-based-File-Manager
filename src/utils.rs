use hyper::body::Incoming;
use hyper::body::Body;
use std::future::poll_fn;
use std::pin::Pin;
use std::task::Poll;

pub async fn collect_body_bytes(mut body: Incoming) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut bytes = Vec::new();
    
    poll_fn(|cx| {
        let mut body = Pin::new(&mut body);
        loop {
            match body.as_mut().poll_frame(cx) {
                Poll::Ready(Some(Ok(frame))) => {
                    if let Some(chunk) = frame.data_ref() {
                        bytes.extend_from_slice(chunk);
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }).await?;
    
    Ok(bytes)
}

pub fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}