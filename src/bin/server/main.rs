mod cli;
mod exe;
mod service;

use std::convert::TryFrom;

use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    exe::Executor::try_from(cli::CLI::from_args())?
        .serve()
        .await
        .map_err(|e| e.into())
}
