//! generic background task handle for all TUI panels

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

/// wraps background thread with a stop signal and event channel
pub struct TaskHandle<E: Send + 'static> {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    rx: mpsc::Receiver<E>,
}

impl<E: Send + 'static> TaskHandle<E> {
    pub fn new(stop: Arc<AtomicBool>, thread: JoinHandle<()>, rx: mpsc::Receiver<E>) -> Self {
        Self { stop, thread, rx }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }

    pub fn try_recv(&self) -> Option<E> {
        self.rx.try_recv().ok()
    }
}
