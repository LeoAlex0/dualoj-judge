mod build;
mod echo;
mod new_job;
mod upload;

use crate::cli::commands::SubCommand::*;
use crate::cli::{commands::SubCommand, CLI};
use dualoj_judge::proto::builder_client::BuilderClient;
use futures::executor::block_on;
use std::convert::TryFrom;
use tonic::transport::{Certificate, Channel};

pub(crate) struct Client {
    pub(crate) raw: BuilderClient<Channel>,
}

pub struct Executor {
    pub(crate) command: SubCommand,
    pub(crate) client: Client,
}

impl Executor {
    pub async fn invoke(mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Echo => self.client.echo().await,
            Upload(param) => self.client.upload(param).await,
            Build(param) => self.client.build(param).await,
            NewJob(param) => self.client.new_job(param).await,
        }
    }
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
