use std::fmt::Debug;

use futures::{
    channel::{mpsc, oneshot},
    future::join,
    SinkExt, StreamExt,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{ListParams, ResourceExt, WatchEvent},
    Api,
};
use log::{error, info, warn};
use tokio::task;

use super::SOLVER_CONTAINER_NAME;

pub struct ListenPodResult {
    /// sending pod when when pod running
    pub pod_running: oneshot::Receiver<Pod>,
    /// send when pod ended (Failed/Succeeded)
    pub pod_ended: oneshot::Receiver<Option<SolverStopReason>>,

    /// send when listen error (Dead before Running)
    pub listen_error: oneshot::Receiver<ListenStopReason>,
}

#[derive(Debug, PartialEq)]
pub enum SolverStopReason {
    Completed,
    OOMKilled,
    Error,

    Other,
}

#[derive(Debug, PartialEq)]
pub enum ListenStopReason {
    ImagePullBackOff,
    Error,
}

pub async fn pod_listener(
    pod_api: Api<Pod>,
    judge_id: &str,
) -> Result<ListenPodResult, kube::Error> {
    let (tx_start, rx_start) = oneshot::channel();
    let (tx_end, rx_end) = oneshot::channel();
    let (tx_error, rx_error) = oneshot::channel();
    let (tx_mul_start, rx_mul_start) = mpsc::channel(1);
    let (tx_mul_end, rx_mul_end) = mpsc::channel(1);

    let mut result = pod_api
        .watch(
            &ListParams::default()
                .labels(&format!("judge-id={}", judge_id))
                .disable_bookmarks(),
            "0",
        )
        .await?
        .boxed();

    let canceller = task::spawn(async move {
        while let Some(Ok(event)) = result.next().await {
            let mut run = tx_mul_start.clone();
            let mut end = tx_mul_end.clone();
            let delete = |reason: Option<SolverStopReason>| async move { end.send(reason).await };
            let running = |pod: Pod| async move { run.send(pod).await };
            match event {
                WatchEvent::Added(pod) => {
                    info!("{} pod created", pod.name_any());
                }
                WatchEvent::Modified(pod) => {
                    info!("{} current phase: {}", pod.name_any(), phase(&pod));
                    match phase(&pod).as_str() {
                        "Running" => {
                            task::spawn(running(pod.clone()));
                        }
                        "Failed" => {
                            task::spawn(delete(solver_exit_reason(&pod)));
                            break;
                        }
                        "Succeeded" => {
                            task::spawn(delete(solver_exit_reason(&pod)));
                            break;
                        }
                        _ => {}
                    };
                    if let Some(reason) = solver_exit_reason(&pod) {
                        // Judger may need some time to judge solver's output, so no reason to delete pod.
                        if reason != SolverStopReason::Completed {
                            info!("{} solver exited abnormal", pod.name_any());
                            task::spawn(delete(Some(reason)));
                            break;
                        }
                    }
                    if is_pull_image_error(&pod) {
                        error!("{} ImagePullOff", pod.name_any());
                        let _ = tx_error.send(ListenStopReason::ImagePullBackOff);
                        break;
                    }
                }
                WatchEvent::Deleted(pod) => {
                    info!("{} deleted. send delete signal", pod.name_any());
                    task::spawn(delete(solver_exit_reason(&pod)));
                    break;
                }
                WatchEvent::Error(e) => {
                    error!("Error occured, send delete signal: {}", e.to_string());
                    let _ = tx_error.send(ListenStopReason::Error);
                    break;
                }
                _ => {}
            }
        }
    });

    task::spawn(async move {
        join(
            forward_first(tx_start, rx_mul_start),
            forward_first(tx_end, rx_mul_end),
        )
        .await;
        canceller.abort();
    });
    Ok(ListenPodResult {
        pod_running: rx_start,
        pod_ended: rx_end,
        listen_error: rx_error,
    })
}

async fn forward_first<T: Debug>(tx: oneshot::Sender<T>, mut rx: mpsc::Receiver<T>) {
    if let Some(x) = rx.next().await {
        if let Err(x) = tx.send(x) {
            warn!("Signal forward failed: {:?}", x)
        }
    }
}

fn phase(pod: &Pod) -> String {
    if let Some(s) = pod.status.as_ref() {
        s.phase.clone().unwrap_or_default()
    } else {
        String::new()
    }
}

fn solver_exit_reason(pod: &Pod) -> Option<SolverStopReason> {
    let solver_terminated_state = pod
        .status
        .clone()?
        .container_statuses?
        .into_iter()
        .filter(|x| x.name == SOLVER_CONTAINER_NAME)
        .collect::<Vec<_>>()
        .pop()?
        .state?
        .terminated?;

    match solver_terminated_state.reason {
        Some(reason) => match reason.as_str() {
            "Completed" => Some(SolverStopReason::Completed),
            "OOMKilled" => Some(SolverStopReason::OOMKilled),
            "Error" => Some(SolverStopReason::Error),
            _ => Some(SolverStopReason::Other),
        },
        _ => Some(SolverStopReason::Other),
    }
}

fn is_pull_image_error(pod: &Pod) -> bool {
    let container_status = pod.status.clone().unwrap().container_statuses;

    if let Some(status) = container_status {
        status.into_iter().any(|s| {
            s.state
                .and_then(|x| {
                    x.waiting
                        .map(|wait| wait.reason.map(|reason| reason == "ImagePullBackOff"))
                })
                .flatten()
                .unwrap_or(false)
        })
    } else {
        false
    }
}
