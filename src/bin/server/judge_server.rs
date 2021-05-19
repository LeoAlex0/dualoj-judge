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
    pub judge_id: String,
    pub api_key: String,
    /// Cancel signal, when reached, delete registry.
    pub cancel: oneshot::Receiver<()>,
    /// When get input from judger, trigger this.
    pub on_success: oneshot::Sender<TestResult>,
}

struct Key {
    api_key: String,
    signal_sender: oneshot::Sender<TestResult>,
}

pub(crate) struct JudgeServer {
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
            let name = msg.judge_id.clone();

            let mut new_list = job_list.lock().await;
            new_list.insert(
                msg.judge_id,
                Key {
                    api_key: msg.api_key,
                    signal_sender: msg.on_success,
                },
            );
            drop(new_list);
            info!("{} registered", name);

            let cancel = msg.cancel;
            task::spawn(async move {
                let e = cancel.await;
                if e.is_ok() {
                    warn!("{} cancelled", name);
                    let mut cur_map = cancel_handler.lock().await;
                    cur_map.remove(&name);
                } else {
                    error!("{} cancel canceled", name);
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
            if let Some((id, val)) = list.remove_entry(&req.judge_id) {
                if val.api_key == req.api_key {
                    task::spawn(async move {
                        let _ = val.signal_sender.send(req.result);
                    });
                    info!("{} judger posted", id);
                    JudgerResponse {
                        status: JudgerStatus::Ok.into(),
                    }
                } else {
                    list.insert(id, val);
                    JudgerResponse {
                        status: JudgerStatus::InvalidAuth.into(),
                    }
                }
            } else {
                JudgerResponse {
                    status: JudgerStatus::InvalidName.into(),
                }
            },
        ))
    }
}
