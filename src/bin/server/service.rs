#[path = "../../proto.rs"]
mod proto;
use futures::StreamExt;
use proto::{builder_server::Builder, builder_server::BuilderServer, Chunk, EchoMsg, UploadStatus};
use tonic::{Request, Response, Status, Streaming};

const ARCHIVE_SIZE_LIMIT_BYTES: usize = 10 << 20; // 10 MiB

#[derive(Default)]
pub(crate) struct FileService {}

#[tonic::async_trait]
impl Builder for FileService {
    async fn upload_archive(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        let _input = request
            .into_inner()
            .filter_map(|it| async { it.ok() })
            .flat_map(|x| futures::stream::iter(x.content))
            .take(ARCHIVE_SIZE_LIMIT_BYTES);
        todo!()
    }

    async fn echo(&self, request: Request<EchoMsg>) -> Result<Response<EchoMsg>, Status> {
        println!("Request in: {}", request.get_ref().message);
        Ok(Response::new(request.into_inner()))
    }
}

impl FileService {
    pub fn new_default() -> BuilderServer<FileService> {
        BuilderServer::new(Self::default())
    }
}
