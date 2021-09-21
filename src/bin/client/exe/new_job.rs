use dualoj_judge::{id::gen::ID, proto::NewJobResponse};

use crate::console::commands::NewJobParam;

use super::Client;

impl Client {
    pub async fn new_job(&mut self, param: NewJobParam) -> Result<(), Box<dyn std::error::Error>> {
        let NewJobResponse { result, code } =
            self.raw.new_job(ID::from(param.dir)).await?.into_inner();

        if let Some(res) = result {
            match res {
                dualoj_judge::proto::new_job_response::Result::ErrorMsg(msg) => {
                    eprintln!("code: {}, error_msg: {}", code, msg)
                }
                dualoj_judge::proto::new_job_response::Result::JobUid(uid) => {
                    println!("{}", uid.content)
                }
            }
        } else {
            eprintln!("code: {}, no code or uid response", code)
        }
        Ok(())
    }
}
