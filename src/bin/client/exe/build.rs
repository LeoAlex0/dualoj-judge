use dualoj_judge::proto::{build_msg::MsgOrReturn, BuildMsg, Uuid};

use crate::cli::commands::BuildParam;

use super::Client;

impl Client {
    pub async fn build(
        &mut self,
        BuildParam { uuid }: BuildParam,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut resp = self
            .raw
            .build(Uuid {
                data: uuid.as_bytes().to_owned().to_vec(),
            })
            .await?
            .into_inner();

        while let Some(BuildMsg { msg_or_return }) = resp.message().await? {
            match msg_or_return {
                None => println!("None MSG"),
                Some(MsgOrReturn::Code(code)) => println!("`buildctl` exited, code: {}", code),
                Some(MsgOrReturn::Stdout(line)) => println!("{}", line),
                Some(MsgOrReturn::Stderr(line)) => eprintln!("{}", line),
            }
        }
        Ok(())
    }
}
