use std::fmt::Debug;

use futures::{
    channel::{mpsc, oneshot},
    SinkExt, StreamExt,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{ListParams, Meta, WatchEvent},
    Api,
};
use log::{error, info, warn};
use tokio::{join, task};

use super::SOLVER_CONTAINER_NAME;

pub struct ListenPodResult {
    /// sending pod when when pod running
    pub pod_running: oneshot::Receiver<Pod>,
    /// send when pod ended (Failed/Succeeded)
    pub pod_ended: oneshot::Receiver<Option<SolverStopReason>>,
}

#[derive(Debug, PartialEq)]
pub enum SolverStopReason {
    Completed,
    OOMKilled,
    Error,

    Other,
}

pub async fn pod_listener(
    pod_api: Api<Pod>,
    judge_id: &str,
) -> Result<ListenPodResult, kube::Error> {
    let (tx_start, rx_start) = oneshot::channel();
    let (tx_end, rx_end) = oneshot::channel();
    let (tx_mul_start, rx_mul_start) = mpsc::channel(1);
    let (tx_mul_end, rx_mul_end) = mpsc::channel(1);

    let mut result = pod_api
        .watch(
            &ListParams::default().labels(&format!("judge-id={}", judge_id)),
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
                    info!("{} pod created", pod.name());
                }
                WatchEvent::Modified(pod) => {
                    info!("{} current phase: {}", pod.name(), phase(&pod));
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
                            info!("{} solver exited abnormal", pod.name());
                            task::spawn(delete(Some(reason)));
                            break;
                        }
                    }
                }
                WatchEvent::Deleted(pod) => {
                    info!("{} deleted. send delete signal", pod.name());
                    task::spawn(delete(solver_exit_reason(&pod)));
                    break;
                }
                WatchEvent::Error(e) => {
                    error!("Error occured, send delete signal: {}", e.to_string());
                    task::spawn(delete(None));
                    break;
                }
                _ => {}
            }
        }
    });

    task::spawn(async move {
        join!(
            forward_first(tx_start, rx_mul_start),
            forward_first(tx_end, rx_mul_end),
        );
        canceller.abort();
    });
    Ok(ListenPodResult {
        pod_running: rx_start,
        pod_ended: rx_end,
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
