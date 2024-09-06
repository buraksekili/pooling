use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadPoolError {
    #[error("Failed to send job to worker thread")]
    JobSendError,
    #[error("Thread pool is shutting down")]
    ShuttingDown,
    #[error("Timeout while waiting for threads to finish")]
    ShutdownTimeout,
    #[error("Failed to join worker thread: {0}")]
    ThreadJoinError(String),
}

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job execution failed: {0}")]
    ExecutionError(String),
}
