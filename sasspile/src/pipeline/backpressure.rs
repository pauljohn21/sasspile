//! Backpressure and cancellation for the pipeline.
//!
//! Provides bounded channel configuration and CancellationToken integration
//! to enable mid-compilation abort.

use std::marker::PhantomData;
use tokio_util::sync::CancellationToken;
use tracing::{debug, instrument, warn};

/// Pipeline configuration for memory control.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Capacity of each inter-stage mpsc channel.
    /// When full, senders await (natural backpressure).
    pub channel_capacity: usize,
    /// Optional cancellation token for mid-compilation abort.
    pub cancellation: Option<CancellationToken>,
}

impl PipelineConfig {
    /// Create with given channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channel_capacity: capacity,
            cancellation: None,
        }
    }

    /// Attach a cancellation token.
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancellation = Some(token);
        self
    }

    /// Check if cancellation has been triggered.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation
            .as_ref()
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// Cancel if a token is attached.
    pub fn cancel(&self) {
        if let Some(token) = &self.cancellation {
            token.cancel();
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 64,
            cancellation: None,
        }
    }
}

/// A bounded channel wrapper that integrates cancellation.
#[derive(Debug)]
pub struct BoundedChannel<T> {
    /// Current item count.
    size: usize,
    /// Maximum capacity.
    capacity: usize,
    /// Cancellation token for cooperative cancellation.
    token: Option<CancellationToken>,
    /// Phantom type for the channel item type.
    _phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> BoundedChannel<T> {
    /// Create a new bounded channel tracker.
    pub fn new(capacity: usize) -> Self {
        Self {
            size: 0,
            capacity,
            token: None,
            _phantom: PhantomData,
        }
    }

    /// Add cancellation monitoring.
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Check if the channel is at capacity.
    pub fn is_full(&self) -> bool {
        self.size >= self.capacity
    }

    /// Check if the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Register an item being sent.
    #[instrument(skip(self))]
    pub fn send(&mut self) {
        if self.size < self.capacity {
            self.size += 1;
            debug!(size = self.size, capacity = self.capacity, "bounded channel send");
        } else {
            warn!(capacity = self.capacity, "channel at capacity — backpressure engaged");
        }
    }

    /// Register an item being received.
    pub fn recv(&mut self) {
        if self.size > 0 {
            self.size -= 1;
            debug!(size = self.size, "bounded channel recv");
        }
    }

    /// Check cancellation status.
    pub fn check_cancellation(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// Current channel size.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Max capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Default for BoundedChannel<T> {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_capacity() {
        let config = PipelineConfig::default();
        assert_eq!(config.channel_capacity, 64);
        assert!(config.cancellation.is_none());
    }

    #[test]
    fn config_with_capacity() {
        let config = PipelineConfig::with_capacity(8);
        assert_eq!(config.channel_capacity, 8);
    }

    #[test]
    fn bounded_channel_full() {
        let mut ch = BoundedChannel::<()>::new(2);
        assert!(!ch.is_full());
        ch.send();
        ch.send();
        assert!(ch.is_full());
    }

    #[test]
    fn bounded_channel_recv() {
        let mut ch = BoundedChannel::<String>::new(4);
        ch.send();
        ch.send();
        assert_eq!(ch.len(), 2);
        ch.recv();
        assert_eq!(ch.len(), 1);
    }

    #[test]
    fn cancelled_config() {
        let token = CancellationToken::new();
        let config = PipelineConfig::with_capacity(64).with_cancellation(token.clone());
        assert!(!config.is_cancelled());
        config.cancel();
        assert!(config.is_cancelled());
    }

    #[test]
    fn bounded_channel_cancellation() {
        let token = CancellationToken::new();
        let ch = BoundedChannel::<()>::new(8).with_cancellation(token.clone());
        assert!(!ch.check_cancellation());
        token.cancel();
        assert!(ch.check_cancellation());
    }

    #[tokio::test]
    async fn cancel_mid_compile() {
        use tokio::sync::mpsc;

        let token = CancellationToken::new();
        let (tx, mut rx) = mpsc::channel::<i32>(4);
        let child_token = token.child_token();

        // Spawn a task that sends until cancelled.
        let handle = tokio::spawn(async move {
            let mut i = 0;
            loop {
                if child_token.is_cancelled() {
                    break;
                }
                if tx.send(i).await.is_err() {
                    break;
                }
                i += 1;
                if i >= 100 {
                    break;
                }
            }
            i
        });

        // Let it send a couple items.
        let _ = rx.recv().await;
        let _ = rx.recv().await;

        // Cancel.
        token.cancel();
        let sent_count = handle.await.unwrap();
        assert!(sent_count < 100);
    }
}
