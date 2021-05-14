use core::future::Future;
use std::{error::Error, fmt::Display};

use dualoj_judge::proto::{judge_event::Event, JobErrorMsg, JudgeEvent};
use futures::{
    channel::{mpsc::Sender, oneshot::Canceled},
    SinkExt,
};
use tokio::time::error::Elapsed;
use tonic::Status;

pub async fn wrap_error<F, T, E>(
    fut: F,
    mut event_sender: Sender<Result<JudgeEvent, Status>>,
) -> Option<T>
where
    F: Future<Output = Result<T, E>>,
    E: ToString,
{
    match fut.await {
        Ok(result) => Some(result),
        Err(e) => {
            // If error send error, ignore it.
            let _ = event_sender
                .send(Ok(JudgeEvent {
                    event: Some(Event::Error(JobErrorMsg { msg: e.to_string() })),
                }))
                .await;
            None
        }
    }
}

pub trait ResultInspectErr<F, E> {
    fn inspect_err(self, f: F) -> Self;
}

impl<F, T, E> ResultInspectErr<F, E> for Result<T, E>
where
    F: FnOnce(&E),
    E: Sized,
{
    fn inspect_err(self, f: F) -> Self {
        if let Err(ref e) = self {
            (f)(&e);
        }
        self
    }
}

#[derive(Debug)]
pub enum JudgeError {
    PodJobNotFoundError { job_name: String },
    IOBindingFail { job_name: String, pod_name: String },
    Log(String),
    KubeError(kube::Error),
    Timeout(Elapsed),
    IOError(std::io::Error),
    Canceled(Canceled),
}

impl Display for JudgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PodJobNotFoundError { job_name } => {
                write!(f, "Pod of job {} not found", job_name)
            }
            JudgeError::IOBindingFail { job_name, pod_name } => {
                write!(f, "Pod {} of job {} IO binding fail", pod_name, job_name)
            }
            Self::KubeError(e) => e.fmt(f),
            Self::Timeout(e) => e.fmt(f),
            Self::IOError(e) => e.fmt(f),
            Self::Canceled(e) => e.fmt(f),
            JudgeError::Log(s) => s.fmt(f),
        }
    }
}

impl Error for JudgeError {}

impl From<kube::Error> for JudgeError {
    fn from(e: kube::Error) -> Self {
        JudgeError::KubeError(e)
    }
}

impl From<Elapsed> for JudgeError {
    fn from(e: Elapsed) -> Self {
        JudgeError::Timeout(e)
    }
}

impl From<std::io::Error> for JudgeError {
    fn from(e: std::io::Error) -> Self {
        JudgeError::IOError(e)
    }
}

impl From<Canceled> for JudgeError {
    fn from(e: Canceled) -> Self {
        JudgeError::Canceled(e)
    }
}
