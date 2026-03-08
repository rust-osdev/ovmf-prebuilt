use anyhow::{bail, Context, Result};
use serde::Deserialize;
use ureq::Agent;

/// Name of the github organization for this repo.
const ORG: &str = "rust-osdev";

/// Name of this repo on github.
const REPO: &str = "ovmf-prebuilt";

/// User-Agent header to send with requests.
const USER_AGENT: &str = "https://github.com/rust-osdev/ovmf-prebuilt";

#[derive(Debug, Deserialize)]
struct Asset {
    digest: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    name: Option<String>,
    assets: Vec<Asset>,
}

impl Release {
    pub fn sha256(&self) -> Result<&str> {
        if self.assets.len() != 1 {
            bail!("expected exactly one asset, got {}", self.assets.len());
        }
        let asset = &self.assets[0];
        let digest = asset.digest.as_ref().context("asset is missing digest")?;
        digest
            .strip_prefix("sha256:")
            .context("digest format is not sha256")
    }
}

pub struct Github {
    agent: Agent,
}

impl Github {
    pub fn new() -> Self {
        let config = Agent::config_builder().user_agent(USER_AGENT).build();
        let agent = Agent::new_with_config(config);
        Github { agent }
    }

    pub fn get_releases(&self) -> Result<Vec<Release>> {
        let url = format!("https://api.github.com/repos/{ORG}/{REPO}/releases");
        println!("downloading release list");
        let mut releases: Vec<Release> = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .call()?
            .body_mut()
            .read_json()?;
        // Remove releases with a null `name`. This serves to exclude older
        // releases (with tags like "v0.20211216.194+gcc2db6ebfb"), from
        // before the repo was reworked to its current form.
        releases.retain(|release| release.name.is_some());
        Ok(releases)
    }
}
