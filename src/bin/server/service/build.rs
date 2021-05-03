use std::env::temp_dir;

use tonic::{Code, Request, Response, Status};

use dualoj_judge::proto;

use crate::service::FileService;

impl FileService {
    async fn build(&self, request: Request<Uuid>) -> Result<Response<BuildStatus>, Status> {
        // Get UUID from request.
        let uuid = uuid::Uuid::from_slice(&request.into_inner().data)
            .map_err(|e| Status::new(Code::Unavailable, format!("UUID is unavaliable: {}", e)))?;
        let context_dir = temp_dir();

        let mut dockerfile_path = temp_dir().clone();
        dockerfile_path.push("Dockerfile");

        todo!()
    }
}
