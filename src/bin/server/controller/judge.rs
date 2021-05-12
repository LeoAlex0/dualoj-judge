use std::{collections::BTreeMap, future::ready, str::FromStr};

use dualoj_judge::proto::{
    self, controller_server::Controller, judge_event::Event, JobCreatedMsg, JobErrorMsg,
    JudgeEvent, JudgeLimit,
};
use futures::{
    channel::mpsc::{self, Sender},
    SinkExt, StreamExt,
};
use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, Pod, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
    Metadata,
};
use kube::{
    api::{AttachParams, ListParams, Meta, ObjectMeta, PostParams, WatchEvent},
    Api,
};
use log::warn;
use tokio::{io::copy, spawn, try_join};
use tonic::{Response, Status};

use super::ControlService;

const JUDGED_CONTAINER_NAME: &str = "judged";
const JUDGER_CONTAINER_NAME: &str = "judger";

impl ControlService {
    pub(crate) async fn new_judge_job(
        &self,
        limit: JudgeLimit,
        judged: uuid::Uuid,
        judger: uuid::Uuid,
    ) -> Result<Response<<ControlService as Controller>::JudgeStream>, Status> {
        let job = self.judge_job(limit, judged, judger);
        let (mut tx, rx) = mpsc::channel(20);

        let jobs: Api<Job> =
            Api::namespaced(self.k8s_client.clone(), self.pod_env.namespace.as_str());
        let pods: Api<Pod> =
            Api::namespaced(self.k8s_client.clone(), self.pod_env.namespace.as_str());

        tokio::spawn(fail_watcher(jobs.clone(), judger.to_string(), tx.clone()));
        match jobs.create(&PostParams::default(), &job).await {
            Ok(_) => {
                tokio::spawn(io_binder(pods, judger.to_string()));
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

    fn judge_job(
        &self,
        JudgeLimit { cpu, memory, time }: JudgeLimit,
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
                ttl_seconds_after_finished: Some(5), // TODO!: custom TTL for debugging
                template: PodTemplateSpec {
                    metadata: None,
                    spec: Some(PodSpec {
                        active_deadline_seconds: Some(time.into()),
                        containers: vec![
                            Container {
                                name: JUDGED_CONTAINER_NAME.into(),
                                image: Some(self.registry.get_image_url(&judged.to_string())),
                                image_pull_policy: Some("Always".into()),
                                resources: Some(ResourceRequirements {
                                    limits: Some(limits.clone()),
                                    requests: None,
                                }),

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

fn attach_param(container_name: &str) -> AttachParams {
    AttachParams {
        container: Some(container_name.into()),
        stdin: true,
        stdout: true,
        tty: true,

        ..Default::default()
    }
}

async fn io_binder(pods: Api<Pod>, job_name: String) {
    let pod_name = pods
        .list(&ListParams {
            label_selector: Some(format!("job-name={}", job_name)),
            timeout: Some(1),
            limit: Some(1),

            ..Default::default()
        })
        .await
        .unwrap();
    if pod_name.items.len() == 1 {
        warn!("Pod of Job:{} not found", job_name);
        return;
    }
    let pod_name = pod_name.items[0].name();
    let judged_ap = attach_param(JUDGED_CONTAINER_NAME);
    let judger_ap = attach_param(JUDGER_CONTAINER_NAME);
    let judged = pods.attach(pod_name.as_str(), &judged_ap);
    let judger = pods.attach(pod_name.as_str(), &judger_ap);
    if let Ok((mut _judged, mut _judger)) = try_join!(judged, judger) {
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
            spawn(async move {
                let _ = copy(&mut judged_out, &mut judger_in).await;
            });
            spawn(async move {
                let _ = copy(&mut judger_out, &mut judged_in).await;
            });
        }
    }
}

async fn fail_watcher(
    jobs: Api<Job>,
    name: String,
    event_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) {
    if let Ok(event_stream) = jobs
        .watch(
            &ListParams::default().fields(format!("metadata.name={}", name).as_str()),
            "0",
        )
        .await
    {
        let res = event_stream
            .take_while(|x| ready(x.is_ok()))
            .filter_map(|x| ready(x.ok()))
            .map(|x| async move {
                match x {
                    WatchEvent::Added(job) => job
                        .metadata()
                        .uid
                        .as_ref()
                        .map(|s| uuid::Uuid::from_str(s).ok())
                        .flatten()
                        .map(|uid| {
                            Event::Created(JobCreatedMsg {
                                job_uid: proto::Uuid {
                                    data: uid.as_bytes().to_vec(),
                                },
                            })
                        }),
                    WatchEvent::Modified(_) => None,
                    WatchEvent::Deleted(_) => None,
                    WatchEvent::Bookmark(_) => None,
                    WatchEvent::Error(e) => None,
                }
            });
        todo!()
    }
}
