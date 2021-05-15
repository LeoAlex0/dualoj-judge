use dualoj_judge::proto::JudgeEvent;
use futures::{
    channel::{
        mpsc::{self, Sender},
        oneshot,
    },
    SinkExt, StreamExt,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{ListParams, Meta, WatchEvent},
    Api,
};
use log::{error, info};
use tokio::{join, task, try_join};

use crate::{
    controller::judge::{error::ResultInspectErr, judger::register_judger_callback},
    judge_server::JudgeMsg,
};

use super::{bind, error::JudgeError};

pub(crate) async fn launch(
    pod_api: Api<Pod>,
    job_name: String,
    api_key: String,
    job_poster: Sender<JudgeMsg>,
    controller_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) -> Result<(), JudgeError> {
    let ListenPodResult {
        pod_create,
        pod_end,
    } = pod_listener(pod_api.clone(), &job_name).await?;

    let pod = pod_create
        .await
        .inspect_err(|e| error!("{} failed to watch pod start: {}", job_name, e))?;

    try_join!(
        bind::bind_io(pod_api, pod),
        register_judger_callback(job_name, api_key, pod_end, job_poster, controller_sender)
    )?;

    Ok(())
}

struct ListenPodResult {
    pod_create: oneshot::Receiver<Pod>,
    pod_end: oneshot::Receiver<()>,
}
async fn pod_listener(pod_api: Api<Pod>, job_name: &str) -> Result<ListenPodResult, kube::Error> {
    let (tx_start, rx_start) = oneshot::channel();
    let (tx_end, rx_end) = oneshot::channel();
    let (tx_mul_start, rx_mul_start) = mpsc::channel(1);
    let (tx_mul_end, rx_mul_end) = mpsc::channel(1);

    let mut result = pod_api
        .watch(
            &ListParams::default().labels(&format!("job-name={}", job_name)),
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
                    }
                }
                WatchEvent::Deleted(p) => {
                    info!("{} deleted. send delete signal", p.name());
                    let mut end = tx_mul_end.clone();
                    task::spawn(async move { end.send(()).await });
                }
                WatchEvent::Error(e) => {
                    error!("Error occured, send delete signal: {}", e.to_string());
                    let mut end = tx_mul_end.clone();
                    task::spawn(async move { end.send(()).await });
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

async fn forward_first<T>(tx: oneshot::Sender<T>, mut rx: mpsc::Receiver<T>) {
    if let Some(x) = rx.next().await {
        let _ = tx.send(x);
    }
}

fn is_running(pod: &Pod) -> bool {
    let s = pod.status.as_ref().expect("status exists on pod");
    let current = s.phase.clone().unwrap_or_default();
    info!("{} current phase: {}", pod.name(), current);
    current == "Running"
}
