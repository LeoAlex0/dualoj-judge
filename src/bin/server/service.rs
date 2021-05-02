mod build;

mod upload;

use log::info;

use dualoj_judge::proto::{
    builder_server::Builder, BuildStatus, Chunk, EchoMsg, UploadStatus, Uuid,
};

use tonic::{Request, Response, Status};

pub(crate) struct FileService {
    pub archive_size_limit: usize,
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

    async fn build(&self, request: Request<Uuid>) -> Result<Response<BuildStatus>, Status> {
        todo!()
    }
}
