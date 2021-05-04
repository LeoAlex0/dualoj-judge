use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) enum SubCommand {
    /// Echo rpc to check whether server is healthy.
    Echo,

    /// Upload to Build Server
    Upload(UploadParam),

    /// Build an Docker project through uuid
    Build(BuildParam),
}

#[derive(StructOpt)]
pub(crate) struct UploadParam {
    pub path: PathBuf,

    /// Exclude file/dir (support glob pattern)
    #[structopt(long)]
    pub exclude: Vec<String>,
}

#[derive(StructOpt)]
pub(crate) struct BuildParam {
    /// UUID of uploaded directory
    pub uuid: uuid::Uuid,
}
