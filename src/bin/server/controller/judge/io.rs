use std::time::Duration;

use futures::{
    future::{ready, try_join},
    StreamExt,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, ListParams, Meta, WatchEvent},
    Api,
};
use log::{error, info};
use tokio::{io::copy, time::timeout};

use crate::controller::judge::{
    error::{JudgeError, ResultInspectErr},
    JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME,
};

/// Bind pods' stdin & stdout of judger & solver
pub async fn io_binder(pods: Api<Pod>, job_name: String) -> Result<(), JudgeError> {
    let pod_name = wait_for_pod(pods.clone(), &job_name).await?.name();

    let solver_ap = attach_param(SOLVER_CONTAINER_NAME);
    let judger_ap = attach_param(JUDGER_CONTAINER_NAME);
    let solver = pods.attach(pod_name.as_str(), &solver_ap);
    let judger = pods.attach(pod_name.as_str(), &judger_ap);

    info!("{} job located pod: {}, attaching", job_name, pod_name);
    // TODO!: Customize attach timeout.
    let (mut _judged, mut _judger) = timeout(Duration::from_millis(50), try_join(solver, judger))
        .await
        .inspect_err(|e| error!("{} binding timeout: {}", job_name, e))?
        .inspect_err(|e| error!("{} attach fail: {}", job_name, e))?;

    if let (Some(mut judged_in), Some(mut judged_out), Some(mut judger_in), Some(mut judger_out)) = (
        _judged.stdin(),
        _judged.stdout(),
        _judger.stdin(),
        _judger.stdout(),
    ) {
        info!("{} copying stdin & stdout", job_name);
        let copied = try_join(
            copy(&mut judged_out, &mut judger_in),
            copy(&mut judger_out, &mut judged_in),
        )
        .await?;
        info!(
            "{} io_binder copy complete: copied {:?} byte",
            job_name, copied
        );
        Ok(())
    } else {
        let err = JudgeError::IOBindingFail { job_name, pod_name };
        error!("{}", err);
        Err(err)
    }
}

fn attach_param(container_name: &str) -> AttachParams {
    AttachParams {
        container: Some(container_name.into()),
        stdin: true,
        stdout: true,

        ..Default::default()
    }
}

async fn wait_for_pod(pod_api: Api<Pod>, job_name: &str) -> Result<Pod, JudgeError> {
    let mut result: Vec<_> = pod_api
        .watch(
            &ListParams::default()
                .labels(&format!("job-name={}", job_name))
                //TODO!:  Customize TTL
                .timeout(1),
            "0",
        )
        .await?
        .boxed()
        .filter_map(|it| {
            ready(match it {
                Ok(WatchEvent::Added(pod)) => Some(pod),
                _ => None,
            })
        })
        .take(1)
        .collect()
        .await;

    result.pop().ok_or_else(|| JudgeError::PodJobNotFoundError {
        job_name: job_name.to_string(),
    })
}
