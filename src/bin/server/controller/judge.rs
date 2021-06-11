mod bind;
mod error;
mod core;
mod judger;
mod manifest;
mod pod_listener;

use self::core::JudgeEnv;
use super::ControlService;
use dualoj_judge::proto::{controller_server::Controller, JudgeEvent, JudgeLimit};

use futures::{channel::mpsc, StreamExt};

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
        let (tx1, rx1) = mpsc::channel(20);
        let (tx2, rx2) = mpsc::channel(20);

        let judge = core::Judge::new(
            self.pod_api.clone(),
            JudgeEnv {
                pod_env: self.pod_env.clone(),
                server_addr: self.judger_addr,
            },
            self.job_poster.clone(),
            tx1,
            self.registry.get_image_url(&judged.to_string()),
            self.registry.get_image_url(&judger.to_string()),
            limit,
        );

        task::spawn(judge.invoke());

        task::spawn(
            rx1.map(|e| JudgeEvent { event: Some(e) })
                .map(Ok)
                .map(Ok)
                .forward(tx2),
        );

        Ok(Response::new(rx2))
    }
}
