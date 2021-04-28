use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) enum SubCommand {
    /// Echo rpc to check whether server is healthy.
    Echo,
}
