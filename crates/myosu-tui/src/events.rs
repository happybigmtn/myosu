use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, Stream, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal events that drive the TUI event loop.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Terminal tick for animations/updates (60Hz).
    Tick,
    /// Key press from the terminal.
    Key(KeyEvent),
    /// Window resize event.
    Resize(u16, u16),
    /// Async update from external source (solver, network, etc.).
    Update(UpdateEvent),
    /// Application quit signal.
    Quit,
}

/// External update events injected into the event loop.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateEvent {
    /// Solver advisor computed action distribution.
    SolverAdvice { actions: Vec<(String, f32)> },
    /// Game state changed.
    StateChanged { state: String },
    /// Training progress update.
    TrainingProgress { iteration: u64, exploitability: f64 },
    /// Structured shell status update.
    Status {
        state: InteractionState,
        detail: Option<String>,
    },
    /// New message to display.
    Message(String),
}

/// Shell-visible interaction states for loading and operator-facing feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionState {
    Neutral,
    Loading,
    Empty,
    Partial,
    Error,
    Success,
}

/// Event loop for async terminal event handling.
///
/// Bridges crossterm's event stream with tokio channels to enable
/// non-blocking event processing with timeout-based ticks.
pub struct EventLoop {
    /// Receiver for processed events.
    rx: mpsc::UnboundedReceiver<Event>,
    /// Sender for injecting updates from external tasks.
    update_tx: mpsc::UnboundedSender<UpdateEvent>,
    /// Handle to stop the event task.
    _task: tokio::task::JoinHandle<()>,
}

impl EventLoop {
    /// Create a new event loop with the given tick rate.
    ///
    /// # Arguments
    /// * `tick_rate` - Duration between tick events for UI updates.
    ///
    /// # Example
    /// ```no_run
    /// use std::time::Duration;
    /// use myosu_tui::events::EventLoop;
    ///
    /// # tokio_test::block_on(async {
    /// let event_loop = EventLoop::new(Duration::from_millis(16));
    /// # });
    /// ```
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (update_tx, update_rx) = mpsc::unbounded_channel::<UpdateEvent>();
        let _task = Self::spawn_task(
            tx,
            update_rx,
            crossterm::event::EventStream::new(),
            tick_rate,
        );

        Self {
            rx,
            update_tx,
            _task,
        }
    }

    /// Receive the next event from the loop.
    ///
    /// Returns `None` if the event loop has shut down.
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// Get a sender for injecting external updates.
    ///
    /// This can be cloned and passed to async tasks that need to
    /// communicate with the TUI (e.g., solver callbacks).
    pub fn update_sender(&self) -> mpsc::UnboundedSender<UpdateEvent> {
        self.update_tx.clone()
    }

    fn spawn_task<S>(
        tx: mpsc::UnboundedSender<Event>,
        mut update_rx: mpsc::UnboundedReceiver<UpdateEvent>,
        mut reader: S,
        tick_rate: Duration,
    ) -> tokio::task::JoinHandle<()>
    where
        S: Stream<Item = std::io::Result<CrosstermEvent>> + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                let tick = tick_interval.tick().fuse();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = tick => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                let event = match evt {
                                    CrosstermEvent::Key(key) => Event::Key(key),
                                    CrosstermEvent::Resize(w, h) => Event::Resize(w, h),
                                    CrosstermEvent::FocusGained => continue,
                                    CrosstermEvent::FocusLost => continue,
                                    CrosstermEvent::Mouse(_) => continue,
                                    CrosstermEvent::Paste(_) => continue,
                                };
                                if tx.send(event).is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                    Some(update) = update_rx.recv() => {
                        if tx.send(Event::Update(update)).is_err() {
                            break;
                        }
                    }
                }
            }
        })
    }

    #[cfg(test)]
    fn with_stream<S>(tick_rate: Duration, reader: S) -> Self
    where
        S: Stream<Item = std::io::Result<CrosstermEvent>> + Unpin + Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let (update_tx, update_rx) = mpsc::unbounded_channel::<UpdateEvent>();
        let _task = Self::spawn_task(tx, update_rx, reader, tick_rate);

        Self {
            rx,
            update_tx,
            _task,
        }
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.rx.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use futures::stream;

    #[tokio::test]
    async fn key_event_handled() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let reader = stream::iter(vec![Ok(CrosstermEvent::Key(key))]);
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(1), reader);

        let event = tokio::time::timeout(Duration::from_millis(100), loop_handle.next())
            .await
            .expect("timeout waiting for event")
            .expect("event loop closed");

        assert_eq!(event, Event::Key(key));
    }

    #[tokio::test]
    async fn async_response_received() {
        let reader = stream::pending::<std::io::Result<CrosstermEvent>>();
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(1), reader);
        let update_tx = loop_handle.update_sender();

        let update = UpdateEvent::Message("test".into());
        update_tx.send(update.clone()).expect("send failed");

        let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
        let mut found = false;

        while tokio::time::Instant::now() < deadline {
            let timeout = deadline - tokio::time::Instant::now();
            if let Ok(Some(Event::Update(UpdateEvent::Message(msg)))) =
                tokio::time::timeout(timeout, loop_handle.next()).await
            {
                assert_eq!(msg, "test");
                found = true;
                break;
            }
        }

        assert!(found, "did not receive injected update");
    }

    #[tokio::test]
    async fn resize_event_handled() {
        let reader = stream::iter(vec![Ok(CrosstermEvent::Resize(120, 40))]);
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(1), reader);

        let event = tokio::time::timeout(Duration::from_millis(100), loop_handle.next())
            .await
            .expect("timeout waiting for event")
            .expect("event loop closed");

        assert_eq!(event, Event::Resize(120, 40));
    }

    /// Test that the update sender can be cloned and used from multiple
    /// background tasks concurrently.
    #[tokio::test]
    async fn update_sender_cloned() {
        let reader = stream::pending::<std::io::Result<CrosstermEvent>>();
        let loop_handle = EventLoop::with_stream(Duration::from_secs(1), reader);
        let tx1 = loop_handle.update_sender();
        let tx2 = loop_handle.update_sender();

        // Both should be able to send
        assert!(tx1.send(UpdateEvent::Message("from tx1".into())).is_ok());
        assert!(tx2.send(UpdateEvent::Message("from tx2".into())).is_ok());
    }

    #[test]
    fn update_event_variants() {
        let advice = UpdateEvent::SolverAdvice {
            actions: vec![("fold".into(), 0.3), ("call".into(), 0.7)],
        };
        assert_eq!(
            advice,
            UpdateEvent::SolverAdvice {
                actions: vec![("fold".into(), 0.3), ("call".into(), 0.7)]
            }
        );

        let progress = UpdateEvent::TrainingProgress {
            iteration: 1000,
            exploitability: 0.05,
        };
        assert_eq!(
            progress,
            UpdateEvent::TrainingProgress {
                iteration: 1000,
                exploitability: 0.05
            }
        );

        let status = UpdateEvent::Status {
            state: InteractionState::Partial,
            detail: Some("live advice stale".into()),
        };
        assert_eq!(
            status,
            UpdateEvent::Status {
                state: InteractionState::Partial,
                detail: Some("live advice stale".into())
            }
        );
    }
}
