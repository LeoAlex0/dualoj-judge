use dualoj_judge::proto::{NewJobResponse, Uuid};

use crate::cli::commands::NewJobParam;

use super::Client;

impl Client {
    pub async fn new_job(&mut self, param: NewJobParam) -> Result<(), Box<dyn std::error::Error>> {
        let NewJobResponse { error_msg, code } = self
            .raw
            .new_job(Uuid {
                data: param.uuid.as_bytes().to_vec(),
            })
            .await?
            .into_inner();

        if let Some(msg) = error_msg {
            eprintln!("code: {}, error_msg: {}", code, msg)
        }
        Ok(())
    }
}
