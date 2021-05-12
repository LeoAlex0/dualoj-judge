use std::{convert::TryFrom, net::SocketAddr};

use log::info;
use tokio::{
    select,
    signal::{ctrl_c, unix::SignalKind},
    spawn,
};
use tonic::transport::{
    server::{Router, Unimplemented},
    Server,
};

use crate::{cli::CLI, service::ControlService};
use dualoj_judge::proto::controller_server::ControllerServer;

pub struct Executor {
    router: Router<ControllerServer<ControlService>, Unimplemented>,
    addr: SocketAddr,
}

impl Executor {
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Server listen on {}", self.addr);

        let rpc_thread = spawn(async move { self.router.serve(self.addr).await });
        let mut term = tokio::signal::unix::signal(SignalKind::terminate())?;

        select! {
            _ = ctrl_c() => {}
            _ = term.recv() => {}
        }

        rpc_thread.abort();
        Ok(())
    }
}

impl TryFrom<CLI> for Executor {
    type Error = tonic::transport::Error;

    fn try_from(value: CLI) -> Result<Self, Self::Error> {
        let server = ControllerServer::new(ControlService {
            archive_size_limit: value.archive_size_limit,
            buildkit: value.buildkit,
            registry: value.registry,
            pod_env: value.pod_env,
        });

        Ok(Executor {
            router: Server::builder().add_service(server),
            addr: value.addr,
        })
    }
}
