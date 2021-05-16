use std::future::ready;

use futures::{future::try_join, StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, LogParams, Meta},
    Api,
};
use log::{error, info, warn};
use tokio::io::copy;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::controller::judge::{error::JudgeError, JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME};

/// Bind pods' stdin & stdout of judger & solver
pub async fn bind_io(pods: Api<Pod>, pod: Pod) -> Result<(), JudgeError> {
    let pod_name = pod.name();

    info!("{} pod io binding", pod_name);

    let copied = try_join(
        log_binder(
            pods.clone(),
            &pod_name,
            JUDGER_CONTAINER_NAME.into(),
            SOLVER_CONTAINER_NAME.into(),
        ),
        log_binder(
            pods.clone(),
            &pod_name,
            SOLVER_CONTAINER_NAME.into(),
            JUDGER_CONTAINER_NAME.into(),
        ),
    )
    .await?;

    info!("{} copied {:?} bytes", pod_name, copied);
    Ok(())
}

async fn log_binder(
    pods: Api<Pod>,
    pod_name: &String,
    from: String,
    to: String,
) -> Result<u64, JudgeError> {
    let log_param = LogParams {
        container: Some(from.clone()),
        follow: true,
        pretty: false,

        ..Default::default()
    };
    let attach_param = AttachParams {
        container: Some(to.clone()),
        stdin: true,
        stdout: false,
        stderr: false,
        tty: false,

        ..Default::default()
    };

    let log_stream = pods.log_stream(pod_name, &log_param);
    let attach_pod = pods.attach(pod_name, &attach_param);

    info!("{} pod {} -> {} attaching & logging", pod_name, from, to);
    let (log_stream, mut attach_pod) = try_join(log_stream, attach_pod).await.map_err(|e| {
        let msg = format!("{} -> {} attach/log error: {}", from, to, e);
        error!("{} pod {}", pod_name, msg);
        JudgeError::Log(msg)
    })?;

    info!("{} pod {} -> {} attached, coping", pod_name, from, to);
    let mut stdin = attach_pod.stdin().unwrap();
    let mut log_stream = log_stream
        .take_while(|it| ready(it.is_ok()))
        .filter_map(|it| ready(it.ok()))
        .map(Ok)
        .into_async_read()
        .compat();

    let copied = copy(&mut log_stream, &mut stdin).await.map_err(|e| {
        let msg = format!("{} -> {} closed: {}", from, to, e);
        warn!("{} pod {}", pod_name, msg);
        JudgeError::Log(msg)
    })?;

    Ok(copied)
}
