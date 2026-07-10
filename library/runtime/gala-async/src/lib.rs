//! Async runtime for Gala's hybrid execution model.
//!
//! Provides an async task system for coordinating classical computation
//! with quantum kernel dispatch, mid-circuit measurement feedback,
//! and shot batching — without blocking the classical orchestrator.

use gala_core::int::Int;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};

/// A lightweight async task handle.
pub struct Task<T: Send + 'static> {
    handle: Option<JoinHandle<T>>,
    cancel_flag: Arc<Mutex<bool>>,
}

impl<T: Send + 'static> Task<T> {
    /// Creates a new task from a closure.
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let cancel_flag = Arc::new(Mutex::new(false));
        let flag = cancel_flag.clone();
        let handle = thread::spawn(move || {
            if *flag.lock().unwrap() {
                panic!("task cancelled before start");
            }
            f()
        });
        Task {
            handle: Some(handle),
            cancel_flag,
        }
    }

    /// Waits for the task to complete and returns the result.
    pub fn join(&mut self) -> T {
        self.handle.take().unwrap().join().unwrap()
    }

    /// Requests cancellation of the task.
    pub fn cancel(&self) {
        *self.cancel_flag.lock().unwrap() = true;
    }
}

/// A simple executor for running coroutines.
pub struct Executor {
    tasks: Vec<Box<dyn FnOnce() + Send>>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor { tasks: Vec::new() }
    }

    /// Spawns a task on this executor.
    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tasks.push(Box::new(f));
    }

    /// Runs all spawned tasks to completion.
    pub fn run(self) {
        let handles: Vec<_> = self
            .tasks
            .into_iter()
            .map(thread::spawn)
            .collect();
        for h in handles {
            let _ = h.join();
        }
    }
}

/// Quantum kernel dispatch as an async operation.
pub struct KernelDispatch {
    kernel_id: Int,
    params: Vec<f64>,
    shots: Int,
}

impl KernelDispatch {
    pub fn new(kernel_id: Int, params: Vec<f64>, shots: Int) -> Self {
        KernelDispatch {
            kernel_id,
            params,
            shots,
        }
    }

    /// Returns a future-like object for awaiting the result.
    pub fn execute(self) -> Task<KernelResult> {
        Task::spawn(move || {
            // Simulated kernel execution: deterministic based on params hash
            let results: Vec<Int> = (0..self.shots)
                .map(|i| {
                    let hash = (self.kernel_id as f64 * 0.5 + self.params.iter().sum::<f64>() * i as f64) % 1.0;
                    if hash > 0.5 { 1 } else { 0 }
                })
                .collect();

            let mut histogram = std::collections::HashMap::new();
            for r in &results {
                *histogram.entry(*r).or_insert(0) += 1;
            }

            KernelResult {
                kernel_id: self.kernel_id,
                shots: self.shots,
                histogram,
            }
        })
    }
}

/// Result of a quantum kernel execution.
pub struct KernelResult {
    pub kernel_id: Int,
    pub shots: Int,
    pub histogram: std::collections::HashMap<Int, Int>,
}

/// Runtime statistics for monitoring.
pub struct RuntimeStats {
    pub tasks_spawned: Int,
    pub tasks_completed: Int,
    pub tasks_failed: Int,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeStats {
    pub fn new() -> Self {
        RuntimeStats {
            tasks_spawned: 0,
            tasks_completed: 0,
            tasks_failed: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_spawn_and_join() {
        let mut task = Task::spawn(|| 42);
        assert_eq!(task.join(), 42);
    }

    #[test]
    fn test_executor_run() {
        let mut ex = Executor::new();
        let result = Arc::new(Mutex::new(0i32));
        let r1 = result.clone();
        ex.spawn(move || {
            *r1.lock().unwrap() = 42;
        });
        ex.run();
        assert_eq!(*result.lock().unwrap(), 42);
    }

    #[test]
    fn test_kernel_dispatch() {
        let dispatch = KernelDispatch::new(1, vec![0.5, 0.3], 100);
        let mut task = dispatch.execute();
        let result = task.join();
        assert_eq!(result.kernel_id, 1);
        assert_eq!(result.shots, 100);
        let total: Int = result.histogram.values().sum();
        assert_eq!(total, 100);
    }
}
