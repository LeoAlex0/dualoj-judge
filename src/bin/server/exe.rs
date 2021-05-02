use std::{convert::TryFrom, net::SocketAddr};

use log::info;
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
    pub async fn serve(self) -> Result<(), tonic::transport::Error> {
        info!("Server listen on {}", self.addr);
        self.router.serve(self.addr).await
    }
}

impl TryFrom<CLI> for Executor {
    type Error = tonic::transport::Error;

    fn try_from(value: CLI) -> Result<Self, Self::Error> {
        let server = BuilderServer::new(FileService {
            archive_size_limit: value.archive_size_limit,
        });

        Ok(Executor {
            router: Server::builder().add_service(server),
            addr: value.addr,
        })
    }
}
