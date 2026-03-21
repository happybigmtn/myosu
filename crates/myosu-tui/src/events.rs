use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, Stream, StreamExt, stream::BoxStream};
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

type TerminalEventStream = BoxStream<'static, io::Result<CrosstermEvent>>;

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

    fn from_stream<S>(tick_rate: Duration, reader: S) -> Self
    where
        S: Stream<Item = io::Result<CrosstermEvent>> + Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let (update_tx, mut update_rx) = mpsc::unbounded_channel::<UpdateEvent>();
        let mut reader: TerminalEventStream = reader.boxed();

        let _task = tokio::spawn(async move {
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
                                let Some(event) = map_crossterm_event(evt) else {
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

fn map_crossterm_event(evt: CrosstermEvent) -> Option<Event> {
    match evt {
        CrosstermEvent::Key(key) => Some(Event::Key(key)),
        CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
        CrosstermEvent::FocusGained => None,
        CrosstermEvent::FocusLost => None,
        CrosstermEvent::Mouse(_) => None,
        CrosstermEvent::Paste(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use futures::stream;

    async fn next_non_tick(loop_handle: &mut EventLoop) -> Event {
        for _ in 0..8 {
            let event = tokio::time::timeout(Duration::from_millis(100), loop_handle.next())
                .await
                .expect("timeout waiting for event")
                .expect("event loop closed");

            if event != Event::Tick {
                return event;
            }
        }

        panic!("did not receive non-tick event before deadline");
    }

    #[tokio::test]
    async fn tick_event_handled_headless() {
        let mut loop_handle = EventLoop::from_stream(Duration::from_millis(10), stream::pending());

        let event = tokio::time::timeout(Duration::from_millis(100), loop_handle.next())
            .await
            .expect("timeout waiting for event")
            .expect("event loop closed");

        assert_eq!(event, Event::Tick);
    }

    #[tokio::test]
    async fn key_event_handled_headless() {
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let stream = stream::iter(vec![Ok(CrosstermEvent::Key(key))]).chain(stream::pending());
        let mut loop_handle = EventLoop::from_stream(Duration::from_millis(10), stream);

        let event = next_non_tick(&mut loop_handle).await;

        assert_eq!(event, Event::Key(key));
    }

    #[tokio::test]
    async fn resize_event_handled_headless() {
        let stream =
            stream::iter(vec![Ok(CrosstermEvent::Resize(120, 42))]).chain(stream::pending());
        let mut loop_handle = EventLoop::from_stream(Duration::from_millis(10), stream);

        let event = next_non_tick(&mut loop_handle).await;

        assert_eq!(event, Event::Resize(120, 42));
    }

    #[tokio::test]
    async fn async_response_received_headless() {
        let mut loop_handle = EventLoop::from_stream(Duration::from_millis(10), stream::pending());
        let update_tx = loop_handle.update_sender();

        let update = UpdateEvent::Message("test".into());
        update_tx.send(update.clone()).expect("send failed");

        let event = next_non_tick(&mut loop_handle).await;

        assert_eq!(event, Event::Update(update));
    }

    /// Test that the update sender can be cloned and used from multiple
    /// background tasks concurrently.
    #[tokio::test]
    async fn update_sender_cloned() {
        let loop_handle = EventLoop::from_stream(Duration::from_millis(10), stream::pending());
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
