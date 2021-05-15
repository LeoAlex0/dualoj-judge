use std::{str::FromStr, time::Duration};

use futures::{channel::mpsc::Sender, future::try_join, SinkExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{DeleteParams, Meta},
    Api,
};
use log::error;
use tokio::{task, time::timeout};

use crate::{
    controller::judge::{
        error::ResultInspectErr,
        judger::register_judger_callback,
        pod_listener::{pod_listener, ListenPodResult},
    },
    judge_server::JudgeMsg,
};
use dualoj_judge::proto::{
    self, job_exit_msg::Code, judge_event::Event, JobCreatedMsg, JobExitMsg, JudgeEvent,
};

use super::{bind, error::JudgeError};

pub(crate) async fn launch(
    pod_api: Api<Pod>,
    judge_id: String,
    api_key: String,
    ttl: Duration,
    job_poster: Sender<JudgeMsg>,
    mut controller_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) -> Result<(), JudgeError> {
    let ListenPodResult {
        pod_create,
        pod_end,
    } = pod_listener(pod_api.clone(), &judge_id).await?;

    let pod = pod_create
        .await
        .inspect_err(|e| error!("{} failed to watch pod start: {}", judge_id, e))?;
    let pod_name = pod.name();

    let mut sender1 = controller_sender.clone();
    let uid = pod.meta().uid.clone();
    task::spawn(async move {
        let uid_str = uid.unwrap();
        let uuid = uuid::Uuid::from_str(&uid_str).unwrap();
        let event = Event::Created(JobCreatedMsg {
            job_uid: proto::Uuid {
                data: uuid.as_bytes().to_vec(),
            },
        });
        let _ = sender1.send(Ok(JudgeEvent { event: Some(event) })).await;
    });

    let ac_receiver = register_judger_callback(
        judge_id,
        api_key,
        pod_end,
        job_poster,
        controller_sender.clone(),
    );
    let result = timeout(
        ttl,
        try_join(bind::bind_io(pod_api.clone(), pod), ac_receiver),
    )
    .await;

    // TLE or closed already, clean pods.
    task::spawn(async move {
        let dp = DeleteParams::default();
        pod_api.delete(&pod_name, &dp).await
    });

    match result {
        Err(_) => {
            // Send TLE signal.
            let mut exit_msg = JobExitMsg::default();
            exit_msg.set_judge_code(Code::TimeLimitExceeded);

            let _ = controller_sender
                .send(Ok(JudgeEvent {
                    event: Some(Event::Exit(exit_msg)),
                }))
                .await;
        }
        Ok(e) => {
            e?;
        }
    }

    Ok(())
}
