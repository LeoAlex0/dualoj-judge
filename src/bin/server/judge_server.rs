use std::{collections::HashMap, sync::Arc};

use dualoj_judge::proto::judger::{
    judger_response::JudgerStatus, judger_server::Judger, JudgerRequest, JudgerResponse, TestResult,
};
use futures::{
    channel::{mpsc, oneshot},
    StreamExt,
};
use log::{error, info, warn};
use tokio::{
    sync::Mutex,
    task::{self, JoinHandle},
};
use tonic::{Request, Response, Status};

pub(crate) struct JudgeMsg {
    /// For log
    pub judge_id: String,
    pub token: String,
    /// Cancel signal, when reached, delete registry.
    pub cancel: oneshot::Receiver<()>,
    /// When get input from judger, trigger this.
    pub on_success: oneshot::Sender<TestResult>,
}

struct Key {
    judge_id: String,
    signal_sender: oneshot::Sender<TestResult>,
}

pub(crate) struct JudgeServer {
    // TODO: use redis
    job_list: Arc<Mutex<HashMap<String, Key>>>,
    daemon_handler: JoinHandle<()>,
}

impl JudgeServer {
    pub fn new(receive: mpsc::Receiver<JudgeMsg>) -> Self {
        let job_list = Arc::new(Mutex::new(HashMap::new()));
        JudgeServer {
            job_list: job_list.clone(),
            daemon_handler: task::spawn(receive_daemon(job_list, receive)),
        }
    }
}

impl Drop for JudgeServer {
    fn drop(&mut self) {
        self.daemon_handler.abort();
    }
}

async fn receive_daemon(
    job_list: Arc<Mutex<HashMap<String, Key>>>,
    request_receiver: mpsc::Receiver<JudgeMsg>,
) {
    request_receiver
        .for_each(|msg| async {
            let cancel_handler = job_list.clone();

            let mut new_list = job_list.lock().await;
            new_list.insert(
                msg.token.clone(),
                Key {
                    judge_id: msg.judge_id.clone(),
                    signal_sender: msg.on_success,
                },
            );
            drop(new_list);
            info!("{} registered", msg.judge_id);

            let cancel = msg.cancel;
            let token = msg.token;
            task::spawn(async move {
                let e = cancel.await;
                if e.is_ok() {
                    warn!("{} cancelled", msg.judge_id);
                    let mut cur_map = cancel_handler.lock().await;
                    cur_map.remove(&token);
                } else {
                    error!("{} cancel canceled", msg.judge_id);
                }
            });
        })
        .await;
}

#[tonic::async_trait]
impl Judger for JudgeServer {
    async fn post_test_result(
        &self,
        request: Request<JudgerRequest>,
    ) -> Result<Response<JudgerResponse>, Status> {
        let req = request.into_inner();
        let mut list = self.job_list.lock().await;
        Ok(Response::new(
            if let Some((_, info)) = list.remove_entry(&req.token) {
                task::spawn(async move {
                    let _ = info.signal_sender.send(req.result);
                });
                info!("{} judger posted", info.judge_id);
                JudgerResponse {
                    status: JudgerStatus::Ok.into(),
                }
            } else {
                JudgerResponse {
                    status: JudgerStatus::InvalidAuth.into(),
                }
            },
        ))
    }
}
