use std::{
    env::var,
    io::{stdin, BufRead},
};

use a_plus_b_judger::pb::{self, JudgerRequest, TestCode, TestResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connect_addr = var("JUDGER_ADDR")?;
    let name = var("JOB_ID")?;
    let apikey = var("APIKEY")?;

    let mut client =
        pb::judger_client::JudgerClient::connect(format!("grpc://{}", connect_addr)).await?;

    for _ in 0..5000 {
        let a = rand::random::<u32>() >> 1;
        let b = rand::random::<u32>() >> 1;
        println!("{} {}", a, b);

        let mut line = String::new();
        stdin().lock().read_line(&mut line)?;
        let ans: u32 = line.parse()?;
        if ans != a + b {
            client
                .post_test_result(JudgerRequest {
                    job_id: name.clone(),
                    api_key: apikey.clone(),
                    result: TestResult {
                        other_msg: Some(format!("{} + {} = {}, but output {}.", a, b, a + b, ans)),
                        code: TestCode::WrongAnswer.into(),
                    },
                })
                .await?;
            return Ok(());
        }
    }

    Ok(())
}
