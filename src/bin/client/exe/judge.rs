use dualoj_judge::{
    id::gen::ID,
    proto::{self, JudgeLimit, JudgeRequest},
};
use proto::judge_event::Event;

use crate::console::commands::JudgeParam;

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
                judged: ID::from(judged),
                judger: ID::from(judger),
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
                    Event::Created(msg) => println!("Created: {}", msg.job_uid),
                    Event::Exit(msg) => println!("Exited: {:?}", msg),
                    Event::Error(msg) => println!("Error: {:?}", msg),
                }
            }
        }
        Ok(())
    }
}
