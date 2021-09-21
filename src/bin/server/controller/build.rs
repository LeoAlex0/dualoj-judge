use std::{future::ready, process::Stdio};

use futures::{channel::mpsc, io::BufReader, AsyncBufReadExt, SinkExt, StreamExt, TryStreamExt};
use tokio::{process::Command, task};
use tokio_util::compat::TokioAsyncReadCompatExt;
use tonic::{Response, Status};

use crate::controller::ControlService;
use dualoj_judge::proto::{controller_server::Controller, upbuild_msg::MsgOrReturn, UpbuildMsg};

use super::receive::Received;

impl ControlService {
    pub(crate) async fn build(
        &self,
        Received { dir, hashed_id }: Received,
    ) -> Result<Response<<ControlService as Controller>::UpbuildStream>, Status> {
        // Execute buildctl to build
        let mut child = Command::new("buildctl")
            .args(&[
                format!("--addr=tcp://{}", self.buildkit.buildkit_url).as_str(),
                "--tlsdir=/certs",
                "build",
                "--frontend=dockerfile.v0",
                format!("--local=context={}", dir.path().display()).as_str(),
                format!("--local=dockerfile={}", dir.path().display()).as_str(),
                format!(
                    "--output=type=image,name={},registry.insecure=true,push=true",
                    self.registry.push_url(&hashed_id.content)
                )
                .as_str(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().unwrap();

        let stderr = child.stderr.take().unwrap();

        let stdout_lines = BufReader::new(stdout.compat()).lines().map_ok(|line| {
            Ok::<_, Status>(UpbuildMsg {
                msg_or_return: Some(MsgOrReturn::Stdout(line)),
            })
        });
        let stderr_lines = BufReader::new(stderr.compat()).lines().map_ok(|line| {
            Ok::<_, Status>(UpbuildMsg {
                msg_or_return: Some(MsgOrReturn::Stderr(line)),
            })
        });

        // ready for return.
        let (mut tx, rx) = mpsc::unbounded();
        let tx1 = tx.clone();
        let tx2 = tx.clone();

        task::spawn(
            stdout_lines
                .take_while(|it| ready(it.is_ok()))
                .filter_map(|it| ready(it.ok()))
                .map(Ok)
                .forward(tx1),
        );

        task::spawn(
            stderr_lines
                .take_while(|it| ready(it.is_ok()))
                .filter_map(|it| ready(it.ok()))
                .map(Ok)
                .forward(tx2),
        );

        task::spawn(async move {
            let _dir = dir; // Remove when invoke complete
            if let Ok(code) = child.wait().await {
                if let Some(code) = code.code() {
                    if code == 0 {
                        tx.send(Ok(UpbuildMsg {
                            msg_or_return: Some(MsgOrReturn::Complete(hashed_id)),
                        }))
                        .await
                        .unwrap();
                    }
                    tx.send(Ok(UpbuildMsg {
                        msg_or_return: Some(MsgOrReturn::Code(code)),
                    }))
                    .await
                    .unwrap();
                }
            }
        });

        Ok::<_, Status>(Response::new(rx))
    }
}
