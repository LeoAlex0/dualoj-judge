use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Param {
    /// In-cluster registry url (for push)
    #[structopt(env = "registry-push-url", default_value = "localhost")]
    pub push_url: String,

    /// External registry url (for k8s pull)
    #[structopt(env = "registry-pull-url", default_value = "localhost")]
    pub pull_url: String,

    /// The username when upload builded image to internal registry.
    #[structopt(long, default_value = "build")]
    pub username: String,
}

impl Param {
    pub(crate) fn push_url(&self, name: &str) -> String {
        format!("{}/{}/{}", self.push_url, self.username, name)
    }

    pub(crate) fn pull_url(&self, name: &str) -> String {
        format!("{}/{}/{}", self.pull_url, self.username, name)
    }
}
