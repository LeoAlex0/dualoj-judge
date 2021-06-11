mod console;
mod exe;

use std::convert::TryFrom;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    exe::Executor::try_from(console::Console::from_args())?
        .invoke()
        .await
}
