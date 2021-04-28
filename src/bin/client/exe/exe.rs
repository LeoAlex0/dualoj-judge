use std::convert::TryFrom;

use futures::executor::block_on;
use tonic::transport::{Certificate, Channel};

use crate::cli::{commands::SubCommand, Cli};

#[path = "../../../proto.rs"]
mod proto;
pub use proto::*;

use proto::builder_client::BuilderClient;

pub struct Executor {
    pub(crate) command: SubCommand,
    pub(crate) client: BuilderClient<Channel>,
}

impl TryFrom<Cli> for Executor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let mut tls = tonic::transport::ClientTlsConfig::new();

        if let Some(path) = &cli.ca_cert_path {
            tls = tls.ca_certificate(Certificate::from_pem(std::fs::read(path)?));
        }

        let channel = block_on(
            tonic::transport::channel::Channel::from_shared(cli.addr)?
                .tls_config(tls)?
                .connect(),
        )?;

        let client = BuilderClient::new(channel);

        Ok(Executor {
            client,
            command: cli.command,
        })
    }
}
