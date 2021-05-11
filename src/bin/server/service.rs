mod build;
mod kube;
mod new_job;
mod upload;

use futures::channel::mpsc;
use log::info;

use dualoj_judge::proto::{
    controller_server::Controller, BuildMsg, Chunk, EchoMsg, UploadStatus, Uuid,
};

use tonic::{Request, Response, Status};

use crate::cli;

pub(crate) struct FileService {
    pub archive_size_limit: usize,
    pub registry: cli::registry::Param,
    pub buildkit: cli::buildkit::Param,
    pub pod_env: cli::pod_env::Param,
}

#[tonic::async_trait]
impl Controller for FileService {
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
}
