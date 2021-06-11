pub mod commands;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Console {
    /// Address of server to connect to.
    #[structopt(long = "addr", default_value = "grpcs://localhost:443")]
    pub addr: String,

    /// command to execute
    #[structopt(subcommand)]
    pub command: commands::SubCommand,

    /// when connect, trust custom tls ca certificate.
    #[structopt(long = "tls-ca-cert")]
    pub ca_cert_path: Option<PathBuf>,
}
