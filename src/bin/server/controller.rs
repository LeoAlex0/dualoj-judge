mod build;
mod judge;
mod new_job;
mod upload;

use futures::channel::mpsc;
use log::info;

use dualoj_judge::{
    proto::{
        controller_server::Controller, BuildMsg, Chunk, EchoMsg, JudgeEvent, UploadStatus, Uuid,
    },
    to_internal,
};

use tonic::{Request, Response, Status};

use crate::{cli, judge_server::JudgeMsg};

pub(crate) struct ControlService {
    pub archive_size_limit: usize,
    pub registry: cli::registry::Param,
    pub buildkit: cli::buildkit::Param,
    pub pod_env: cli::pod_env::Param,
    pub job_poster: mpsc::Sender<JudgeMsg>,
    pub k8s_client: kube::Client,
}

#[tonic::async_trait]
impl Controller for ControlService {
    async fn echo(&self, request: Request<EchoMsg>) -> Result<Response<EchoMsg>, Status> {
        info!("Request in: {}", request.get_ref().message);
        Ok(Response::new(request.into_inner()))
    }

    async fn upload_archive(
        &self,
        request: Request<tonic::Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        self.upload_archive(request).await
    }

    type BuildStream = mpsc::UnboundedReceiver<Result<BuildMsg, Status>>;

    async fn build(&self, request: Request<Uuid>) -> Result<Response<Self::BuildStream>, Status> {
        self.build(request).await
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
