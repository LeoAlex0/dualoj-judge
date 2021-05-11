use std::{convert::TryFrom, net::SocketAddr};

use log::info;
use tokio::{signal::ctrl_c, spawn};
use tonic::transport::{
    server::{Router, Unimplemented},
    Server,
};

use crate::{cli::CLI, service::FileService};
use dualoj_judge::proto::builder_server::BuilderServer;

pub struct Executor {
    router: Router<BuilderServer<FileService>, Unimplemented>,
    addr: SocketAddr,
}

impl Executor {
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Server listen on {}", self.addr);

        let rpc_thread = spawn(async move { self.router.serve(self.addr).await });

        rpc_thread.await??;
        Ok(())
    }
}

impl TryFrom<CLI> for Executor {
    type Error = tonic::transport::Error;

    fn try_from(value: CLI) -> Result<Self, Self::Error> {
        let server = BuilderServer::new(FileService {
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
