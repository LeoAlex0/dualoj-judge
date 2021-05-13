use dualoj_judge::proto::{self, JudgeLimit, JudgeRequest};
use proto::judge_event::Event;

use crate::cli::commands::JudgeParam;

use super::Client;

impl Client {
    pub async fn judge(
        &mut self,
        JudgeParam {
            judger,
            judged,
            cpu_limit,
            mem_limit,
            time_limit,
        }: JudgeParam,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut resp = self
            .raw
            .judge(JudgeRequest {
                judged: proto::Uuid {
                    data: judged.as_bytes().to_vec(),
                },
                judger: proto::Uuid {
                    data: judger.as_bytes().to_vec(),
                },
                limit: JudgeLimit {
                    cpu: cpu_limit,
                    memory: mem_limit,
                    time: time_limit,
                },
            })
            .await?
            .into_inner();

        while let Some(msg) = resp.message().await? {
            if let Some(event) = msg.event {
                match event {
                    Event::Created(msg) => {
                        println!("Created: {}", uuid::Uuid::from_slice(&msg.job_uid.data)?)
                    }
                    Event::Exit(msg) => println!("Exited: {:?}", msg),
                    Event::Error(msg) => println!("Error: {:?}", msg),
                }
            }
        }
        Ok(())
    }
}
