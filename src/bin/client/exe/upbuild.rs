use std::{path::PathBuf, process::exit};

use futures::{stream::iter, StreamExt};
use glob::Pattern;
use log::{debug, info};

use dualoj_judge::proto::{upbuild_msg::MsgOrReturn, Chunk, UpbuildMsg};

use super::Client;
use crate::console::commands::UploadParam;

const CHUNK_LEN: usize = 1 << 20; // 1 MiB

impl Client {
    pub(crate) async fn upbuild(
        &mut self,
        UploadParam {
            path,
            exclude,
            brief,
        }: UploadParam,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let blacklist: Vec<_> = exclude
            .iter()
            .filter_map(|x| Pattern::new(x).ok())
            .collect();

        info!("Compressing");

        let data = {
            let mut tar = tar::Builder::new(Vec::new());

            let mut inner_root = PathBuf::new();
            inner_root.push(".");
            let mut stack = vec![(path, inner_root)];
            while let Some((ext, inner)) = stack.pop() {
                if !blacklist.iter().any(|x| x.matches_path(&ext)) {
                    debug!("add {} into {}", ext.display(), inner.display());
                    tar.append_path_with_name(&ext, &inner)?;

                    if ext.is_dir() {
                        for entry in std::fs::read_dir(ext)? {
                            let entry = entry?;
                            let mut new_inner = inner.clone();

                            new_inner.push(entry.file_name());
                            stack.push((entry.path(), new_inner));
                        }
                    }
                } else {
                    debug!("file/directory {} is excluded", ext.display());
                }
            }
            tar.finish()?;
            tar.into_inner()?
        };

        let len = data.len();
        info!("Tar length:{}, Sending", len);

        let response = self
            .raw
            .upbuild(
                iter(data)
                    .chunks(CHUNK_LEN)
                    .zip(iter(0..))
                    .map(move |(x, id)| {
                        info!("Sending: {}/{}", id * CHUNK_LEN + x.len(), &len);
                        Chunk { content: x }
                    }),
            )
            .await;

        match response {
            Ok(response) => {
                let mut response = response.into_inner();
                while let Some(UpbuildMsg { msg_or_return }) = response.message().await? {
                    match msg_or_return {
                        None => println!("None MSG"),
                        Some(MsgOrReturn::Code(code)) => {
                            if !brief {
                                println!("`buildctl` exited, code: {}", code);
                            }
                            exit(code);
                        }
                        Some(MsgOrReturn::Stdout(line)) => {
                            if !brief {
                                println!("{}", line)
                            }
                        }
                        Some(MsgOrReturn::Stderr(line)) => eprintln!("{}", line),
                        Some(MsgOrReturn::Complete(id)) => {
                            if brief {
                                println!("{}", id.content)
                            } else {
                                println!("complete: {}", id.content)
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Server-side error received: {}", e),
        }
        Ok(())
    }
}
