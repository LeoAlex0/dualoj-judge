use std::time::Duration;

use futures::future::try_join;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{AttachParams, Meta},
    Api,
};
use log::{error, info, warn};
use tokio::{
    io::{self, copy, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    join,
    time::timeout,
};

use crate::controller::judge::{
    error::{JudgeError, ResultInspectErr},
    JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME,
};

/// Bind pods' stdin & stdout of judger & solver
pub async fn bind_io(pods: Api<Pod>, job_name: String, pod: Pod) -> Result<(), JudgeError> {
    let pod_name = pod.name();

    let solver_ap = attach_param(SOLVER_CONTAINER_NAME);
    let judger_ap = attach_param(JUDGER_CONTAINER_NAME);
    let solver = pods.attach(pod_name.as_str(), &solver_ap);
    let judger = pods.attach(pod_name.as_str(), &judger_ap);

    info!("{} job located pod: {}, attaching", job_name, pod_name);
    // TODO!: Customize attach timeout.
    // FIXME!: Binding timeout.
    let (mut solver, mut judger) = try_join(solver, judger)
        .await
        // .inspect_err(|e| error!("{} binding timeout: {}", job_name, e))?
        .inspect_err(|e| error!("{} attach fail: {}", job_name, e))?;

    info!("{} Attached", job_name);
    if let (Some(mut judged_in), Some(mut judged_out), Some(mut judger_in), Some(mut judger_out)) = (
        solver.stdin(),
        solver.stdout(),
        judger.stdin(),
        judger.stdout(),
    ) {
        info!("{} copying stdin & stdout", job_name);

        // TODO!: use copy instead of logged_copy
        // FIXME!: Cannot copy their stdin & stdout, use pod log instead of attach stdout.
        let copied = try_join(
            logged_copy(&mut judged_out, &mut judger_in),
            logged_copy(&mut judger_out, &mut judged_in),
        )
        .await?;
        info!(
            "{} io_binder copy complete: copied {:?} byte",
            job_name, copied
        );
        join!(solver, judger);
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
        stderr: false,
        tty: false,

        // max_stdin_buf_size: Some(0),
        // max_stdout_buf_size: Some(0),
        ..Default::default()
    }
}

#[warn(dead_code)]
async fn logged_copy<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> io::Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = [0; 5];
    let mut copied = 0;

    // use timeout to output an log
    while let Ok(size) = reader.read(&mut buf).await {
        if size == 0 {
            warn!("copied 0 byte,unknown reason");
            break;
        }
        copied += size;
        info!("copied: {}", String::from_utf8_lossy(&buf));
        let _ = writer.write_all(&buf[..size]).await;
    }
    warn!("copy complete");

    Ok(copied)
}
