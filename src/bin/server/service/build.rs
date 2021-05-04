use std::{env::temp_dir, process::Stdio};

use futures::{channel::mpsc, io::BufReader, AsyncBufReadExt, SinkExt, TryFutureExt, TryStreamExt};
use log::debug;
use tokio::process::Command;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tonic::{Code, Request, Response, Status};

use crate::service::FileService;
use dualoj_judge::proto::{build_msg::MsgOrReturn, BuildMsg, Uuid};
use dualoj_judge::to_internal;

type BuildStream = futures::channel::mpsc::UnboundedReceiver<Result<BuildMsg, Status>>;

impl FileService {
    pub(crate) async fn build(
        &self,
        request: Request<Uuid>,
    ) -> Result<Response<BuildStream>, Status> {
        // Get UUID from request.
        let uuid = uuid::Uuid::from_slice(&request.into_inner().data)
            .map_err(|e| Status::new(Code::Unavailable, format!("UUID is unavaliable: {}", e)))?;
        let context_dir = temp_dir();
        temp_dir().push(uuid.to_string());

        let mut child = Command::new("buildctl")
            .args(&[
                format!("--addr=tcp://{}", self.buildkitd_url).as_str(),
                "--tlsdir=/certs",
                "build",
                "--frontend=dockerfile.v0",
                format!("--local=context=\"{}\"", context_dir.display()).as_str(),
                format!("--local=dockerfile=\"{}\")", context_dir.display()).as_str(),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child
            .stdout
            .take()
            .ok_or(Status::internal("buildkitd stdout cannot piped"))?;

        // ready for return.
        // TODO: use channel instead unbounded;
        let (tx, rx) = mpsc::unbounded();

        BufReader::new(stdout.compat())
            .lines()
            .map_err(to_internal)
            .try_for_each(|line| async {
                let mut tx = tx.clone();
                debug!("buildctl output: {}", line);

                tx.send(Ok(BuildMsg {
                    msg_or_return: Some(MsgOrReturn::Message(line)),
                }))
                .await
                .map_err(to_internal)
            })
            .and_then(|_| async {
                let mut tx = tx.clone();
                let exit_code = child
                    .try_wait()
                    .map_err(to_internal)?
                    .ok_or(Status::internal("no exit code"))?
                    .code()
                    .ok_or(Status::internal("buildctl exit by signal"))?;
                tx.send(Ok(BuildMsg {
                    msg_or_return: Some(MsgOrReturn::Code(exit_code)),
                }))
                .await
                .map_err(to_internal)?;
                Ok::<(), Status>(())
            })
            .await?;

        Ok(Response::new(rx))
    }
}
