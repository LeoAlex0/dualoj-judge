use structopt::StructOpt;
#[derive(StructOpt)]
pub(crate) struct Param {
    #[structopt(env = "POD_NAMESPACE", default_value = "dualoj")]
    pub namespace: String,
    #[structopt(env = "POD_UID", default_value = "")]
    pub uid: String,
    #[structopt(env = "POD_NAME", default_value = "")]
    pub name: String,
}
