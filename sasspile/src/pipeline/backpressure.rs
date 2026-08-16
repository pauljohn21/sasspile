//! Backpressure and cancellation for the pipeline.
//!
//! Provides bounded channel configuration and CancellationToken integration
//! to enable mid-compilation abort.

use tokio_util::sync::CancellationToken;

/// Pipeline configuration for memory control.
///
/// Controls channel capacity for backpressure and optional mid-compilation
/// abort via [`CancellationToken`].
///
/// # Examples
///
/// ```
/// use sasspile::pipeline::PipelineConfig;
///
/// let config = PipelineConfig::with_capacity(128);
/// assert_eq!(config.channel_capacity, 128);
/// assert!(!config.is_cancelled());
/// ```
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
    fn cancelled_config() {
        let token = CancellationToken::new();
        let config = PipelineConfig::with_capacity(64).with_cancellation(token.clone());
        assert!(!config.is_cancelled());
        config.cancel();
        assert!(config.is_cancelled());
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
