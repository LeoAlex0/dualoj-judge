#[path = "../../proto.rs"]
mod proto;
use futures::StreamExt;
pub use proto::{builder_server::Builder, builder_server::BuilderServer, Chunk, EchoMsg, UploadStatus};
use tonic::{Request, Response, Status, Streaming};

use structopt::StructOpt;

#[derive(Default, StructOpt)]
pub(crate) struct FileService {
    archive_size_limit: usize,
}

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
            .take(self.archive_size_limit);
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
