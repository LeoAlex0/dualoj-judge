mod bind;
mod error;
mod judger;
mod manifest;
mod pod_listener;
mod post_pod;
mod watch;

use std::time::Duration;

use self::error::wrap_error;
use super::ControlService;
use dualoj_judge::proto::{controller_server::Controller, JudgeLimit};

use futures::channel::mpsc;
use kube::api::PostParams;
use tokio::task;
use tonic::{Response, Status};

const SOLVER_CONTAINER_NAME: &str = "judged";
const JUDGER_CONTAINER_NAME: &str = "judger";

impl ControlService {
    pub(crate) async fn new_judge_job(
        &self,
        limit: JudgeLimit,
        judged: uuid::Uuid,
        judger: uuid::Uuid,
    ) -> Result<Response<<ControlService as Controller>::JudgeStream>, Status> {
        let apikey = uuid::Uuid::new_v4();
        let judge_id = uuid::Uuid::new_v4();
        let ttl = Duration::from_secs(limit.time.into());
        let pod = manifest::judge_pod(
            &self.pod_env,
            &self.registry,
            &self.judger_addr,
            limit,
            apikey.to_string(),
            judge_id.to_string(),
            judged,
            judger,
        );
        let (tx, rx) = mpsc::channel(20);

        // watch job & judge result watcher

        wrap_error(
            self.pod_api.create(&PostParams::default(), &pod),
            tx.clone(),
        )
        .await;

        task::spawn(error::wrap_error(
            post_pod::launch(
                self.pod_api.clone(),
                judge_id.to_string(),
                apikey.to_string(),
                ttl,
                self.job_poster.clone(),
                tx.clone(),
            ),
            tx,
        ));

        Ok(Response::new(rx))
    }
}
