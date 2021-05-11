use dualoj_judge::proto::{NewJobResponse, Uuid};

use crate::cli::commands::NewJobParam;

use super::Client;

impl Client {
    pub async fn new_job(&mut self, param: NewJobParam) -> Result<(), Box<dyn std::error::Error>> {
        let NewJobResponse { result, code } = self
            .raw
            .new_job(Uuid {
                data: param.uuid.as_bytes().to_vec(),
            })
            .await?
            .into_inner();

        if let Some(res) = result {
            match res {
                dualoj_judge::proto::new_job_response::Result::ErrorMsg(msg) => {
                    eprintln!("code: {}, error_msg: {}", code, msg)
                }
                dualoj_judge::proto::new_job_response::Result::JobUid(uid) => {
                    println!("{}", uuid::Uuid::from_slice(&uid.data)?)
                }
            }
        } else {
            eprintln!("code: {}, no code or uid response", code)
        }
        Ok(())
    }
}
