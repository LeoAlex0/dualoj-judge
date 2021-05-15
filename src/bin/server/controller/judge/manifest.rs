use std::{collections::BTreeMap, net::SocketAddr};

use dualoj_judge::proto::JudgeLimit;
use k8s_openapi::{
    api::core::v1::{Container, EnvVar, Pod, PodSpec, ResourceRequirements},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::api::ObjectMeta;

use crate::{
    cli::{pod_env, registry},
    controller::judge::{JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME},
};

/// Generate Job for judge.
pub(crate) fn judge_pod(
    pod_env: &pod_env::Param,
    registry: &registry::Param,
    judger_addr: &SocketAddr,
    JudgeLimit {
        cpu,
        memory,
        time: _e,
    }: JudgeLimit,
    apikey: String,
    judge_id: String,
    judged: uuid::Uuid,
    judger: uuid::Uuid,
) -> Pod {
    let mut limits = BTreeMap::new();
    limits.insert("cpu".into(), Quantity(format!("{}m", cpu)));
    limits.insert("memory".into(), Quantity(format!("{}Mi", memory)));

    Pod {
        metadata: ObjectMeta {
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app".into(), "judged".into());
                labels.insert("judge-id".into(), judge_id.clone());

                labels
            }),
            generate_name: Some("judged".into()),
            // name: Some(judged.to_string()),
            owner_references: Some(vec![OwnerReference {
                api_version: "v1".into(),
                controller: Some(true),
                kind: "Pod".into(),
                name: pod_env.name.clone(),
                uid: pod_env.uid.clone(),
                ..Default::default()
            }]),

            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![
                Container {
                    name: SOLVER_CONTAINER_NAME.into(),
                    image: Some(registry.get_image_url(&judged.to_string())),
                    image_pull_policy: Some("Always".into()),
                    resources: Some(ResourceRequirements {
                        limits: Some(limits.clone()),
                        requests: None,
                    }),
                    stdin_once: Some(true),
                    stdin: Some(true),

                    ..Default::default()
                },
                Container {
                    name: JUDGER_CONTAINER_NAME.into(),
                    image: Some(registry.get_image_url(&judger.to_string())),
                    image_pull_policy: Some("Always".into()),
                    resources: Some(ResourceRequirements {
                        limits: Some(limits),
                        requests: None,
                    }),
                    stdin_once: Some(true),
                    stdin: Some(true),

                    // inject judge env
                    env: Some(vec![
                        EnvVar {
                            name: "APIKEY".into(),
                            value: Some(apikey),
                            value_from: None,
                        },
                        EnvVar {
                            name: "JUDGE_ID".into(),
                            value: Some(judge_id),
                            value_from: None,
                        },
                        EnvVar {
                            name: "JUDGER_ADDR".into(),
                            // TODO!: use service & k8s network-policy instead of using ip directly.
                            value: Some(judger_addr.to_string()),
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
        ..Default::default()
    }
}
