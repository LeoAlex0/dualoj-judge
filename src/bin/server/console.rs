use structopt::StructOpt;
pub mod buildkit;
pub mod pod_env;
pub mod registry;

#[derive(StructOpt)]
pub(crate) struct Console {
    /// Size-limit of uploaded archives
    #[structopt(long, default_value = "10000000", env = "SIZE_LIMIT")]
    pub archive_size_limit: usize,

    /// Buildkitd Client CA Certificate content
    #[structopt(long, env = "ca.pem")]
    pub ca_cert: Option<String>,

    /// Port for judger controller to listen
    #[structopt(long, env = "CONTROLLER_PORT", default_value = "50051")]
    pub controller_port: u16,

    /// Port for judger server to listen
    #[structopt(long, env = "JUDGER_PORT", default_value = "80")]
    pub judger_port: u16,

    #[structopt(flatten)]
    pub buildkit: buildkit::Param,

    #[structopt(flatten)]
    pub registry: registry::Param,

    #[structopt(flatten)]
    pub pod_env: pod_env::Param,
}
