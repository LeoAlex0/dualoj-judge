use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Param {
    /// Buildkitd Server addr
    #[structopt(long, env = "buildkitd-url", default_value = "localhost:1234")]
    pub url: String,

    /// Buildkitd Client Key content
    #[structopt(long, env = "key.pem")]
    pub key: Option<String>,

    /// Buildkitd Client Certificate content
    #[structopt(long, env = "cert.pem")]
    pub cert: Option<String>,
}
