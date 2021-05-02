use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct CLI {
    /// Size-limit of uploaded archives
    #[structopt(long, default_value = "10000000", env = "SIZE_LIMIT")]
    pub archive_size_limit: usize,

    /// Buildkitd Client CA Certificate content
    #[structopt(long, env = "ca.pem")]
    pub ca_cert: Option<String>,

    /// Buildkitd Server addr
    #[structopt(long, env = "buildkitd-url", default_value = "localhost:1234")]
    pub buildkit_url: String,

    /// Buildkitd Client Key content
    #[structopt(long, env = "key.pem")]
    pub key: Option<String>,

    /// Buildkitd Client Certificate content
    #[structopt(long, env = "cert.pem")]
    pub cert: Option<String>,

    /// Network address for judger to listen
    #[structopt(long, env = "ADDR", default_value = "0.0.0.0:50051")]
    pub addr: SocketAddr,
}
