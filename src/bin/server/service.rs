#[path = "../../proto.rs"]
mod proto;

use std::env::temp_dir;

use log::{debug, info};

use prost::bytes::Buf;
pub use proto::{
    builder_server::Builder, builder_server::BuilderServer, Chunk, EchoMsg, UploadStatus,
};
use tonic::{Request, Response, Status, Streaming};

pub(crate) struct FileService {
    pub archive_size_limit: usize,
}

#[tonic::async_trait]
impl Builder for FileService {
    async fn upload_archive(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        let mut stream = request.into_inner();
        let mut data = Vec::new();
        let mut received_size = 0usize;

        while let Some(chunk) = stream.message().await? {
            received_size += chunk.content.len();
            debug!("received: {} Byte", received_size);

            if received_size > self.archive_size_limit {
                return Err(Status::new(
                    tonic::Code::Aborted,
                    format!(
                        "too large archive, max size is {} Byte",
                        self.archive_size_limit
                    ),
                ));
            }

            data.push(chunk.content);
        }
        let raw = data.concat();

        debug!("received complete, unpacking");

        let mut tar = tar::Archive::new(raw.reader());
        tar.unpack(temp_dir())?;

        debug!("unpacking complete");

        Ok(Response::new(UploadStatus {
            code: 0,
            message: format!("upload OK, upload to {}", temp_dir().display()),
        }))
    }

    async fn echo(&self, request: Request<EchoMsg>) -> Result<Response<EchoMsg>, Status> {
        info!("Request in: {}", request.get_ref().message);
        Ok(Response::new(request.into_inner()))
    }
}
