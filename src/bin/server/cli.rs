use std::net::SocketAddr;

use structopt::StructOpt;
pub mod buildkit;
pub mod pod_env;
pub mod registry;

#[derive(StructOpt)]
pub(crate) struct CLI {
    /// Size-limit of uploaded archives
    #[structopt(long, default_value = "10000000", env = "SIZE_LIMIT")]
    pub archive_size_limit: usize,

    /// Buildkitd Client CA Certificate content
    #[structopt(long, env = "ca.pem")]
    pub ca_cert: Option<String>,

    /// Network address for judger to listen
    #[structopt(long, env = "ADDR", default_value = "0.0.0.0:50051")]
    pub addr: SocketAddr,

    #[structopt(flatten)]
    pub buildkit: buildkit::Param,

    #[structopt(flatten)]
    pub registry: registry::Param,

    #[structopt(flatten)]
    pub pod_env: pod_env::Param,
}
