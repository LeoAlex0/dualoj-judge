use std::{collections::HashMap, sync::Arc, time::Duration};

use dualoj_judge::proto::judger::{
    judger_response::JudgerStatus, judger_server::Judger, JudgerResponse, TestResult,
};
use futures::{channel::mpsc, FutureExt, StreamExt};
use tokio::sync::{oneshot, Mutex};
use tonic::{Request, Response, Status};

pub(crate) struct JudgeMsg {
    pub name: String,
    pub api_key: String,
    pub ttl: Duration,
    pub signal_sender: oneshot::Sender<()>,
}

struct Key {
    api_key: String,
    signal_sender: oneshot::Sender<()>,
}

pub(crate) struct JudgeServer {
    job_list: Arc<Mutex<HashMap<String, Key>>>,
}

impl JudgeServer {
    pub fn new(receive: mpsc::Receiver<JudgeMsg>) -> Self {
        let job_list = Arc::new(Mutex::new(HashMap::new()));
        tokio::spawn(receive_daemon(job_list.clone(), receive));
        JudgeServer { job_list }
    }
}

async fn receive_daemon(
    job_list: Arc<Mutex<HashMap<String, Key>>>,
    request_receiver: mpsc::Receiver<JudgeMsg>,
) {
    request_receiver
        .for_each(|msg| async {
            let ttl_handler = job_list.clone();
            let name = msg.name.clone();
            let ttl = msg.ttl;

            let mut new_list = job_list.lock().await;
            new_list.insert(
                msg.name,
                Key {
                    api_key: msg.api_key,
                    signal_sender: msg.signal_sender,
                },
            );
            drop(new_list);

            // When TTL reached
            tokio::spawn(tokio::time::sleep(ttl).then(|_| async move {
                let mut cur_map = ttl_handler.lock().await;
                cur_map.remove(&name);
            }));
        })
        .await;
}

#[tonic::async_trait]
impl Judger for JudgeServer {
    async fn post_test_result(
        &self,
        request: Request<TestResult>,
    ) -> Result<Response<JudgerResponse>, Status> {
        let req = request.into_inner();
        let mut list = self.job_list.lock().await;
        Ok(Response::new(
            if let Some((id, val)) = list.remove_entry(&req.job_id) {
                if val.api_key == req.api_key {
                    tokio::spawn(async move {
                        let _ = val.signal_sender.send(());
                    });
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
