mod echo;
mod exe;

pub use exe::Executor;

use crate::cli::commands::SubCommand::*;

impl Executor {
    pub async fn invoke(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Echo => self.echo().await,
        }
    }
}
