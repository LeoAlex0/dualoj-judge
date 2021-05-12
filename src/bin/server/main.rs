mod cli;
mod controller;
mod exe;
mod judge_server;

use std::convert::TryFrom;

use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    exe::Executor::try_from(cli::CLI::from_args())?
        .serve()
        .await
        .map_err(|e| e.into())
}
