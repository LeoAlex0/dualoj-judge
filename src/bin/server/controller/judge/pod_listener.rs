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

pub struct ListenPodResult {
    pub pod_create: oneshot::Receiver<Pod>,
    pub pod_end: oneshot::Receiver<()>,
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
            match event {
                WatchEvent::Added(pod) => {
                    info!("{} pod created", pod.name());
                    if is_running(&pod) {
                        let mut mul = tx_mul_start.clone();
                        task::spawn(async move { mul.send(pod).await });
                    }
                }
                WatchEvent::Modified(pod) => {
                    info!("{} pod changed", pod.name());
                    if is_running(&pod) {
                        let mut mul = tx_mul_start.clone();
                        task::spawn(async move { mul.send(pod).await });
                    } else if is_fail(&pod) {
                        let mut end = tx_mul_end.clone();
                        task::spawn(async move { end.send(()).await });
                        break;
                    }
                }
                WatchEvent::Deleted(p) => {
                    info!("{} deleted. send delete signal", p.name());
                    let mut end = tx_mul_end.clone();
                    task::spawn(async move { end.send(()).await });
                    break;
                }
                WatchEvent::Error(e) => {
                    error!("Error occured, send delete signal: {}", e.to_string());
                    let mut end = tx_mul_end.clone();
                    task::spawn(async move { end.send(()).await });
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
        pod_create: rx_start,
        pod_end: rx_end,
    })
}

async fn forward_first<T: Debug>(tx: oneshot::Sender<T>, mut rx: mpsc::Receiver<T>) {
    if let Some(x) = rx.next().await {
        if let Err(x) = tx.send(x) {
            warn!("Signal forward failed: {:?}", x)
        }
    }
}

fn is_running(pod: &Pod) -> bool {
    let s = pod.status.as_ref().expect("status exists on pod");
    let current = s.phase.clone().unwrap_or_default();
    info!("{} current phase: {}", pod.name(), current);
    current == "Running"
}

fn is_fail(pod: &Pod) -> bool {
    let s = pod.status.as_ref().expect("status exists on pod");
    let current = s.phase.clone().unwrap_or_default();
    info!("{} current phase: {}", pod.name(), current);
    current == "Failed"
}
