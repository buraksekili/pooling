use crossbeam_channel::{bounded, Receiver, RecvTimeoutError, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::errors::ThreadPoolError;

type Job = Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Receiver<Job>, shutdown_flag: Arc<AtomicBool>) -> Worker {
        let thread = thread::spawn(move || {
            while !shutdown_flag.load(Ordering::Relaxed) {
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(job) => {
                        if let Err(e) = job() {
                            eprintln!("Worker {}: Job error: {}", id, e);
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            println!("Worker {} shutting down", id);
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = bounded(size * 2);
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                receiver.clone(),
                Arc::clone(&shutdown_flag),
            ));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            shutdown_flag,
        }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        if self.shutdown_flag.load(Ordering::Relaxed) {
            return Err(ThreadPoolError::ShuttingDown);
        }

        let job = Box::new(f);
        self.sender
            .as_ref()
            .ok_or(ThreadPoolError::ShuttingDown)?
            .send(job)
            .map_err(|_| ThreadPoolError::JobSendError)
    }

    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), ThreadPoolError> {
        // Signal all threads to shutdown
        self.shutdown_flag.store(true, Ordering::Relaxed);

        // Drop the sender to close the channel
        self.sender.take();

        let start = Instant::now();
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let remaining = timeout
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::ZERO);
                if remaining.is_zero() {
                    return Err(ThreadPoolError::ShutdownTimeout);
                }

                if let Err(e) = thread.join() {
                    return Err(ThreadPoolError::ThreadJoinError(format!(
                        "Worker {} failed to join: {:?}",
                        worker.id, e
                    )));
                }
            }
        }

        if start.elapsed() > timeout {
            Err(ThreadPoolError::ShutdownTimeout)
        } else {
            Ok(())
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if !self.shutdown_flag.load(Ordering::Relaxed) {
            eprintln!("ThreadPool dropped without calling shutdown. Forcing shutdown now.");
            let _ = self.shutdown(Duration::from_secs(2));
        }
    }
}
