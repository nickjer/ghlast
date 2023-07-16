use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use serde::Deserialize;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Output {
    Tag,
    Tarball,
    Assets,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Owner of repo
    owner: String,

    /// Name of repo
    repo: String,

    /// Output value(s)
    #[clap(short, long, value_enum, default_value_t = Output::Tag)]
    output: Output,
}

#[derive(Debug, Deserialize)]
struct Releases(Vec<Release>);

impl Releases {
    pub fn last_release(self) -> Option<Release> {
        self.0.into_iter().next()
    }
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    tarball_url: Option<String>,
    assets: Vec<Asset>,
}

impl Release {
    pub fn tag_name(&self) -> String {
        self.tag_name.clone()
    }

    pub fn tarball_url(&self) -> Option<String> {
        self.tarball_url.clone()
    }

    pub fn download_urls(&self) -> Vec<String> {
        self.assets
            .iter()
            .map(|asset| asset.download_url())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct Asset {
    #[serde(rename = "browser_download_url")]
    download_url: String,
}

impl Asset {
    pub fn download_url(&self) -> String {
        self.download_url.clone()
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let owner = cli.owner.clone();
    let repo = cli.repo.clone();

    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    let response = minreq::get(&url)
        .with_header("Accept", "application/vnd.github.v3+json")
        .with_header("User-Agent", "ghlast")
        .with_param("per_page", "1")
        .send()
        .with_context(|| format!("Failed to get {url}"))?;
    if response.status_code != 200 {
        return Err(anyhow!(
            "Response: {} {}",
            response.status_code,
            response.reason_phrase
        ))
        .with_context(|| format!("Bad response from {url}"));
    }

    let parsed_response = response
        .json::<Releases>()
        .context("Failed to parse response body")?;
    let release = parsed_response
        .last_release()
        .context("Unable to find a release")?;

    match cli.output {
        Output::Tag => println!("{}", release.tag_name()),
        Output::Tarball => {
            let tarball_url = release.tarball_url().unwrap_or_default();
            println!("{tarball_url}")
        }
        Output::Assets => release
            .download_urls()
            .into_iter()
            .for_each(|url| println!("{url}")),
    }

    Ok(())
}
