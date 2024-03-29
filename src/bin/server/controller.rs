mod build;
mod judge;
mod new_job;
mod receive;

use std::net::SocketAddr;

use futures::channel::mpsc;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use log::info;

use dualoj_judge::proto::{controller_server::Controller, Chunk, EchoMsg, JudgeEvent, UpbuildMsg};

use tonic::{Request, Response, Status};

use crate::{console, judge_server::JudgeMsg};

pub(crate) struct ControlService {
    pub archive_size_limit: usize,
    pub registry: console::registry::Param,
    pub buildkit: console::buildkit::Param,
    pub pod_env: console::pod_env::Param,
    pub judger_addr: SocketAddr,

    // TODO: use redis/cache to specify listening status.
    pub job_poster: mpsc::Sender<JudgeMsg>,

    pub pod_api: Api<Pod>,
}

#[mockall::automock]
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
        request: tonic::Request<dualoj_judge::proto::Id>,
    ) -> Result<tonic::Response<dualoj_judge::proto::NewJobResponse>, tonic::Status> {
        self.new_job(request).await
    }

    type JudgeStream = mpsc::Receiver<Result<JudgeEvent, tonic::Status>>;

    async fn judge(
        &self,
        request: Request<dualoj_judge::proto::JudgeRequest>,
    ) -> Result<Response<Self::JudgeStream>, Status> {
        let req = request.into_inner();
        self.new_judge_job(req.limit, &req.judged.content, &req.judger.content)
            .await
    }
}
