/*
Copyright 2025 [rhythmcache]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

         http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

/* Cargo toml Configuration 

[package]
name = "ghrls"
version = "0.3.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
bytes = "1.5"
clap = { version = "4.4", features = ["derive"] } 
futures-util = "0.3"
humansize = "2.1"
indicatif = "0.17"
reqwest = { version = "0.11", features = ["json", "stream", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
colored = "2.0"

*/


use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use humansize::{format_size, BINARY};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, RANGE, USER_AGENT};
use serde::Deserialize;
use std::{env, path::PathBuf, sync::Arc};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncSeekExt;
use colored::*;
const MAX_CONCURRENT_DOWNLOADS: usize = 3;
#[derive(Parser)]
#[command(name = "ghrls")]
#[command(about = "ghrls cli tool to view and download release assets", long_about = None)]
struct Cli {
    #[arg(long)]
    pat: Option<String>,

    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    View {
        repo: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(short, long)]
        detailed: bool,
        #[arg(short, long)]
        no_color: bool,
        #[arg(long)]
        latest: Option<Option<usize>>,
    },
    Pull {
        repo: String,
        #[arg(short = 'o')]
        output_dir: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        all: bool,
        file: Option<String>,
    },
}
#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}
#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
    created_at: String,
}
fn parse_github_url(repo_url: &str) -> Option<(String, String)> {
    let repo_url = repo_url.trim();
    if !repo_url.contains("://") && !repo_url.contains('.') {
        let parts: Vec<&str> = repo_url.split('/').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    let url = repo_url.trim_end_matches(".git");
    let without_protocol = url.split("://").last()?;
    let parts: Vec<&str> = without_protocol.split('/').collect();

    if parts.len() >= 3 && parts[0].contains("github.com") {
        return Some((parts[1].to_string(), parts[2].to_string()));
    }
    None
}
#[derive(Clone)]
struct GitHub {
    client: reqwest::Client,
}
impl GitHub {
    fn new(token: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("ghrls"));
        if let Some(token) = token {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("token {}", token))?,
            );
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }
    async fn get_releases(&self, owner: &str, repo: &str) -> Result<Vec<Release>> {
        let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
        let releases = self.client.get(&url).send().await?.json().await?;
        Ok(releases)
    }
    async fn get_release(&self, owner: &str, repo: &str, tag: &str) -> Result<Release> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            owner, repo, tag
        );
        let release = self.client.get(&url).send().await?.json().await?;
        Ok(release)
    }
    async fn download_asset(
        &self,
        url: &str,
        path: &PathBuf,
        pb: Arc<ProgressBar>,
    ) -> Result<()> {
        let mut file = if path.exists() {
            let mut f = File::options().write(true).open(path).await?;
            f.seek(std::io::SeekFrom::End(0)).await?;
            f
        } else {
            File::create(path).await?
        };
        let downloaded = file.metadata().await?.len();
        pb.set_position(downloaded);
        let mut stream = self
            .client
            .get(url)
            .header(RANGE, format!("bytes={}-", downloaded))
            .send()
            .await?
            .bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }
        pb.finish_with_message("✓");
        Ok(())
    }
}
async fn view_command(
    repo_url: &str,
    tag: Option<&str>,
    token: Option<&str>,
    detailed: bool,
    no_color: bool,
    latest: Option<Option<usize>>,
) -> Result<()> {
    let (owner, repo) = parse_github_url(repo_url)
        .ok_or_else(|| anyhow::anyhow!("Invalid GitHub repository URL"))?;
    let github = GitHub::new(token)?;
    match tag {
        Some(tag) => {
            let release = github.get_release(&owner, &repo, tag).await?;
            print_release_tree(&release, detailed, no_color);
        }
        None => {
            let releases = github.get_releases(&owner, &repo).await?;
            let n = match latest {
                Some(Some(n)) => n,
                Some(None) => 1,
                None => releases.len(),
            };
            for release in releases.iter().take(n) {
                print_release_tree(release, detailed, no_color);
                println!();
            }
        }
    }
    Ok(())
}
fn print_release_tree(release: &Release, detailed: bool, no_color: bool) {
    let tag_name = if no_color {
        release.tag_name.to_string()
    } else {
        release.tag_name.bold().green().to_string()
    };
    println!("{}", tag_name);
    println!("{}", "=".repeat(release.tag_name.len()));

    for (i, asset) in release.assets.iter().enumerate() {
        let asset_name = if no_color {
            asset.name.to_string()
        } else {
            asset.name.bold().to_string()
        };
        println!("{}. {}", i + 1, asset_name);

        if detailed {
            let size = if no_color {
                format_size(asset.size, BINARY).to_string()
            } else {
                format_size(asset.size, BINARY).yellow().to_string()
            };
            let date = if no_color {
                asset.created_at.to_string()
            } else {
                asset.created_at.yellow().to_string()
            };
            println!("   {}: {}", "Size".cyan(), size);
            println!("   {}: {}", "Date".cyan(), date);
        }
        let url = if no_color {
            asset.browser_download_url.to_string()
        } else {
            asset.browser_download_url.blue().to_string()
        };
        println!("   {}: {}", "URL".cyan(), url);
        println!();
    }
}
async fn pull_command(
    repo_url: &str,
    output_dir: Option<&str>,
    tag: Option<&str>,
    all: bool,
    file: Option<&str>,
    token: Option<&str>,
    urls_only: bool,
) -> Result<()> {
    let (owner, repo) = parse_github_url(repo_url)
        .ok_or_else(|| anyhow::anyhow!("Invalid GitHub repository URL"))?;

    let github = GitHub::new(token)?;

    let release = match tag {
        Some(tag) => github.get_release(&owner, &repo, tag).await?,
        None => {
            let releases = github.get_releases(&owner, &repo).await?;
            releases
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No releases found"))?
        }
    };

    let assets = if all {
        release.assets
    } else if let Some(file_name) = file {
        release
            .assets
            .into_iter()
            .filter(|asset| asset.name == file_name)
            .collect()
    } else {
        return Err(anyhow::anyhow!("Either --all or a specific file must be specified"));
    };

    if urls_only {
        println!("Download URLs for release {}:", release.tag_name);
        for asset in assets {
            println!("{}: {}", asset.name, asset.browser_download_url);
        }
        return Ok(());
    }
    let output_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&output_dir)?;

    let mp = MultiProgress::new();
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-");

    let mut handles = Vec::new();
    let assets = Arc::new(assets);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS));
    for asset in assets.iter() {
        let permit = semaphore.clone().acquire_owned().await?;
        let pb = mp.add(ProgressBar::new(asset.size));
        pb.set_style(sty.clone());

        let github = github.clone();
        let output_path = output_dir.join(&asset.name);
        let url = asset.browser_download_url.clone();
        let pb = Arc::new(pb);

        let handle = tokio::spawn(async move {
            let result = github.download_asset(&url, &output_path, pb.clone()).await;
            drop(permit);
            result
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    mp.clear()?;
    println!("All downloads completed!");
    Ok(())
}
fn print_usage() {
    println!("ghrls view <user/repo or URL> [--tag <tag>] [-d/--detailed] [-n/--no-color] [--latest [N]]");
    println!();
    println!("ghrls pull <user/repo or URL> [--tag <tag>] (--all | <file>) [-o <dir>]");
}
#[tokio::main]
async fn main() -> Result<()> {
    let cli = if env::args().len() <= 1 {
        print_usage();
        std::process::exit(1);
    } else {
        Cli::parse()
    };
    let token = cli.pat.or_else(|| env::var("GH_TOKEN").ok());
    if token.is_some() {
        println!("Using GitHub token for authentication.");
    } else {
        println!("No GitHub token provided. Using unauthenticated requests.");
    }
    match cli.command {
        Commands::View {
            repo,
            tag,
            detailed,
            no_color,
            latest,
        } => {
            view_command(&repo, tag.as_deref(), token.as_deref(), detailed, no_color, latest)
                .await
                .context("Failed to view releases")?;
        }
        Commands::Pull {
            repo,
            output_dir,
            tag,
            all,
            file,
        } => {
            pull_command(
                &repo,
                output_dir.as_deref(),
                tag.as_deref(),
                all,
                file.as_deref(),
                token.as_deref(),
                false,
            )
            .await
            .context("Failed to pull release assets")?;
        }
    }
    Ok(())
}
