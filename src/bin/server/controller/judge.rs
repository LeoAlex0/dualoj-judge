mod fail;
mod io;

use std::{collections::BTreeMap, time::Duration};

use dualoj_judge::proto::{
    controller_server::Controller, judge_event::Event, JobErrorMsg, JobExitMsg, JudgeEvent,
    JudgeLimit,
};
use futures::{
    channel::mpsc::{self, Sender},
    SinkExt,
};
use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, EnvVar, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::api::{Meta, ObjectMeta, PostParams};
use log::warn;
use tokio::{sync::oneshot, time::timeout};
use tonic::{Response, Status};

use crate::judge_server::JudgeMsg;

use super::ControlService;

const SOLVER_CONTAINER_NAME: &str = "judged";
const JUDGER_CONTAINER_NAME: &str = "judger";

impl ControlService {
    pub(crate) async fn new_judge_job(
        &self,
        limit: JudgeLimit,
        judged: uuid::Uuid,
        judger: uuid::Uuid,
    ) -> Result<Response<<ControlService as Controller>::JudgeStream>, Status> {
        let inject_apikey = uuid::Uuid::new_v4();
        let ttl = Duration::from_secs(limit.time.into());
        let job = self.judge_job(limit, &inject_apikey, judged, judger);
        let job_name = job.name();
        let (mut tx, rx) = mpsc::channel(20);

        // fail watcher & judge result watcher
        tokio::spawn(tokio::time::timeout(
            ttl,
            fail::fail_watcher(self.job_api.clone(), job_name.clone(), tx.clone()),
        ));
        tokio::spawn(register_judger_callback(
            judged.to_string(),
            inject_apikey.to_string(),
            ttl,
            self.job_poster.clone(),
            tx.clone(),
        ));
        match self.job_api.create(&PostParams::default(), &job).await {
            Ok(_) => {
                tokio::spawn(io::io_binder(self.pod_api.clone(), job_name));
            }
            Err(e) => {
                let _ = tx
                    .send(Ok(JudgeEvent {
                        event: Some(Event::Error(JobErrorMsg { msg: e.to_string() })),
                    }))
                    .await;
            }
        }

        Ok(Response::new(rx))
    }

    // TODO!: split away from ControllerService
    fn judge_job(
        &self,
        JudgeLimit { cpu, memory, time }: JudgeLimit,
        apikey: &uuid::Uuid,
        judged: uuid::Uuid,
        judger: uuid::Uuid,
    ) -> Job {
        let mut limits = BTreeMap::new();
        limits.insert("cpu".into(), Quantity(format!("{}m", cpu)));
        limits.insert("memory".into(), Quantity(format!("{}Mi", memory)));

        Job {
            metadata: ObjectMeta {
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("app".into(), "judged".into());

                    labels
                }),
                name: Some(judged.to_string()),
                namespace: Some(self.pod_env.namespace.clone()),
                owner_references: Some(vec![OwnerReference {
                    api_version: "v1".into(),
                    controller: Some(true),
                    kind: "Pod".into(),
                    name: self.pod_env.name.clone(),
                    uid: self.pod_env.uid.clone(),
                    ..Default::default()
                }]),

                ..Default::default()
            },
            spec: Some(JobSpec {
                backoff_limit: Some(0),
                // ttl_seconds_after_finished: Some(5), // TODO!: custom TTL for debugging
                template: PodTemplateSpec {
                    metadata: None,
                    spec: Some(PodSpec {
                        active_deadline_seconds: Some(time.into()),
                        containers: vec![
                            Container {
                                name: SOLVER_CONTAINER_NAME.into(),
                                image: Some(self.registry.get_image_url(&judged.to_string())),
                                image_pull_policy: Some("Always".into()),
                                resources: Some(ResourceRequirements {
                                    limits: Some(limits.clone()),
                                    requests: None,
                                }),
                                stdin: Some(true),

                                ..Default::default()
                            },
                            Container {
                                name: JUDGER_CONTAINER_NAME.into(),
                                image: Some(self.registry.get_image_url(&judger.to_string())),
                                image_pull_policy: Some("Always".into()),
                                resources: Some(ResourceRequirements {
                                    limits: Some(limits),
                                    requests: None,
                                }),
                                stdin: Some(true),

                                // inject judge env
                                env: Some(vec![
                                    EnvVar {
                                        name: "APIKEY".into(),
                                        value: Some(apikey.to_string()),
                                        value_from: None,
                                    },
                                    EnvVar {
                                        name: "JOB_ID".into(),
                                        value: Some(judged.to_string()),
                                        value_from: None,
                                    },
                                    EnvVar {
                                        name: "JUDGER_ADDR".into(),
                                        // TODO!: use service & k8s network-policy instead of using ip directly.
                                        value: Some(self.judger_addr.to_string()),
                                        value_from: None,
                                    },
                                ]),

                                ..Default::default()
                            },
                        ],
                        restart_policy: Some("Never".into()),
                        share_process_namespace: Some(true),
                        termination_grace_period_seconds: Some(0),

                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

async fn register_judger_callback(
    job_id: String,
    api_key: String,
    ttl: Duration,
    mut job_poster: Sender<JudgeMsg>,
    mut controller_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) {
    let (tx, rx) = oneshot::channel();
    let log_name = job_id.clone();

    tokio::spawn(async move {
        job_poster
            .send(JudgeMsg {
                name: job_id,
                api_key,
                ttl,
                signal_sender: tx,
            })
            .await
    });

    match timeout(ttl, rx).await {
        Err(e) => warn!("{} Accepted signal timeout:{}", log_name, e),
        Ok(Err(e)) => warn!("{} Cannot get Signal from judger: {}", log_name, e),
        Ok(Ok(result)) => {
            let _ = controller_sender
                .send(Ok(JudgeEvent {
                    event: Some(Event::Exit(JobExitMsg {
                        judge_code: result.code,
                        other_msg: result.other_msg,
                    })),
                }))
                .await;
        }
    }
}
