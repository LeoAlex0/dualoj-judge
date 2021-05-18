use std::{net::SocketAddr, option::Option, time::Duration};

use crate::{
    cli::pod_env,
    controller::judge::{bind::bind_io, pod_listener::ListenStopReason},
    judge_server::JudgeMsg,
};
use dualoj_judge::proto::{
    job_exit_msg::Code, judge_event::Event, judger::TestResult, JobCreatedMsg, JobErrorMsg,
    JobExitMsg, JudgeLimit,
};
use futures::{
    channel::{mpsc::Sender, oneshot},
    future::try_join,
    FutureExt, SinkExt,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, api::{DeleteParams, PostParams, ResourceExt}};
use log::{error, info};
use tokio::{
    select,
    task::{self, JoinHandle},
};

use super::{
    error::wrap_error,
    judger::{set_judge_server, JudgeIO},
    manifest::judge_pod,
    pod_listener::{pod_listener, SolverStopReason},
};

pub(crate) struct Judge {
    secure: JudgeSecure,
    pod_data: Pod,
    judger_api: Sender<JudgeMsg>,
    ttl: Duration,
    pod_api: Api<Pod>,
    transfer: Sender<Event>,

    handlers: Vec<JoinHandle<()>>,

    on_receive: Option<oneshot::Receiver<TestResult>>,
    server_canceller: Option<oneshot::Sender<()>>,
}

pub struct JudgeEnv {
    /// For indicating reverse owner relationship.
    pub(crate) pod_env: pod_env::Param,
    /// For injecting judge-server address.
    pub server_addr: SocketAddr,
}

pub struct JudgePodParam {
    /// CPU limit (in mili-cpu)
    pub cpu_limit: u32,
    /// Memory limit (in MiB)
    pub mem_limit: u32,
    /// Solver image url
    pub solver_image: String,
    /// Judger image url
    pub judger_image: String,
}

pub struct JudgeSecure {
    /// ID of an judge. generate randomly.
    pub judge_id: String,
    /// APIKEY for judger to use, to avoid solver commit Accepted itself.
    pub apikey: String,
}

impl Judge {
    pub(crate) fn new(
        pod_api: Api<Pod>,
        env: JudgeEnv,
        judger_api: Sender<JudgeMsg>,
        transfer: Sender<Event>,
        solver_image: String,
        judger_image: String,
        limit: JudgeLimit,
    ) -> Self {
        let secure = JudgeSecure::new();
        let param = JudgePodParam {
            cpu_limit: limit.cpu,
            mem_limit: limit.memory,
            solver_image,
            judger_image,
        };
        let pod_data = judge_pod(&env, &secure, &param);

        Judge {
            secure,
            pod_data,
            judger_api,
            pod_api,
            ttl: Duration::from_secs(limit.time.into()),
            transfer,
            handlers: Vec::new(),

            on_receive: None,
            server_canceller: None,
        }
    }

    fn on_running(&mut self) -> JoinHandle<()> {
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();

        self.on_receive = Some(rx1);
        self.server_canceller = Some(tx2);

        task::spawn(
            wrap_error(
                self.transfer.clone(),
                try_join(
                    bind_io(self.pod_api.clone(), self.pod_data.clone()),
                    set_judge_server(
                        self.judger_api.clone(),
                        self.secure.judge_id.clone(),
                        self.secure.apikey.clone(),
                        JudgeIO {
                            on_receive: tx1,
                            canceller: rx2,
                        },
                    ),
                ),
            )
            .map(|_| ()),
        )
    }

    fn on_timeout(&self) -> JoinHandle<()> {
        let mut trans = self.transfer.clone();
        task::spawn(async move {
            let mut exit_msg = JobExitMsg::default();
            exit_msg.set_judge_code(Code::TimeLimitExceeded);
            let _ = trans.send(Event::Exit(exit_msg)).await;
        })
    }

