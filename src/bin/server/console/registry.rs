use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Param {
    /// In-cluster registry url
    #[structopt(env = "registry-url", default_value = "localhost")]
    pub registry_url: String,

    /// The username when upload builded image to internal registry.
    #[structopt(long, default_value = "build")]
    pub username: String,
}

impl Param {
    pub(crate) fn get_image_url(&self, name: &str) -> String {
        format!("{}/{}/{}", self.registry_url, self.username, name)
    }
}
