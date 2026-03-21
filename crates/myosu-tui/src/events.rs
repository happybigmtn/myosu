use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use std::{io, time::Duration};
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
    /// ```
    /// use std::time::Duration;
    /// use myosu_tui::events::EventLoop;
    ///
    /// # tokio_test::block_on(async {
    /// let event_loop = EventLoop::new(Duration::from_millis(16));
    /// # });
    /// ```
    pub fn new(tick_rate: Duration) -> Self {
        Self::from_stream(tick_rate, crossterm::event::EventStream::new())
    }

    fn from_stream<S>(tick_rate: Duration, event_stream: S) -> Self
    where
        S: futures::Stream<Item = io::Result<CrosstermEvent>> + Send + Unpin + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let (update_tx, mut update_rx) = mpsc::unbounded_channel::<UpdateEvent>();

        let _task = tokio::spawn(async move {
            let mut reader = event_stream;
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
        });

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
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.rx.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use futures::Stream;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    struct MockEventStream {
        rx: mpsc::UnboundedReceiver<io::Result<CrosstermEvent>>,
    }

    impl MockEventStream {
        fn new() -> (
            mpsc::UnboundedSender<io::Result<CrosstermEvent>>,
            MockEventStream,
        ) {
            let (tx, rx) = mpsc::unbounded_channel();
            (tx, Self { rx })
        }
    }

    impl Stream for MockEventStream {
        type Item = io::Result<CrosstermEvent>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.rx.poll_recv(cx)
        }
    }

    fn headless_event_loop(
        tick_rate: Duration,
    ) -> (EventLoop, mpsc::UnboundedSender<io::Result<CrosstermEvent>>) {
        let (stream_tx, stream) = MockEventStream::new();
        (EventLoop::from_stream(tick_rate, stream), stream_tx)
    }

    #[tokio::test]
    async fn headless_stream_delivers_tick_key_resize_and_update() {
        let (mut loop_handle, stream_tx) = headless_event_loop(Duration::from_millis(10));
        let update_tx = loop_handle.update_sender();
        let expected_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::SHIFT);

        stream_tx
            .send(Ok(CrosstermEvent::Key(expected_key)))
            .expect("send key event");
        stream_tx
            .send(Ok(CrosstermEvent::Resize(120, 40)))
            .expect("send resize event");
        update_tx
            .send(UpdateEvent::Message("headless update".into()))
            .expect("send update event");

        let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
        let mut saw_tick = false;
        let mut saw_key = false;
        let mut saw_resize = false;
        let mut saw_update = false;

        while tokio::time::Instant::now() < deadline
            && !(saw_tick && saw_key && saw_resize && saw_update)
        {
            let timeout = deadline - tokio::time::Instant::now();
            let event = tokio::time::timeout(timeout, loop_handle.next())
                .await
                .expect("timeout waiting for event")
                .expect("event loop closed");

            match event {
                Event::Tick => saw_tick = true,
                Event::Key(key) => saw_key = key == expected_key,
                Event::Resize(width, height) => saw_resize = (width, height) == (120, 40),
                Event::Update(UpdateEvent::Message(message)) => {
                    saw_update = message == "headless update"
                }
                _ => {}
            }
        }

        assert!(saw_tick, "did not receive tick event");
        assert!(saw_key, "did not receive key event");
        assert!(saw_resize, "did not receive resize event");
        assert!(saw_update, "did not receive injected update");
    }

    #[tokio::test]
    async fn async_response_received() {
        let (mut loop_handle, _stream_tx) = headless_event_loop(Duration::from_secs(1));
        let update_tx = loop_handle.update_sender();

        let update = UpdateEvent::Message("test".into());
        update_tx.send(update.clone()).expect("send failed");

        let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
        let mut found = false;

        while tokio::time::Instant::now() < deadline {
            let timeout = deadline - tokio::time::Instant::now();
            if let Ok(Some(event)) = tokio::time::timeout(timeout, loop_handle.next()).await {
                if let Event::Update(UpdateEvent::Message(msg)) = &event {
                    assert_eq!(msg, "test");
                    found = true;
                    break;
                }
            }
        }

        assert!(found, "did not receive injected update");
    }

    /// Test that the update sender can be cloned and used from multiple
    /// background tasks concurrently.
    #[tokio::test]
    async fn update_sender_cloned() {
        let (loop_handle, _stream_tx) = headless_event_loop(Duration::from_secs(1));
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
