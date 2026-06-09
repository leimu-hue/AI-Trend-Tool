use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Lightweight event sent between pipeline stages.
#[derive(Debug, Clone, Copy)]
pub enum PipelineEvent {
    /// Downstream module should process new data.
    NewData,
}

/// Shared pipeline bus carrying inter-module channels and a cancellation token.
#[derive(Clone)]
pub struct Pipeline {
    /// Parser → Filter: new articles have been inserted.
    pub articles_ready_tx: mpsc::Sender<PipelineEvent>,
    /// Filter → Pusher: new push records have been created.
    pub push_ready_tx: mpsc::Sender<PipelineEvent>,
    /// Shared cancellation token for graceful shutdown.
    pub cancel: CancellationToken,
}

impl Pipeline {
    /// Create a new pipeline with capacity-16 channels and a fresh CancellationToken.
    ///
    /// Returns `(Pipeline, articles_rx, push_rx)` so that callers can pass each
    /// receiver to the appropriate downstream module.
    pub fn new() -> (
        Self,
        mpsc::Receiver<PipelineEvent>,
        mpsc::Receiver<PipelineEvent>,
    ) {
        let (articles_ready_tx, articles_rx) = mpsc::channel(16);
        let (push_ready_tx, push_rx) = mpsc::channel(16);
        let cancel = CancellationToken::new();

        let pipeline = Self {
            articles_ready_tx,
            push_ready_tx,
            cancel,
        };

        (pipeline, articles_rx, push_rx)
    }
}
