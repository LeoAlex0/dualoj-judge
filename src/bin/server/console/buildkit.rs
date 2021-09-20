use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Param {
    /// Buildkitd Server addr
    #[structopt(env = "buildkitd-url", default_value = "localhost:1234")]
    pub buildkit_url: String,
}
