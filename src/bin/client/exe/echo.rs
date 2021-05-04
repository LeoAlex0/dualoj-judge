use std::io::BufRead;

use dualoj_judge::proto::EchoMsg;

use super::Client;

impl Client {
    pub(crate) async fn echo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        std::io::stdin()
            .lock()
            .lines()
            .filter_map(|s| s.ok())
            .map(|s| {
                futures::executor::block_on(self.raw.echo(EchoMsg { message: s })).map_or_else(
                    |e| format!("<Error status: {}>", e),
                    |r| r.into_inner().message,
                )
            })
            .for_each(|line| println!("{}", line));

        Ok(())
    }
}
