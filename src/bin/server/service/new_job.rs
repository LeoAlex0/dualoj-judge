use std::{collections::BTreeMap, env};

use dualoj_judge::proto::{NewJobResponse, Uuid};
use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, PodSpec, PodTemplateSpec},
    },
    apimachinery::pkg::apis::meta::v1::OwnerReference,
};
use kube::{
    api::{ObjectMeta, PostParams},
    Api,
};
use tonic::{Request, Response, Status};

use super::FileService;

impl FileService {
    pub async fn new_job(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<dualoj_judge::proto::NewJobResponse>, Status> {
        let uuid = uuid::Uuid::from_slice(&request.get_ref().data)
            .map_err(|e| e.to_string())
            .map_err(Status::invalid_argument)?;

        let resp = match self.kube_newjob(uuid).await {
            Ok(_) => NewJobResponse {
                code: 0,
                ..Default::default()
            },
            Err(e) => NewJobResponse {
                code: 1,
                error_msg: Some(e.to_string()),
            },
        };

        Ok(Response::new(resp))
    }

    async fn kube_newjob(&self, uuid: uuid::Uuid) -> Result<(), kube::Error> {
        let client = kube::Client::try_default().await?;
        let uuid_str = uuid.to_string();

        // use of internal image, so we can use `latest` tag.
        let image_name = format!(
            "{}/{}/{}:latest",
            self.registry_url, self.registry_username, uuid_str
        );

        //TODO!: do not use hard-coded namespace, add this to structopt
        let namespace = env::var("POD_NAMESPACE").unwrap_or("dualoj".into());
        let pod_uid = env::var("POD_UID").unwrap_or_default();
        let pod_name = env::var("POD_NAME").unwrap_or_default();

        let jobs: Api<Job> = Api::namespaced(client, namespace.as_str());

        jobs.create(
            &PostParams::default(),
            &Job {
                metadata: ObjectMeta {
                    labels: Some({
                        let mut labels = BTreeMap::new();
                        labels.insert("app".into(), "judged".into());

                        labels
                    }),
                    name: Some(uuid_str.clone()),
                    namespace: Some(namespace),
                    owner_references: Some(vec![OwnerReference {
                        api_version: "v1".into(),
                        controller: Some(true),
                        kind: "Pod".into(),
                        name: pod_name,
                        uid: pod_uid,
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
                                name: uuid_str,
                                image: Some(image_name),
                                image_pull_policy: Some("Always".into()),

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

        Ok(())
    }
}
