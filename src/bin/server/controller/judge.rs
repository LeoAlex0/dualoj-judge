mod bind;
mod error;
mod judger;
mod metadata;
mod post_pod;
mod watch;

use std::time::Duration;

use self::error::wrap_error;
use super::ControlService;
use dualoj_judge::proto::{controller_server::Controller, JudgeLimit};

use futures::channel::mpsc;
use kube::api::{Meta, PostParams};
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
        let inject_apikey = uuid::Uuid::new_v4();
        let ttl = Duration::from_secs(limit.time.into());
        let job = metadata::judge_job(
            &self.pod_env,
            &self.registry,
            &self.judger_addr,
            limit,
            &inject_apikey,
            judged,
            judger,
        );
        let job_name = job.name();
        let (tx, rx) = mpsc::channel(20);

        // watch job & judge result watcher
        task::spawn(tokio::time::timeout(
            ttl,
            error::wrap_error(
                watch::watch_job(self.job_api.clone(), job_name.clone(), tx.clone()),
                tx.clone(),
            ),
        ));

        wrap_error(
            self.job_api.create(&PostParams::default(), &job),
            tx.clone(),
        )
        .await;

        task::spawn(error::wrap_error(
            post_pod::launch(
                self.pod_api.clone(),
                job_name,
                inject_apikey.to_string(),
                self.job_poster.clone(),
                tx.clone(),
            ),
            tx,
        ));

        Ok(Response::new(rx))
    }
}
