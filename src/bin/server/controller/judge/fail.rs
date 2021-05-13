use std::str::FromStr;

use dualoj_judge::proto::{
    self, job_exit_msg::Code, judge_event::Event, JobCreatedMsg, JobExitMsg, JudgeEvent,
};
use futures::{channel::mpsc::Sender, future::ready, SinkExt, StreamExt};
use k8s_openapi::api::batch::v1::Job;
use kube::{
    api::{ListParams, WatchEvent},
    Api,
};
use log::{debug, warn};

pub(crate) async fn fail_watcher(
    jobs: Api<Job>,
    name: String,
    mut event_sender: Sender<Result<JudgeEvent, tonic::Status>>,
) {
    let field_selector = format!("metadata.name={}", name);
    debug!("{} fail watcher forked, watching: {}", name, field_selector);
    match jobs
        .watch(&ListParams::default().fields(&field_selector), "0")
        .await
    {
        Err(e) => warn!("{} watch fail: {}", name, e),
        Ok(event_stream) => {
            let mut res = event_stream
                .boxed()
                .take_while(|x| ready(x.is_ok()))
                .filter_map(|x| ready(x.ok()));

            debug!("{} fail-watcher get event stream OK", name);
            while let Some(x) = res.next().await {
                if let Some(event) = match x {
                    WatchEvent::Added(job) => job
                        .metadata
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
                    WatchEvent::Error(e) => Some(Event::Exit(JobExitMsg {
                        judge_code: Code::RuntimeError.into(),
                        other_msg: Some(e.message),
                    })),
                } {
                    let _ = event_sender
                        .send(Ok(JudgeEvent { event: Some(event) }))
                        .await;
                }
            }
        }
    }
}
