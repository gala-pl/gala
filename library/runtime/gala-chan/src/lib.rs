//! Communication channels for the Gala hybrid runtime.
//!
//! Provides typed channels for passing messages between classical
//! and quantum execution contexts — used for mid-circuit measurement
//! feedback, backend dispatch, and shot management.

use gala_core::int::Int;
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;

/// A typed channel for sending values between runtime components.
pub struct Channel<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T: Send + 'static> Channel<T> {
    /// Creates a new channel pair.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> (ChannelSender<T>, ChannelReceiver<T>) {
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        (
            ChannelSender { sender: tx.clone() },
            ChannelReceiver { receiver },
        )
    }

    /// Sends a value through the channel.
    pub fn send(&self, value: T) -> Result<(), &'static str> {
        self.sender.send(value).map_err(|_| "channel closed")
    }

    /// Receives a value, blocking until one is available.
    pub fn recv(&self) -> Result<T, &'static str> {
        self.receiver
            .lock()
            .map_err(|_| "lock poisoned")?
            .recv()
            .map_err(|_| "channel closed")
    }

    /// Attempts to receive a value without blocking.
    pub fn try_recv(&self) -> Result<Option<T>, &'static str> {
        match self.receiver.lock().map_err(|_| "lock poisoned")?.try_recv() {
            Ok(val) => Ok(Some(val)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err("channel closed"),
        }
    }
}

/// Sender half of a channel.
pub struct ChannelSender<T> {
    sender: Sender<T>,
}

impl<T: Send> ChannelSender<T> {
    pub fn send(&self, value: T) -> Result<(), &'static str> {
        self.sender.send(value).map_err(|_| "channel closed")
    }
}

/// Receiver half of a channel.
pub struct ChannelReceiver<T> {
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> ChannelReceiver<T> {
    pub fn recv(&self) -> Result<T, &'static str> {
        self.receiver
            .lock()
            .map_err(|_| "lock poisoned")?
            .recv()
            .map_err(|_| "channel closed")
    }

    pub fn try_recv(&self) -> Result<Option<T>, &'static str> {
        match self.receiver.lock().map_err(|_| "lock poisoned")?.try_recv() {
            Ok(val) => Ok(Some(val)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err("channel closed"),
        }
    }
}

/// Quantum measurement result sent across channels.
#[derive(Debug, Clone)]
pub struct MeasurementResult {
    pub shot_index: Int,
    pub value: Int,
    pub basis: MeasurementBasis,
}

#[derive(Debug, Clone, Copy)]
pub enum MeasurementBasis {
    Computational,
    X,
    Y,
    Custom(f64, f64),
}

/// Shot batch to dispatch to a backend.
#[derive(Debug, Clone)]
pub struct ShotBatch {
    pub circuit_id: Int,
    pub num_shots: Int,
    pub params: Vec<f64>,
}

impl MeasurementResult {
    pub fn new(shot_index: Int, value: Int) -> Self {
        MeasurementResult {
            shot_index,
            value,
            basis: MeasurementBasis::Computational,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_send_recv() {
        let (tx, rx) = Channel::<Int>::new();
        tx.send(42).unwrap();
        let result = rx.recv().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_try_recv_empty() {
        let (_tx, rx) = Channel::<Int>::new();
        let result = rx.try_recv().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_separate_halves() {
        let (tx, rx) = Channel::<i64>::new();
        tx.send(99).unwrap();
        assert_eq!(rx.recv().unwrap(), 99);
    }
}
