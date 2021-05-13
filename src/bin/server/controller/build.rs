use std::{env::temp_dir, future::ready, process::Stdio};

use futures::{channel::mpsc, io::BufReader, AsyncBufReadExt, SinkExt, StreamExt, TryStreamExt};
use tokio::process::Command;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tonic::{Code, Request, Response, Status};

use crate::controller::ControlService;
use dualoj_judge::proto::{build_msg::MsgOrReturn, controller_server::Controller, BuildMsg, Uuid};

impl ControlService {
    pub(crate) async fn build(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<<ControlService as Controller>::BuildStream>, Status> {
        // Get UUID from request.
        let uuid = uuid::Uuid::from_slice(&request.into_inner().data)
            .map_err(|e| Status::new(Code::Unavailable, format!("UUID is unavaliable: {}", e)))?;

        // Construct context-dir
        let mut context_dir = temp_dir();
        context_dir.push(uuid.to_string());

        // Execute buildctl to build
        let mut child = Command::new("buildctl")
            .args(&[
                format!("--addr=tcp://{}", self.buildkit.buildkit_url).as_str(),
                "--tlsdir=/certs",
                "build",
                "--frontend=dockerfile.v0",
                format!("--local=context={}", context_dir.display()).as_str(),
                format!("--local=dockerfile={}", context_dir.display()).as_str(),
                format!(
                    "--output=type=image,name={},registry.insecure=true,push=true",
                    self.registry.get_image_url(&uuid.to_string())
                )
                .as_str(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child
            .stdout
            .take()
            .ok_or(Status::internal("buildkitd stdout cannot piped"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or(Status::internal("buildctl stderr cannot piped"))?;

        let stdout_lines = BufReader::new(stdout.compat()).lines().map_ok(|line| {
            Ok::<_, Status>(BuildMsg {
                msg_or_return: Some(MsgOrReturn::Stdout(line)),
            })
        });
        let stderr_lines = BufReader::new(stderr.compat()).lines().map_ok(|line| {
            Ok::<_, Status>(BuildMsg {
                msg_or_return: Some(MsgOrReturn::Stderr(line)),
            })
        });

        // ready for return.
        let (mut tx, rx) = mpsc::unbounded();
        let tx1 = tx.clone();
        let tx2 = tx.clone();

        tokio::spawn(
            stdout_lines
                .take_while(|it| ready(it.is_ok()))
                .filter_map(|it| ready(it.ok()))
                .map(Ok)
                .forward(tx1),
        );

        tokio::spawn(
            stderr_lines
                .take_while(|it| ready(it.is_ok()))
                .filter_map(|it| ready(it.ok()))
                .map(Ok)
                .forward(tx2),
        );

        tokio::spawn(async move {
            if let Ok(code) = child.wait().await {
                if let Some(code) = code.code() {
                    let _ = tx
                        .send(Ok(BuildMsg {
                            msg_or_return: Some(MsgOrReturn::Code(code)),
                        }))
                        .await;
                }
            }
        });

        Ok(Response::new(rx))
    }
}
