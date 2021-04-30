mod echo;
mod exe;
mod upload;

pub use exe::Executor;

use crate::cli::commands::SubCommand::*;

impl Executor {
    pub async fn invoke(mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Echo => self.client.echo().await,
            Upload { path } => self.client.upload(path).await,
        }
    }
}
