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
            let mut run = tx_mul_start.clone();
            let mut end = tx_mul_end.clone();
            let delete = async move { end.send(()).await };
            let running = |pod: Pod| async move { run.send(pod).await };
            match event {
                WatchEvent::Added(pod) => {
                    info!("{} pod created", pod.name());
                }
                WatchEvent::Modified(pod) => {
                    if let Some(phase) = pod.status.clone().unwrap().phase {
                        info!("{} current phase: {}", pod.name(), phase);
                    }
                    match phase(&pod).as_str() {
                        "Running" => {
                            task::spawn(running(pod));
                        }
                        "Failed" => {
                            task::spawn(delete);
                        }
                        "Succeeded" => {
                            task::spawn(delete);
                        }
                        _ => {}
                    };
                }
                WatchEvent::Deleted(p) => {
                    info!("{} deleted. send delete signal", p.name());
                    task::spawn(delete);
                    break;
                }
                WatchEvent::Error(e) => {
                    error!("Error occured, send delete signal: {}", e.to_string());
                    task::spawn(delete);
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

fn phase(pod: &Pod) -> String {
    if let Some(s) = pod.status.as_ref() {
        s.phase.clone().unwrap_or_default()
    } else {
        String::new()
    }
}
