use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct CLI {
    /// Size-limit of uploaded archives
    #[structopt(long, default_value = "10000000")]
    pub archive_size_limit: usize,

    /// CA Certificate content
    #[structopt(long, env = "ca.pem")]
    pub ca_cert: Option<String>,

    /// Key content
    #[structopt(long, env = "key.pem")]
    pub key: Option<String>,

    /// Certificate content
    #[structopt(long, env = "cert.pem")]
    pub cert: Option<String>,

    /// Address to listen
    #[structopt(long, env = "ADDR", default_value = "0.0.0.0:50051")]
    pub addr: SocketAddr,
}
