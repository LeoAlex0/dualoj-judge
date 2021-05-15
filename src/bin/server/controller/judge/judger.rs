use dualoj_judge::proto::judger::TestResult;
use futures::{
    channel::{mpsc::Sender, oneshot},
    SinkExt,
};

use crate::judge_server::JudgeMsg;

use super::error::JudgeError;

pub(crate) struct JudgeIO {
    pub canceller: oneshot::Receiver<()>,
    pub on_receive: oneshot::Sender<TestResult>,
}

pub(crate) async fn set_judge_server(
    mut job_poster: Sender<JudgeMsg>,
    judge_id: String,
    api_key: String,
    io: JudgeIO,
) -> Result<(), JudgeError> {
    job_poster
        .send(JudgeMsg {
            name: judge_id,
            api_key,
            ttl: None,
            on_success: io.on_receive,
            cancel: Some(io.canceller),
        })
        .await
        .unwrap();

    Ok(())
}
