use std::{convert::TryFrom, net::SocketAddr};

use tonic::transport::server::Unimplemented;

use crate::service::BuilderServer;
use crate::{cli::CLI, service::FileService};
pub struct Executor {
    router: tonic::transport::server::Router<BuilderServer<FileService>, Unimplemented>,
    addr: SocketAddr,
}

impl Executor {
    pub async fn serve(self) -> Result<(), tonic::transport::Error> {
        println!("Server listen on {}", self.addr);
        self.router.serve(self.addr).await
    }
}

impl TryFrom<CLI> for Executor {
    type Error = tonic::transport::Error;

    fn try_from(value: CLI) -> Result<Self, Self::Error> {
        let server = FileService::new_default();

        Ok(Executor {
            router: tonic::transport::Server::builder().add_service(server),
            addr: value.addr,
        })
    }
}
