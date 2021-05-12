use crate::service::ControlService;

use std::env::temp_dir;

use log::debug;
use prost::bytes::Buf;

use dualoj_judge::proto::{upload_status, Chunk, UploadStatus, Uuid};
use tar::Archive;
use tonic::{Request, Response, Status, Streaming};

impl ControlService {
    pub async fn upload_archive(
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

        let uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, &raw[..]);
        let mut save_dir = temp_dir();
        save_dir.push(uuid.to_string());

        debug!("received complete, unpacking");

        Archive::new(raw.reader()).unpack(&save_dir)?;

        debug!("unpacked to {} complete", save_dir.display());

        Ok(Response::new(UploadStatus {
            code: 0,
            result: Some(upload_status::Result::FolderId(Uuid {
                data: uuid.as_bytes().to_vec(),
            })),
        }))
    }
}
