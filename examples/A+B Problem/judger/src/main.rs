use std::{env::var, process::exit, sync::Arc, thread::spawn};

use a_plus_b_judger::pb::{self, judger_client::JudgerClient, JudgerRequest, TestCode, TestResult};
use rand::random;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ResultPoster::default().await?;
    let mut buf_stdin = BufReader::new(tokio::io::stdin());

    let mut input_data = Vec::new();
    for _ in 0..5000 {
        input_data.push((random::<u32>() / 2, random::<u32>() / 2))
    }
    let data_out = Arc::new(input_data);
    let data_in = data_out.clone();

    spawn(move || {
        data_out.iter().for_each(|(a, b)| println!("{} {}", a, b));
    });

    for (a, b) in data_in.iter() {
        let mut line = String::new();
        while line.trim() == "" {
            buf_stdin.read_line(&mut line).await?;
        }

        match line.trim().parse::<u32>() {
            Ok(ans) => {
                if ans != a + b {
                    client
                        .post(
                            TestCode::WrongAnswer,
                            Some(format!("{} + {} = {}, but output {}.", a, b, a + b, ans)),
                        )
                        .await;
                    eprintln!("Wrong answer");
                    exit(1);
                }
            }
            Err(e) => {
                client
                    .post(
                        TestCode::Other,
                        Some(format!("Line \"{}\"Presentation error: {}", line, e)),
                    )
                    .await;
                eprintln!("Cannot scan input \"{}\": {}", line, e);
                exit(1);
            }
        }
    }

    client.post(TestCode::Accepted, None).await;
    Ok(())
}

struct ResultPoster {
    client: JudgerClient<tonic::transport::channel::Channel>,
    token: String,
}

impl ResultPoster {
    pub async fn default() -> Result<Self, String> {
        if let (Ok(connect_addr), Ok(token)) = (var("JUDGER_ADDR"), var("TOKEN")) {
            if let Some(client) =
                pb::judger_client::JudgerClient::connect(format!("grpc://{}", connect_addr))
                    .await
                    .ok()
            {
                Ok(ResultPoster { client, token })
            } else {
                let err = format!("Connect error to {}", connect_addr);
                eprintln!("{}", &err);
                Err(err)
            }
        } else {
            let err = "No ENV_VAR, may running in bare environment";
            eprintln!("{}", err);
            Err(err.to_string())
        }
    }

    pub async fn post(mut self, code: TestCode, other_msg: Option<String>) {
        let _ = self
            .client
            .post_test_result(JudgerRequest {
                token: self.token,
                result: TestResult {
                    other_msg,
                    code: code.into(),
                },
            })
            .await;
    }
}
