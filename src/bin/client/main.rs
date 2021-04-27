mod k8s_demo;
#[path = "../../proto.rs"]
mod proto;

use proto::builder_client::*;
use proto::EchoMsg;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tls = tonic::transport::ClientTlsConfig::new()
        .domain_name("localhost")
        .identity(tonic::transport::Identity::from_pem(
            include_str!("../../../.cert/client/cert.pem"),
            include_str!("../../../.cert/client/key.pem"),
        ))
        .ca_certificate(tonic::transport::Certificate::from_pem(include_str!(
            "../../../.cert/client/ca.pem"
        )));

    println!("Load TLS Config Ready");

    let channel = tonic::transport::channel::Channel::from_static("grpcs://localhost:443")
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = BuilderClient::new(channel);

    println!("Connected to server, input message for echoing:");

    io::stdin()
        .lock()
        .lines()
        .filter_map(|s| s.ok())
        .map(|s| {
            futures::executor::block_on(client.echo(EchoMsg { message: s })).map_or_else(
                |e| format!("<Error status: {}>", e),
                |r| r.into_inner().message,
            )
        })
        .for_each(|line| println!("{}", line));

    Ok(())
}
