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
    /// New message to display.
    Message(String),
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
        Self::with_stream(tick_rate, crossterm::event::EventStream::new())
    }

    fn with_stream<S>(tick_rate: Duration, reader: S) -> Self
    where
        S: Stream<Item = std::io::Result<CrosstermEvent>> + Send + Unpin + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let (update_tx, mut update_rx) = mpsc::unbounded_channel::<UpdateEvent>();

        let _task = tokio::spawn(async move {
            Self::run_event_task(tx, &mut update_rx, tick_rate, reader).await;
        });

        Self {
            rx,
            update_tx,
            _task,
        }
    }

    async fn run_event_task<S>(
        tx: mpsc::UnboundedSender<Event>,
        update_rx: &mut mpsc::UnboundedReceiver<UpdateEvent>,
        tick_rate: Duration,
        mut reader: S,
    ) where
        S: Stream<Item = std::io::Result<CrosstermEvent>> + Unpin,
    {
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
                            let Some(event) = Self::map_crossterm_event(evt) else {
                                continue;
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
    }

    fn map_crossterm_event(event: CrosstermEvent) -> Option<Event> {
        match event {
            CrosstermEvent::Key(key) => Some(Event::Key(key)),
            CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
            CrosstermEvent::FocusGained => None,
            CrosstermEvent::FocusLost => None,
            CrosstermEvent::Mouse(_) => None,
            CrosstermEvent::Paste(_) => None,
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
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.rx.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct MockEventStream {
        rx: mpsc::UnboundedReceiver<std::io::Result<CrosstermEvent>>,
    }

    impl MockEventStream {
        fn new() -> (Self, mpsc::UnboundedSender<std::io::Result<CrosstermEvent>>) {
            let (tx, rx) = mpsc::unbounded_channel();
            (Self { rx }, tx)
        }
    }

    impl Stream for MockEventStream {
        type Item = std::io::Result<CrosstermEvent>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.rx.poll_recv(cx)
        }
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    async fn next_event(loop_handle: &mut EventLoop) -> Event {
        tokio::time::timeout(Duration::from_millis(100), loop_handle.next())
            .await
            .expect("timeout waiting for event")
            .expect("event loop closed")
    }

    #[tokio::test]
    async fn tick_event_handled_headless() {
        let (reader, _stream_tx) = MockEventStream::new();
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(60), reader);

        assert_eq!(next_event(&mut loop_handle).await, Event::Tick);
    }

    /// Test that key events are properly handled by the event loop
    /// without requiring a real terminal.
    #[tokio::test]
    async fn key_event_handled() {
        let (reader, stream_tx) = MockEventStream::new();
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(60), reader);

        assert_eq!(next_event(&mut loop_handle).await, Event::Tick);

        let key = press(KeyCode::Enter);
        stream_tx
            .send(Ok(CrosstermEvent::Key(key)))
            .expect("send failed");

        assert_eq!(next_event(&mut loop_handle).await, Event::Key(key));
    }

    /// Test that async updates can be injected into the event loop
    /// from background tasks (e.g., miner responses).
    #[tokio::test]
    async fn async_response_received() {
        let (reader, _stream_tx) = MockEventStream::new();
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(60), reader);
        let update_tx = loop_handle.update_sender();

        assert_eq!(next_event(&mut loop_handle).await, Event::Tick);

        let update = UpdateEvent::Message("test".into());
        update_tx.send(update.clone()).expect("send failed");

        assert_eq!(next_event(&mut loop_handle).await, Event::Update(update));
    }

    #[tokio::test]
    async fn resize_event_handled() {
        let (reader, stream_tx) = MockEventStream::new();
        let mut loop_handle = EventLoop::with_stream(Duration::from_secs(60), reader);

        assert_eq!(next_event(&mut loop_handle).await, Event::Tick);

        stream_tx
            .send(Ok(CrosstermEvent::Resize(120, 40)))
            .expect("send failed");

        assert_eq!(next_event(&mut loop_handle).await, Event::Resize(120, 40));
    }

    /// Test that the update sender can be cloned and used from multiple
    /// background tasks concurrently.
    #[tokio::test]
    async fn update_sender_cloned() {
        let (reader, _stream_tx) = MockEventStream::new();
        let loop_handle = EventLoop::with_stream(Duration::from_secs(60), reader);
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
    }
}
