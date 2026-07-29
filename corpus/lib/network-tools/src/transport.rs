use std::collections::VecDeque;

use thiserror::Error;

use crate::DEFAULT_FRAME_LIMIT;

pub trait Transport {
    fn send(&mut self, frame: &[u8]) -> Result<(), TransportError>;
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, TransportError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InjectedFailure {
    Send,
    Receive,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransportError {
    #[error("transport is closed")]
    Closed,
    #[error("loopback queue is full at capacity {capacity}")]
    QueueFull { capacity: usize },
    #[error("frame is {actual} bytes; transport maximum is {limit}")]
    FrameTooLarge { actual: usize, limit: usize },
    #[error("injected {operation} failure")]
    Injected { operation: &'static str },
}

#[derive(Debug)]
pub struct LoopbackTransport {
    queue: VecDeque<Vec<u8>>,
    capacity: usize,
    frame_limit: usize,
    closed: bool,
    injected_failure: Option<InjectedFailure>,
}

impl LoopbackTransport {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
            frame_limit: DEFAULT_FRAME_LIMIT,
            closed: false,
            injected_failure: None,
        }
    }

    pub fn with_frame_limit(mut self, frame_limit: usize) -> Self {
        self.frame_limit = frame_limit;
        self
    }

    pub fn provider_name(&self) -> &'static str {
        "in-memory-loopback"
    }

    pub fn pending_frames(&self) -> usize {
        self.queue.len()
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn inject_failure(&mut self, failure: InjectedFailure) {
        self.injected_failure = Some(failure);
    }
}

impl Transport for LoopbackTransport {
    fn send(&mut self, frame: &[u8]) -> Result<(), TransportError> {
        if self.injected_failure == Some(InjectedFailure::Send) {
            self.injected_failure = None;
            return Err(TransportError::Injected { operation: "send" });
        }
        if self.closed {
            return Err(TransportError::Closed);
        }
        if frame.len() > self.frame_limit {
            return Err(TransportError::FrameTooLarge {
                actual: frame.len(),
                limit: self.frame_limit,
            });
        }
        if self.queue.len() >= self.capacity {
            return Err(TransportError::QueueFull {
                capacity: self.capacity,
            });
        }
        self.queue.push_back(frame.to_vec());
        Ok(())
    }

    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, TransportError> {
        if self.injected_failure == Some(InjectedFailure::Receive) {
            self.injected_failure = None;
            return Err(TransportError::Injected {
                operation: "receive",
            });
        }
        if let Some(frame) = self.queue.pop_front() {
            return Ok(Some(frame));
        }
        if self.closed {
            return Err(TransportError::Closed);
        }
        Ok(None)
    }
}
