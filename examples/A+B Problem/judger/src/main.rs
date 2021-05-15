use std::{env::var, process::exit};

use a_plus_b_judger::pb::{self, judger_client::JudgerClient, JudgerRequest, TestCode, TestResult};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ResultPoster::default().await;
    let mut buf_stdin = BufReader::new(tokio::io::stdin());

    for _ in 0..5000 {
        let a = rand::random::<u32>() >> 1;
        let b = rand::random::<u32>() >> 1;
        println!("{} {}", a, b);

        let mut line = String::new();
        while line.trim() == "" {
            buf_stdin.read_line(&mut line).await?;
        }

        match line.trim().parse::<u32>() {
            Ok(ans) => {
                if ans != a + b {
                    if let Some(client) = &mut client {
                        client
                            .post(
                                TestCode::WrongAnswer,
                                Some(format!("{} + {} = {}, but output {}.", a, b, a + b, ans)),
                            )
                            .await;
                    } else {
                        eprintln!("Wrong answer");
                    }
                    exit(1);
                }
            }
            Err(e) => {
                if let Some(client) = &mut client {
                    client
                        .post(
                            TestCode::Other,
                            Some(format!("Line \"{}\"Presentation error: {}", line, e)),
                        )
                        .await;
                } else {
                    eprintln!("Cannot scan input \"{}\": {}", line, e);
                    exit(1);
                }
            }
        }
    }

    if let Some(mut client) = client {
        client.post(TestCode::Accepted, None).await;
    }
    Ok(())
}

struct ResultPoster {
    client: JudgerClient<tonic::transport::channel::Channel>,
    id: String,
    apikey: String,
}

impl ResultPoster {
    pub async fn default() -> Option<Self> {
        if let (Ok(connect_addr), Ok(id), Ok(apikey)) =
            (var("JUDGER_ADDR"), var("JUDGE_ID"), var("APIKEY"))
        {
            if let Some(client) =
                pb::judger_client::JudgerClient::connect(format!("grpc://{}", connect_addr))
                    .await
                    .ok()
            {
                Some(ResultPoster {
                    client,
                    id,
                    apikey,
                })
            } else {
                eprintln!("Connect error to {}", connect_addr);
                None
            }
        } else {
            eprintln!("No ENV_VAR, may running in bare environment");
            None
        }
    }

    pub async fn post(&mut self, code: TestCode, other_msg: Option<String>) {
        let _ = self
            .client
            .post_test_result(JudgerRequest {
                judge_id: self.id.clone(),
                api_key: self.apikey.clone(),
                result: TestResult {
                    other_msg,
                    code: code.into(),
                },
            })
            .await;
    }
}
