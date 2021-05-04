use futures::{stream::iter, StreamExt};
use glob::Pattern;
use log::debug;

use dualoj_judge::proto::Chunk;

use super::Client;
use crate::cli::commands::UploadParam;

const CHUNK_LEN: usize = 1 << 20; // 1 MiB

impl Client {
    pub(crate) async fn upload(
        &mut self,
        UploadParam { path, exclude }: UploadParam,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let blacklist: Vec<_> = exclude
            .iter()
            .filter_map(|x| Pattern::new(x).ok())
            .collect();

        println!("Compressing");

        let data = {
            let mut tar = tar::Builder::new(Vec::new());
            // tar.append_dir_all(".", &path)?;

            let mut stack = vec![path];
            while let Some(path) = stack.pop() {
                if !blacklist.iter().any(|x| x.matches_path(&path)) {
                    tar.append_path(&path)?;

                    if path.is_dir() {
                        for entry in std::fs::read_dir(path)? {
                            let entry = entry?;
                            stack.push(entry.path());
                        }
                    }
                } else {
                    debug!("file/directory {} is excluded", path.display());
                }
            }

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
