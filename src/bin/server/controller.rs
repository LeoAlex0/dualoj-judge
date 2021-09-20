mod build;
mod judge;
mod new_job;
mod receive;

use std::net::SocketAddr;

use futures::channel::mpsc;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use log::info;

use dualoj_judge::{
    proto::{controller_server::Controller, Chunk, EchoMsg, JudgeEvent, UpbuildMsg, Uuid},
    to_internal,
};

use tonic::{Request, Response, Status};

use crate::{console, judge_server::JudgeMsg};

pub(crate) struct ControlService {
    pub archive_size_limit: usize,
    pub registry: console::registry::Param,
    pub buildkit: console::buildkit::Param,
    pub pod_env: console::pod_env::Param,
    pub judger_addr: SocketAddr,
    pub job_poster: mpsc::Sender<JudgeMsg>,

    pub pod_api: Api<Pod>,
}

#[tonic::async_trait]
impl Controller for ControlService {
    async fn echo(&self, request: Request<EchoMsg>) -> Result<Response<EchoMsg>, Status> {
        info!("Request in: {}", request.get_ref().message);
        Ok(Response::new(request.into_inner()))
    }

    type UpbuildStream = mpsc::UnboundedReceiver<Result<UpbuildMsg, Status>>;

    async fn upbuild(
        &self,
        request: Request<tonic::Streaming<Chunk>>,
    ) -> Result<Response<Self::UpbuildStream>, Status> {
        let received = self.receive_archive(request).await?;
        self.build(received).await
    }

    async fn new_job(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<dualoj_judge::proto::NewJobResponse>, Status> {
        self.new_job(request).await
    }

    type JudgeStream = mpsc::Receiver<Result<JudgeEvent, tonic::Status>>;

    async fn judge(
        &self,
        request: Request<dualoj_judge::proto::JudgeRequest>,
    ) -> Result<Response<Self::JudgeStream>, Status> {
        let req = request.into_inner();
        let judged = uuid::Uuid::from_slice(&req.judged.data.to_vec()).map_err(to_internal)?;
        let judger = uuid::Uuid::from_slice(&req.judger.data.to_vec()).map_err(to_internal)?;
        self.new_judge_job(req.limit, judged, judger).await
    }
}
