mod build;

mod upload;

use futures::channel::mpsc::UnboundedReceiver;
use log::info;

use dualoj_judge::proto::{builder_server::Builder, BuildMsg, Chunk, EchoMsg, UploadStatus, Uuid};

use tonic::{Request, Response, Status};

pub(crate) struct FileService {
    pub archive_size_limit: usize,
    pub buildkitd_url: String,
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

    type BuildStream = UnboundedReceiver<Result<BuildMsg, Status>>;

    async fn build(&self, request: Request<Uuid>) -> Result<Response<Self::BuildStream>, Status> {
        self.build(request).await
    }
}
