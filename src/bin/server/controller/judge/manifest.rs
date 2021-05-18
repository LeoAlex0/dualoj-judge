use std::collections::BTreeMap;

use k8s_openapi::{
    api::core::v1::{Container, EnvVar, Pod, PodSpec, ResourceRequirements},
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::OwnerReference},
};
use kube::api::ObjectMeta;

use crate::controller::judge::{JUDGER_CONTAINER_NAME, SOLVER_CONTAINER_NAME};

use super::judge::{JudgeEnv, JudgePodParam, JudgeSecure};

/// Generate Job for judge.
pub(crate) fn judge_pod(env: &JudgeEnv, secure: &JudgeSecure, param: &JudgePodParam) -> Pod {
    let mut limits = BTreeMap::new();
    limits.insert("cpu".into(), Quantity(format!("{}m", param.cpu_limit)));
    limits.insert("memory".into(), Quantity(format!("{}Mi", param.mem_limit)));

    Pod {
        metadata: ObjectMeta {
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app".into(), "judged".into());
                labels.insert("judge-id".into(), secure.judge_id.clone());

                labels
            }),
            generate_name: Some("judged".into()),
            owner_references: Some(vec![OwnerReference {
                api_version: "v1".into(),
                controller: Some(true),
                kind: "Pod".into(),
                name: env.pod_env.name.clone(),
                uid: env.pod_env.uid.clone(),
                ..Default::default()
            }]),

            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![
                Container {
                    name: SOLVER_CONTAINER_NAME.into(),
                    image: Some(param.solver_image.clone()),
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
                    image: Some(param.judger_image.clone()),
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
                            value: Some(secure.apikey.clone()),
                            value_from: None,
                        },
                        EnvVar {
                            name: "JUDGE_ID".into(),
                            value: Some(secure.judge_id.clone()),
                            value_from: None,
                        },
                        EnvVar {
                            name: "JUDGER_ADDR".into(),
                            value: Some(env.server_addr.to_string()),
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
