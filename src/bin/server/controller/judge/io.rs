use std::time::Duration;

use futures::future::{join, try_join};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, ListParams, Meta},
    Api,
};
use log::{error, info};
use tokio::{io::copy, time::timeout};

use crate::controller::judge::{JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME};

/// Bind pods' stdin & stdout of judger & solver
pub async fn io_binder(pods: Api<Pod>, job_name: String) {
    let label_selector = format!("job-name={}", job_name);
    info!(
        "{} listing label-selector: \"{}\"",
        job_name, label_selector
    );
    match timeout(
        // TODO!: Customize list timeout.
        Duration::from_millis(20),
        pods.list(&ListParams {
            label_selector: Some(label_selector),
            limit: Some(1),

            ..Default::default()
        }),
    )
    .await
    {
        Err(e) => error!("{} list timeout:{}", job_name, e),
        Ok(Err(e)) => error!("{} list fail: {}", job_name, e),
        Ok(Ok(pod_name)) => {
            if pod_name.items.len() == 1 {
                error!("Pod of Job:{} not found", job_name);
                return;
            }
            let pod_name = pod_name.items[0].name();
            let solver_ap = attach_param(SOLVER_CONTAINER_NAME);
            let judger_ap = attach_param(JUDGER_CONTAINER_NAME);
            let solver = pods.attach(pod_name.as_str(), &solver_ap);
            let judger = pods.attach(pod_name.as_str(), &judger_ap);

            info!("{} job located pod: {}, attaching", job_name, pod_name);
            // TODO!: Customize attach timeout.
            match timeout(Duration::from_millis(50), try_join(solver, judger)).await {
                Err(e) => info!("{} binding timeout: {}", job_name, e),
                Ok(Err(e)) => info!("{} attach fail: {}", job_name, e),
                Ok(Ok((mut _judged, mut _judger))) => {
                    if let (
                        Some(mut judged_in),
                        Some(mut judged_out),
                        Some(mut judger_in),
                        Some(mut judger_out),
                    ) = (
                        _judged.stdin(),
                        _judged.stdout(),
                        _judger.stdin(),
                        _judger.stdout(),
                    ) {
                        info!("{} copying stdin & stdout", job_name);
                        let _ = join(
                            copy(&mut judged_out, &mut judger_in),
                            copy(&mut judger_out, &mut judged_in),
                        )
                        .await;
                        info!("{} io_binder copy complete", job_name);
                    } else {
                        info!("{} take stdin/stdout fail", job_name);
                    }
                }
            }
        }
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
