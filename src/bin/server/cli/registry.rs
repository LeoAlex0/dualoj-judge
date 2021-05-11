use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Param {
    /// In-cluster registry url
    #[structopt(long, env = "registry-url", default_value = "localhost")]
    pub url: String,

    /// The username when upload builded image to internal registry.
    #[structopt(long, default_value = "build")]
    pub username: String,
}
