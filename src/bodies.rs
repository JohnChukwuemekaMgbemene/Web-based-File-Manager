use hyper::body::Body;
use hyper::body::Bytes;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct StringBody {
    data: Option<String>,
}

impl StringBody {
    pub fn new(data: String) -> Self {
        Self { data: Some(data) }
    }
}

impl Body for StringBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        if let Some(data) = self.data.take() {
            let bytes = Bytes::from(data);
            Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes))))
        } else {
            Poll::Ready(None)
        }
    }
}

pub struct BytesBody {
    data: Option<Vec<u8>>,
}

impl BytesBody {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data: Some(data) }
    }
}

impl Body for BytesBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        if let Some(data) = self.data.take() {
            let bytes = Bytes::from(data);
            Poll::Ready(Some(Ok(hyper::body::Frame::data(bytes))))
        } else {
            Poll::Ready(None)
        }
    }
}