    fn on_killed(&self, reason: Option<SolverStopReason>) -> JoinHandle<()> {
        let mut transfer = self.transfer.clone();
        info!("{} exit reason: {:?}", self.pod_data.name(), reason);

        task::spawn(async move {
            let mut exit_msg = JobExitMsg::default();
            if reason == Some(SolverStopReason::OOMKilled) {
                exit_msg.set_judge_code(Code::MemoryLimitExceeded);
            } else {
                exit_msg.set_judge_code(Code::RuntimeError);
            }
            transfer.send(Event::Exit(exit_msg)).await.unwrap();
        })
    }

    fn on_error(&self, reason: ListenStopReason) -> JoinHandle<()> {
        let mut transfer = self.transfer.clone();
        error!(
            "{} error: exit before running: {:?}",
            self.pod_data.name(),
            reason
        );

        task::spawn(async move {
            let mut error_msg = JobErrorMsg::default();
            use ListenStopReason::*;

            match reason {
                ImagePullBackOff => error_msg.msg = "ImagePullBackOff".into(),
                Error => error_msg.msg = "Listen error".into(),
            }
            transfer.send(Event::Error(error_msg)).await.unwrap();
        })
    }

    fn on_receive(&self, result: TestResult) -> JoinHandle<()> {
        let mut trans = self.transfer.clone();
        task::spawn(async move {
            let mut exit_msg = JobExitMsg::default();
            exit_msg.judge_code = result.code;
            exit_msg.other_msg = result.other_msg;

            let _ = trans.send(Event::Exit(exit_msg)).await;
        })
    }

    fn on_created(&self) -> JoinHandle<()> {
        let uid = self.pod_data.metadata.uid.clone();
        let mut trans = self.transfer.clone();
        task::spawn(async move {
            let _ = trans
                .send(Event::Created(JobCreatedMsg {
                    job_uid: uid.unwrap(),
                }))
                .await;
        })
    }

    pub(crate) async fn invoke(mut self) {
        let pod_api = self.pod_api.clone();

        self.invoke_inner().await;

        for handle in self.handlers {
            handle.abort();
        }
        if let Some(canceller) = self.server_canceller {
            let _ = canceller.send(());
        }
        let name = self.pod_data.name();
        info!("{} deleting pod", name);
        let _ = pod_api.delete(&name, &DeleteParams::default()).await;
        info!("{} pod deleted", name);
    }

    async fn invoke_inner(&mut self) -> Option<()> {
        if let Some(signals) = wrap_error(
            self.transfer.clone(),
            pod_listener(self.pod_api.clone(), &self.secure.judge_id),
        )
        .await
        {
            // update pod.name;
            info!("{} waiting for pod adding", self.secure.judge_id);
            self.pod_data = self
                .pod_api
                .create(&PostParams::default(), &self.pod_data)
                .await
                .ok()?;
            info!(
                "{} pod {} created",
                self.secure.judge_id,
                self.pod_data.name()
            );
            let handler = self.on_created();
            self.handlers.push(handler);

            info!("{} waiting for pod running", self.secure.judge_id);

            select! {
                Ok(pod_cur) = signals.pod_running => {
                    self.pod_data = pod_cur;
                    info!("{} pod running", self.secure.judge_id);

                    let handler = self.on_running();
                    self.handlers.push(handler);

                    let on_receive = self.on_receive.take()?;
                    select! {
                        _ = tokio::time::sleep(self.ttl) => {
                            let canceller = self.server_canceller.take()?;
                            let _ = canceller.send(());
                            let _ = self.on_timeout().await;
                        },
                        Ok(reason) = signals.pod_ended => {
                            let _ = self.on_killed(reason).await;
                        },
                        Ok(result) = on_receive => {
                            let _ = self.on_receive(result).await;
                        },
                    }
                },
                Ok(reason) = signals.listen_error => {
                    let _ = self.on_error(reason).await;
                }
            }
        }
        Some(())
    }
}

impl JudgeSecure {
    pub fn new() -> Self {
        JudgeSecure {
            judge_id: uuid::Uuid::new_v4().to_string(),
            apikey: uuid::Uuid::new_v4().to_string(),
        }
    }
}
