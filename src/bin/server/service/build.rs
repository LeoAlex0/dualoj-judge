use std::{env::temp_dir, process::Stdio};

use futures::{channel::mpsc, io::BufReader, AsyncBufReadExt, SinkExt, StreamExt, TryStreamExt};
use tokio::process::Command;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tonic::{Code, Request, Response, Status};

use crate::service::FileService;
use dualoj_judge::proto::{build_msg::MsgOrReturn, BuildMsg, Uuid};

type BuildStream = futures::channel::mpsc::Receiver<Result<BuildMsg, Status>>;

impl FileService {
    pub(crate) async fn build(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<BuildStream>, Status> {
        // Get UUID from request.
        let uuid = uuid::Uuid::from_slice(&request.into_inner().data)
            .map_err(|e| Status::new(Code::Unavailable, format!("UUID is unavaliable: {}", e)))?;

        // Construct context-dir
        let mut context_dir = temp_dir();
        context_dir.push(uuid.to_string());

        // Execute buildctl to build
        let mut child = Command::new("buildctl")
            .args(&[
                format!("--addr=tcp://{}", self.buildkitd_url).as_str(),
                "--tlsdir=/certs",
                "build",
                "--frontend=dockerfile.v0",
                format!("--local=context={}", context_dir.display()).as_str(),
                format!("--local=dockerfile={}", context_dir.display()).as_str(),
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
        let (mut tx, rx) = mpsc::channel(5);
        let tx1 = tx.clone();
        let tx2 = tx.clone();

        tokio::spawn(async move {
            stdout_lines
                .filter_map(|it| async { it.ok() })
                .map(|it| Ok(it))
                .forward(tx1)
                .await
        });

        tokio::spawn(async move {
            stderr_lines
                .filter_map(|it| async { it.ok() })
                .map(|it| Ok(it))
                .forward(tx2)
                .await
        });

        tokio::spawn(async move {
            if let Ok(code) = child.wait().await {
                if let Some(code) = code.code() {
                    tx.send(Ok(BuildMsg {
                        msg_or_return: Some(MsgOrReturn::Code(code)),
                    }))
                    .await
                    .unwrap_or_default();
                }
            }
        });

        Ok(Response::new(rx))
    }
}
