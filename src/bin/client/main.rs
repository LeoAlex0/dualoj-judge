mod cli;
mod exe;
mod k8s_demo;

use std::convert::TryFrom;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    exe::Executor::try_from(cli::Cli::from_args())?
        .invoke()
        .await
}
