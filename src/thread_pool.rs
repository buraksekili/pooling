use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::errors::ThreadPoolError;

pub enum Job {
    Task(Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static>),
    Shutdown,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, job_queue: Arc<SegQueue<Job>>) -> Worker {
        // TODO: can we use lazy initialization?
        // TODO: each tasks are executed sequentially by each worker.
        // maybe, we can consider async within a worker.
        let thread = thread::spawn(move || loop {
            match job_queue.pop() {
                Some(Job::Task(task)) => if let Err(e) = task() {},
                Some(Job::Shutdown) => {
                    break;
                }
                None => {
                    // TODO: This might be an issue (sleeping for a while to wait for a new job.)
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_queue: Arc<SegQueue<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let job_queue = Arc::new(SegQueue::new());
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&job_queue)));
        }

        ThreadPool { workers, job_queue }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        let job = Job::Task(Box::new(f));
        self.job_queue.push(job);
        Ok(())
    }

    pub fn shutdown(&mut self, timeout: Duration) -> Result<(), ThreadPoolError> {
        let start = Instant::now();

        // Signal all threads to shutdown
        for _ in &self.workers {
            self.job_queue.push(Job::Shutdown);
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let remaining = timeout
                    .checked_sub(start.elapsed())
                    .unwrap_or(Duration::ZERO);
                if remaining.is_zero() {
                    return Err(ThreadPoolError::ShutdownTimeout);
                }

                if thread.join().is_err() {
                    return Err(ThreadPoolError::ThreadJoinError(format!(
                        "Worker {} failed to join",
                        worker.id
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
        if !self.workers.is_empty() {
            eprintln!("ThreadPool dropped without calling shutdown. Forcing shutdown now.");
            let _ = self.shutdown(Duration::from_secs(2));
        }
    }
}
