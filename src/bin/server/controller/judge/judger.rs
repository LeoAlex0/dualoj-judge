use dualoj_judge::proto::{judge_event::Event, JobExitMsg, JudgeEvent};
use futures::{
    channel::{mpsc::Sender, oneshot},
    SinkExt,
};
use log::warn;
use tokio::task;

use crate::judge_server::JudgeMsg;

use super::error::{JudgeError, ResultInspectErr};

pub(crate) async fn register_judger_callback(
    job_id: String,
    api_key: String,
    canceller: oneshot::Receiver<()>,
    mut job_poster: Sender<JudgeMsg>,
    mut controller_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) -> Result<(), JudgeError> {
    let (tx, rx) = oneshot::channel();
    let log_name = job_id.clone();

    task::spawn(async move {
        job_poster
            .send(JudgeMsg {
                name: job_id,
                api_key,
                ttl: None,
                success: tx,
                cancel: Some(canceller),
            })
            .await
    });

    if let Ok(result) = rx
        .await
        // When Pod exit, this will be canceled, not so serious.
        .inspect_err(|e| warn!("{} Judger canceled: {}", log_name, e))
    {
        let _ = controller_sender
            .send(Ok(JudgeEvent {
                event: Some(Event::Exit(JobExitMsg {
                    judge_code: result.code,
                    other_msg: result.other_msg,
                })),
            }))
            .await;
    }

    Ok(())
}
