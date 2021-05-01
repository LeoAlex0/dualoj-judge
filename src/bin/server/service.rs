#[path = "../../proto.rs"]
pub mod proto;
mod upload;

use log::info;

pub use proto::{
    builder_server::Builder, builder_server::BuilderServer, Chunk, EchoMsg, UploadStatus,
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
}
