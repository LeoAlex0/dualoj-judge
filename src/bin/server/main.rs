use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, ListParams, Meta, PostParams, WatchEvent};
use kube::Client;

extern crate pretty_env_logger;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    pretty_env_logger::init();

    println!("Trying Default K8S Client");

    // Read the environment to find config for kube client.
    // Note that this tries an in-cluster configuration first,
    // then falls back on a kubeconfig file.
    let client = Client::try_default().await?;

    println!("K8S Client connected, try to get pods");

    // Get a strongly typed handle to the Kubernetes API for interacting
    // with pods in the "default" namespace.
    let pods: Api<Pod> = Api::namespaced(client, "dualoj");

    println!("PodList Get OK:");
    let pod_list = pods.list(&ListParams::default()).await?;
    pod_list.iter().for_each(|it| println!("{:?}", it));

    // Create a pod from JSON
    let pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "my-pod",
            "namespace": "dualoj"
        },
        "spec": {
            "containers": [
                {
                    "name": "my-container",
                    "image": "myregistry.azurecr.io/hello-world:v1",
                },
            ],
        }
    }))?;

    // Create the pod
    let _pod = pods.create(&PostParams::default(), &pod).await?;

    // Start a watch call for pods matching our name
    let lp = ListParams::default()
        .fields(&format!("metadata.name={}", "my-pod"))
        .timeout(10);
    let mut stream = pods.watch(&lp, "0").await?.boxed();

    // Observe the pods phase for 10 seconds
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(o) => println!("Added {}", Meta::name(&o)),
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                let phase = s.phase.clone().unwrap_or_default();
                println!("Modified: {} with phase: {}", Meta::name(&o), phase);
            }
            WatchEvent::Deleted(o) => println!("Deleted {}", Meta::name(&o)),
            WatchEvent::Error(e) => println!("Error {}", e),
            _ => {}
        }
    }

    Ok(())
}
