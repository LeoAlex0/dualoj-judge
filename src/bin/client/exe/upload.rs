use std::path::PathBuf;

use futures::{stream::iter, StreamExt};

use super::exe::Chunk;
use super::exe::Client;

const CHUNK_LEN: usize = 1 << 20; // 1 MiB

impl Client {
    pub(crate) async fn upload(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("Compressing");

        let data = {
            let mut tar = tar::Builder::new(Vec::new());
            tar.append_dir_all(".", path)?;
            tar.finish()?;

            tar.into_inner()?
        };

        let len = data.len();
        println!("Tar length:{}, Sending", len);

        let response = self
            .raw
            .upload_archive(
                iter(data)
                    .chunks(CHUNK_LEN)
                    .zip(iter(0..))
                    .map(move |(x, id)| {
                        println!("Sending: {}/{}", id * CHUNK_LEN + x.len(), &len);
                        Chunk { content: x }
                    }),
            )
            .await;

        match response {
            Ok(response) => println!(
                "Response {}: {}",
                response.get_ref().code,
                response.get_ref().message
            ),
            Err(e) => println!("Server-side error received: {}", e),
        }

        Ok(())
    }
}
