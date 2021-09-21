use std::collections::BTreeMap;

use dualoj_judge::{
    id::gen::ID,
    proto::{new_job_response, NewJobResponse},
};
use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, Pod, PodSpec, PodTemplateSpec, ResourceRequirements},
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::{
    api::{ListParams, ObjectMeta, PostParams},
    Api,
};
use tonic::{Request, Response, Status};

use super::ControlService;

impl ControlService {
    pub async fn new_job(
        &self,
        request: Request<ID>,
    ) -> Result<Response<dualoj_judge::proto::NewJobResponse>, Status> {
        let req = request.into_inner();
        let resp = match self.kube_newjob(&req.content).await {
            Ok(uid) => NewJobResponse {
                code: 0,
                result: Some(new_job_response::Result::JobUid(ID::from(uid))),
            },
            Err(e) => NewJobResponse {
                code: 1,
                result: Some(new_job_response::Result::ErrorMsg(e.to_string())),
            },
        };

        Ok(Response::new(resp))
    }

    async fn kube_newjob(&self, uuid: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = kube::Client::try_default().await?;

        // use of internal image, so we can use `latest` tag.
        let image_name = self.registry.pull_url(uuid);

        let jobs: Api<Job> = Api::namespaced(client.clone(), self.pod_env.namespace.as_str());
        let pods: Api<Pod> = Api::namespaced(client, self.pod_env.namespace.as_str());

        let limits = {
            let mut limits = BTreeMap::new();
            limits.insert("cpu".into(), Quantity("2000m".into())); // TODO!: Scalable cpu limit
            limits.insert("memory".into(), Quantity("512Mi".into())); // TODO!: Scalable memory limit

            limits
        };

        let created_job = jobs
            .create(
                &PostParams::default(),
                &Job {
                    metadata: ObjectMeta {
                        labels: Some({
                            let mut labels = BTreeMap::new();
                            labels.insert("app".into(), "judged".into());

                            labels
                        }),
                        name: Some(uuid.to_string()),
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
                                active_deadline_seconds: Some(5), // TODO!: Scalable time limit
                                containers: vec![Container {
                                    name: "judged".into(),
                                    image: Some(image_name),
                                    image_pull_policy: Some("Always".into()),
                                    resources: Some(ResourceRequirements {
                                        limits: Some(limits),
                                        requests: None,
                                    }),

                                    ..Default::default()
                                }],
                                restart_policy: Some("Never".into()),
                                share_process_namespace: Some(true),
                                termination_grace_period_seconds: Some(0),

                                ..Default::default()
                            }),
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;

        let _pod = pods
            .list(&ListParams {
                label_selector: Some(format!("job-name={}", uuid)),
                timeout: Some(1),
                limit: Some(1),

                ..Default::default()
            })
            .await;

        let res = created_job.metadata.uid.unwrap_or_default();
        Ok(res)
    }
}
