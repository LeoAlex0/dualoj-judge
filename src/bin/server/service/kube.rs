use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::api::ObjectMeta;
use uuid::Uuid;

use super::FileService;

pub(crate) struct JudgeLimit {
    // CPU Limit, (in mili-cpu)
    pub cpu: u64,
    // Memory Limit, (in MiB)
    pub memory: u64,
    // Time Limit, (in seconds)
    pub time: i64,
}

impl FileService {
    fn get_image_url(&self, uuid: &Uuid) -> String {
        format!(
            "{}/{}/{}:latest",
            self.registry.url, self.registry.username, uuid
        )
    }
    pub(crate) fn generate_judge_job(
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
                                name: "judged".into(),
                                image: Some(self.get_image_url(&judged)),
                                image_pull_policy: Some("Always".into()),
                                resources: Some(ResourceRequirements {
                                    limits: Some(limits.clone()),
                                    requests: None,
                                }),

                                ..Default::default()
                            },
                            Container {
                                name: "judger".into(),
                                image: Some(self.get_image_url(&judger)),
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
