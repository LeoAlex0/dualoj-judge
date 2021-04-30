use std::convert::TryFrom;

use futures::executor::block_on;
use tonic::transport::{Certificate, Channel};

use crate::cli::{commands::SubCommand, CLI};

#[path = "../../../proto.rs"]
mod proto;
pub use proto::*;

use proto::builder_client::BuilderClient;

pub(crate) struct Client {
    pub(crate) raw: BuilderClient<Channel>,
}

pub struct Executor {
    pub(crate) command: SubCommand,
    pub(crate) client: Client,
}

impl TryFrom<CLI> for Executor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(cli: CLI) -> Result<Self, Self::Error> {
        let mut endpoint = tonic::transport::channel::Channel::from_shared(cli.addr)?;

        if let Some(path) = &cli.ca_cert_path {
            endpoint = endpoint.tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .ca_certificate(Certificate::from_pem(std::fs::read(path)?)),
            )?
        }

        let channel = block_on(endpoint.connect())?;

        let client = BuilderClient::new(channel);

        Ok(Executor {
            client: Client { raw: client },
            command: cli.command,
        })
    }
}
