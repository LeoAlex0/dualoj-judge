mod echo;
mod judge;
mod new_job;
mod upbuild;

use crate::console::commands::SubCommand::*;
use crate::console::{commands::SubCommand, Console};
use dualoj_judge::proto::controller_client::ControllerClient;
use futures::executor::block_on;
use std::convert::TryFrom;
use tonic::transport::{Certificate, Channel};

pub(crate) struct Client {
    pub(crate) raw: ControllerClient<Channel>,
}

pub struct Executor {
    pub(crate) command: SubCommand,
    pub(crate) client: Client,
}

impl Executor {
    pub async fn invoke(mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Echo => self.client.echo().await,
            Upbuild(param) => self.client.upbuild(param).await,
            NewJob(param) => self.client.new_job(param).await,
            Judge(param) => self.client.judge(param).await,
        }
    }
}

impl TryFrom<Console> for Executor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(cli: Console) -> Result<Self, Self::Error> {
        let mut endpoint = tonic::transport::channel::Channel::from_shared(cli.addr)?;

        if let Some(path) = &cli.ca_cert_path {
            endpoint = endpoint.tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .ca_certificate(Certificate::from_pem(std::fs::read(path)?)),
            )?
        }

        let channel = block_on(endpoint.connect())?;

        let client = ControllerClient::new(channel);

        Ok(Executor {
            client: Client { raw: client },
            command: cli.command,
        })
    }
}
