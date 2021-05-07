mod build;
mod new_job;
mod upload;

use futures::channel::mpsc;
use log::info;

use dualoj_judge::proto::{builder_server::Builder, BuildMsg, Chunk, EchoMsg, UploadStatus, Uuid};

use tonic::{Request, Response, Status};

pub(crate) struct FileService {
    pub archive_size_limit: usize,
    pub buildkitd_url: String,
    pub registry_url: String,
    pub registry_username: String,
}

#[tonic::async_trait]
impl Builder for FileService {
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
