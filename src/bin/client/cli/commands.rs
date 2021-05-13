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

    /// New a temporary Job to test the builded image
    NewJob(NewJobParam),

    /// New a Judge job to test if solver is right.
    Judge(JudgeParam),
}

#[derive(StructOpt)]
pub(crate) struct UploadParam {
    pub path: PathBuf,

    /// Exclude file/dir (support glob pattern)
    #[structopt(long)]
    pub exclude: Vec<String>,

    /// Only output uuid returned(or nothing when failed).
    #[structopt(long, short)]
    pub brief: bool,
}

#[derive(StructOpt)]
pub(crate) struct BuildParam {
    /// UUID of uploaded directory
    pub uuid: uuid::Uuid,
}

#[derive(StructOpt)]
pub(crate) struct NewJobParam {
    /// UUID of uploaded directory
    pub uuid: uuid::Uuid,
}

#[derive(StructOpt)]
pub(crate) struct JudgeParam {
    /// UUID of judger
    pub judger: uuid::Uuid,

    /// UUID of judged
    pub judged: uuid::Uuid,

    /// CPU Limit (in mili-cpu)
    #[structopt(long, default_value = "2000")]
    pub cpu_limit: u32,

    /// Memory Limit (in MiB)
    #[structopt(long, default_value = "64")]
    pub mem_limit: u32,

    /// Time Limit (in Second)
    #[structopt(long, default_value = "5")]
    pub time_limit: u32,
}
