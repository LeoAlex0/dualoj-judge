use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, Pod, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::{
    api::{AttachParams, ListParams, Meta, ObjectMeta, PostParams},
    Api,
};
use log::warn;
use tokio::{io::copy, spawn, try_join};
use uuid::Uuid;

use super::ControlService;

const JUDGED_CONTAINER_NAME: &str = "judged";
const JUDGER_CONTAINER_NAME: &str = "judger";

pub(crate) struct JudgeLimit {
    // CPU Limit, (in mili-cpu)
    pub cpu: u64,
    // Memory Limit, (in MiB)
    pub memory: u64,
    // Time Limit, (in seconds)
    pub time: i64,
}

impl ControlService {
    pub(crate) async fn new_judge_job(
        &self,
        limit: JudgeLimit,
        judged: Uuid,
        judger: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let job = self.judge_job(limit, judged, judger);

        let client = kube::client::Client::try_default().await?;
        let jobs: Api<Job> = Api::namespaced(client.clone(), self.pod_env.namespace.as_str());
        let pods: Api<Pod> = Api::namespaced(client, self.pod_env.namespace.as_str());

        let job = jobs.create(&PostParams::default(), &job).await?;

        tokio::spawn(io_binder(pods, judger.to_string()));

        Ok(())
    }

    fn judge_job(
        &self,
        JudgeLimit { cpu, memory, time }: JudgeLimit,
        judged: Uuid,
        judger: Uuid,
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
                template: PodTemplateSpec {
                    metadata: None,
                    spec: Some(PodSpec {
                        active_deadline_seconds: Some(time),
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
