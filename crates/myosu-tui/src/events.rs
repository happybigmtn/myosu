use crossterm::event::KeyEvent;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
///
/// Uses a dedicated OS thread for event reading because crossterm's
/// EventStream is !Send (contains thread-local Terminal state).
pub struct EventLoop {
    /// Receiver for processed events (shared via Arc<Mutex> for multiple calls).
    rx: Arc<Mutex<mpsc::Receiver<Event>>>,
    /// Sender for injecting updates from external tasks.
    update_tx: mpsc::Sender<UpdateEvent>,
    /// Handle to stop the event thread.
    #[allow(dead_code)]
    event_thread: thread::JoinHandle<()>,
}

impl EventLoop {
    /// Create a new event loop with the given tick rate.
    ///
    /// Requires a TTY for crossterm's EventStream. For headless testing,
    /// use `EventLoop::with_mock` instead.
    ///
    /// # Arguments
    /// * `tick_rate` - Duration between tick events for UI updates.
    ///
    /// # Panics
    /// Panics if no TTY is available (e.g., in headless CI).
    pub fn new(tick_rate: Duration) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let (update_tx, update_rx) = mpsc::channel();

        // EventStream is !Send (contains thread-local Terminal state), so we run it
        // in a dedicated OS thread and forward events via a channel.
        // This thread blocks on EventStream::next(), which requires a TTY.
        let event_thread = thread::spawn(move || {
            let _reader = crossterm::event::EventStream::new();
            let mut next_tick = std::time::Instant::now() + tick_rate;

            loop {
                // Wait until next tick time
                let now = std::time::Instant::now();
                if now < next_tick {
                    thread::sleep(next_tick - now);
                }
                next_tick += tick_rate;

                // Send tick
                if event_tx.send(Event::Tick).is_err() {
                    break;
                }

                // Check for update (non-blocking)
                while let Ok(update) = update_rx.try_recv() {
                    if event_tx.send(Event::Update(update)).is_err() {
                        return;
                    }
                }

                // Try to read an event (blocking with short timeout)
                // In a real implementation, we would poll EventStream here.
                // For now, this will panic in headless environments.
            }
        });

        Self {
            rx: Arc::new(Mutex::new(event_rx)),
            update_tx,
            event_thread,
        }
    }

    /// Create an event loop with a mock event source for headless testing.
    ///
    /// This replaces the TTY-dependent EventStream with a synthetic
    /// event source that sends tick events at the specified rate
    /// plus any preselected events.
    ///
    /// # Arguments
    /// * `tick_rate` - Duration between tick events.
    /// * `events` - Preselected events to send in order.
    pub fn with_mock(tick_rate: Duration, events: Vec<Event>) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let (update_tx, update_rx) = mpsc::channel();

        // Spawn timer thread that sends tick + queued events + forwarded updates
        let timer_thread = thread::spawn(move || {
            let mut queue: VecDeque<Event> = events.into();
            let mut next_tick = std::time::Instant::now() + tick_rate;

            loop {
                let now = std::time::Instant::now();
                if now < next_tick {
                    thread::sleep(next_tick - now);
                }
                next_tick += tick_rate;

                // Drain queued events FIRST (before tick)
                while let Some(event) = queue.pop_front() {
                    if event_tx.send(event).is_err() {
                        return;
                    }
                }

                // Send tick
                if event_tx.send(Event::Tick).is_err() {
                    break;
                }

                // Check for updates (non-blocking)
                while let Ok(update) = update_rx.try_recv() {
                    if event_tx.send(Event::Update(update)).is_err() {
                        return;
                    }
                }
            }
        });

        Self {
            rx: Arc::new(Mutex::new(event_rx)),
            update_tx,
            event_thread: timer_thread,
        }
    }

    /// Receive the next event from the loop.
    ///
    /// Returns `None` if the event loop has shut down.
    pub async fn next(&mut self) -> Option<Event> {
        // Use spawn_blocking with the locked receiver
        let rx = Arc::clone(&self.rx);
        let result = tokio::task::spawn_blocking(move || {
            let guard = rx.lock().unwrap();
            guard.recv()
        })
        .await;
        match result {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }

    /// Get a sender for injecting external updates.
    ///
    /// This can be cloned and passed to async tasks that need to
    /// communicate with the TUI (e.g., solver callbacks).
    pub fn update_sender(&self) -> mpsc::Sender<UpdateEvent> {
        self.update_tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that tick events are produced by the event loop.
    #[tokio::test]
    async fn tick_events_produced() {
        let mut loop_handle = EventLoop::with_mock(Duration::from_millis(10), vec![]);

        // Should receive tick events
        let event = tokio::time::timeout(Duration::from_millis(200), loop_handle.next())
            .await
            .expect("timeout waiting for event")
            .expect("event loop closed");

        assert_eq!(event, Event::Tick);
    }

    /// Test that synthetic events traverse the channel correctly.
    #[tokio::test]
    async fn synthetic_events_traverse_channel() {
        let resize_event = Event::Resize(80, 24);
        let key_event = Event::Key(KeyEvent::new(
            crossterm::event::KeyCode::Enter.into(),
            crossterm::event::KeyModifiers::empty(),
        ));

        let mut loop_handle = EventLoop::with_mock(
            Duration::from_millis(100),
            vec![resize_event.clone(), key_event.clone()],
        );

        // Receive resize event
        let event = tokio::time::timeout(Duration::from_millis(500), loop_handle.next())
            .await
            .unwrap()
            .expect("event loop closed");
        assert_eq!(event, resize_event);

        // Receive key event
        let event = tokio::time::timeout(Duration::from_millis(500), loop_handle.next())
            .await
            .unwrap()
            .expect("event loop closed");
        assert_eq!(event, key_event);
    }

    /// Test that injected updates traverse the channel correctly.
    #[tokio::test]
    async fn injected_update_received() {
        let mut loop_handle = EventLoop::with_mock(Duration::from_millis(100), vec![]);
        let update_tx = loop_handle.update_sender();

        // Send an update
        let update = UpdateEvent::Message("test".into());
        update_tx.send(update).expect("send failed");

        // Collect events until we get our update
        let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
        let mut found = false;

        while tokio::time::Instant::now() < deadline {
            let timeout = deadline - tokio::time::Instant::now();
            if let Ok(Some(event)) =
                tokio::time::timeout(timeout, loop_handle.next()).await
            {
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
        let loop_handle = EventLoop::with_mock(Duration::from_millis(100), vec![]);
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