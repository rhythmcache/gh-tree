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
name = "ghtree"
version = "0.3.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.36", features = ["full"] }
# reqwest = { version = "0.11", features = ["json", "stream"] }
reqwest = { version = "0.11", features = ["json", "stream", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
anyhow = "1.0"
directories = "5.0"
indicatif = "0.17"
async-stream = "0.3"
colored = "2.1"
tokio-stream = "0.1"

*/
use clap::{Parser, Subcommand};
use std::env;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::time::sleep;
use std::sync::Arc;
use anyhow::{anyhow, Context, Result};
use colored::*;
use tokio_stream::StreamExt;
use std::collections::VecDeque;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs;
use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
    time::Duration,
};
const GITHUB_API_URL: &str = "https://api.github.com";
const USER_AGENT: &str = "rhythmcache.t.me/gh-tree/0.2.0";
const MAX_RETRIES: u32 = 16;
const INITIAL_DELAY: Duration = Duration::from_secs(3);
#[derive(Debug, Deserialize)]
struct RepoInfo {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct TreeResponse {
    tree: Vec<TreeItem>,
    truncated: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct TreeItem {
    path: String,
    #[serde(rename = "type")]
    item_type: String,
}

#[derive(Debug, Deserialize)]
struct GitHubContentItem {
    path: String,
    #[serde(rename = "type")]
    item_type: String,
    download_url: Option<String>,
}

struct Config {
    api_token: Option<String>,
    client: reqwest::Client,
}

impl Config {
    fn new() -> Self {
        Self {
            api_token: None,
            client: reqwest::Client::new(),
        }
    }

    fn with_token(token: String) -> Self {
        Self {
            api_token: Some(token),
            client: reqwest::Client::new(),
        }
    }

    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );
        headers.insert(reqwest::header::USER_AGENT, USER_AGENT.parse().unwrap());

        if let Some(token) = &self.api_token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        headers
    }
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

async fn fetch_repo_info(user: &str, repo: &str, config: &Config) -> Result<RepoInfo> {
    let url = format!("{}/repos/{}/{}", GITHUB_API_URL, user, repo);

    let response = config
        .client
        .get(&url)
        .headers(config.get_headers())
        .send()
        .await
        .context("Failed to fetch repository info")?;

    let status = response.status();

    if status == reqwest::StatusCode::FORBIDDEN {
        return Err(anyhow!(
            "Rate limit exceeded. Try using a GitHub token with --pat"
        ));
    } else if status == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!(
            "Repository not found. Check the URL and permissions"
        ));
    } else if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        let error_msg = serde_json::from_str::<serde_json::Value>(&error_body)
            .ok()
            .and_then(|j| j.get("message").and_then(|m| m.as_str().map(String::from)))
            .unwrap_or_else(|| "Unknown error".to_string());

        return Err(anyhow!("GitHub API error {}: {}", status, error_msg));
    }

    response
        .json::<RepoInfo>()
        .await
        .context("Failed to parse repository info")
}

async fn fetch_tree_recursive(
    user: &str,
    repo: &str,
    sha: &str,
    config: &Config,
    progress: &ProgressBar,
) -> Result<Vec<TreeItem>> {
    let url = format!(
        "{}/repos/{}/{}/git/trees/{}?recursive=1",
        GITHUB_API_URL, user, repo, sha
    );

    let response = config
        .client
        .get(&url)
        .headers(config.get_headers())
        .send()
        .await
        .context("Failed to fetch tree data")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        return Err(anyhow!("GitHub API error {}: {}", status, error_body));
    }

    let tree_response: TreeResponse = response.json().await.context("Failed to parse tree data")?;

    if tree_response.truncated {
        progress.println("Warning: Repository tree is truncated due to size limitations");
    }

    Ok(tree_response.tree)
}

async fn create_placeholder_structure(
    tree_items: Vec<TreeItem>,
    base_path: &Path,
    progress: &ProgressBar,
) -> Result<()> {
    let mut created_dirs = HashSet::new();
    fs::create_dir_all(base_path).context("Failed to create base directory")?;
    created_dirs.insert(base_path.to_path_buf());

    for item in tree_items {
        let path = base_path.join(&item.path);

        match item.item_type.as_str() {
            "tree" => {
                if !created_dirs.contains(&path) {
                    fs::create_dir_all(&path).with_context(|| {
                        format!("Failed to create directory: {}", path.display())
                    })?;
                    created_dirs.insert(path);
                }
            }
            "blob" => {
                if let Some(parent) = path.parent() {
                    if !created_dirs.contains(parent) {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("Failed to create parent directory: {}", parent.display())
                        })?;
                        created_dirs.insert(parent.to_path_buf());
                    }
                }
                tokio::fs::File::create(&path)
    .await
    .with_context(|| format!("Failed to create file: {}", path.display()))?;
            }
            _ => continue,
        }

        progress.inc(1);
        progress.set_message(format!("Processing: {}", item.path));
    }

    progress.finish_with_message(format!("Structure created at: {}", base_path.display()));
    Ok(())
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    item_type: Option<String>,
}

fn print_tree_colored(
    tree_items: Vec<TreeItem>,
    progress: &ProgressBar,
    colored: bool,
) -> Result<()> {
    let mut structure = TreeNode::default();
    let mut file_count = 0;
    let mut folder_count = 0;

    for item in tree_items {
        let mut current = &mut structure;

        for part in item.path.split('/') {
            current = current.children.entry(part.to_string()).or_default();
        }

        current.item_type = Some(item.item_type.clone());

        match item.item_type.as_str() {
            "blob" => file_count += 1,
            "tree" => folder_count += 1,
            _ => {}
        }

        progress.inc(1);
        progress.set_message(format!("Processing: {}", item.path));
    }

    progress.finish_and_clear();

    fn print_nested(node: &TreeNode, prefix: &str, _is_last: bool, colored: bool) {
        let items: Vec<_> = node.children.iter().collect();

        for (i, (name, child)) in items.iter().enumerate() {
            let is_last_item = i == items.len() - 1;
            let connector = if is_last_item {
                "└── "
            } else {
                "├── "
            };

            let (icon, name_colored) = match child.item_type.as_deref() {
                Some("tree") => (
                    if colored {
                        "📁".blue().to_string()
                    } else {
                        "".to_string()
                    },
                    if colored {
                        name.blue().to_string()
                    } else {
                        name.to_string()
                    },
                ),
                Some("blob") => (
                    if colored {
                        "📄".green().to_string()
                    } else {
                        "".to_string()
                    },
                    if colored {
                        name.green().to_string()
                    } else {
                        name.to_string()
                    },
                ),
                _ => (
                    if colored {
                        "❓".yellow().to_string()
                    } else {
                        "".to_string()
                    },
                    if colored {
                        name.yellow().to_string()
                    } else {
                        name.to_string()
                    },
                ),
            };

            println!("{}{}{} {}", prefix, connector, icon, name_colored);

            let new_prefix = format!("{}{}", prefix, if is_last_item { "    " } else { "│   " });
            print_nested(child, &new_prefix, is_last_item, colored);
        }
    }

    print_nested(&structure, "", true, colored);

    let summary = format!(
        "\nTotal folders: {}\nTotal files: {}",
        if colored {
            folder_count.to_string().blue().to_string()
        } else {
            folder_count.to_string()
        },
        if colored {
            file_count.to_string().green().to_string()
        } else {
            file_count.to_string()
        }
    );

    println!(
        "{}",
        if colored {
            summary
        } else {
            summary.normal().to_string()
        }
    );

    Ok(())
}

fn create_progress_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

async fn download_file_with_retry(
    url: &str, 
    file_path: &Path, 
    config: &Config, 
    progress: &ProgressBar,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<()> {
    let mut retries = 0;
    let mut delay = initial_delay;

    loop {
        match download_file(url, file_path, config, progress).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if retries >= max_retries {
                    return Err(anyhow!("Failed after {} retries: {}", max_retries, e));
                }
                
                retries += 1;
                progress.println(format!(
                    "Download failed, retrying ({}/{}): {}", 
                    retries, max_retries, e
                ));
                
                if e.to_string().contains("HTTP 403 Forbidden") {
                    progress.println("403 Forbidden error. Check your token permissions and rate limits.");
                }
                
                sleep(delay).await;
                delay *= 2;
            }
        }
    }
}

async fn download_file(
    url: &str, 
    file_path: &Path, 
    config: &Config, 
    progress: &ProgressBar,
) -> Result<()> {
    let response = config.client.get(url).send().await.context("Failed to download file")?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download file: HTTP {}", response.status()));
    }

    let mut file = BufWriter::new(tokio::fs::File::create(file_path).await.context("Failed to create file")?);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk")?;
        file.write_all(&chunk).await.context("Failed to write chunk")?;
        progress.inc(chunk.len() as u64);
    }

    file.flush().await.context("Failed to flush file")?;
    progress.set_message(format!("Downloaded: {}", file_path.display()));

    Ok(())
}


async fn pull_file_or_folder(
    user: &str,
    repo: &str,
    branch: &str,
    path: &str,
    output_dir: Option<&Path>,
    config: Arc<Config>,
    progress: Arc<ProgressBar>,
) -> Result<()> {
    let url = format!(
        "{}/repos/{}/{}/contents/{}?ref={}",
        GITHUB_API_URL, user, repo, path, branch
    );

    let response = config
        .client
        .get(&url)
        .headers(config.get_headers())
        .send()
        .await
        .context("Failed to fetch file/folder info")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API error {}: {}",
            response.status(),
            response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
        ));
    }

    let response_body = response.text().await.context("Failed to read response body")?;

    if let Ok(file_info) = serde_json::from_str::<GitHubContentItem>(&response_body) {
        if file_info.item_type == "file" {
            if let Some(download_url) = file_info.download_url {
                let file_path = output_dir
                    .map(|dir| dir.join(&file_info.path))
                    .unwrap_or_else(|| PathBuf::from(&file_info.path));

                if let Some(parent) = file_path.parent() {
                    tokio::fs::create_dir_all(parent).await.context("Failed to create parent directory")?;
                }
                download_file_with_retry(
                    &download_url,
                    &file_path,
                    &config,
                    &progress,
                    MAX_RETRIES,
                    INITIAL_DELAY,
                )
                .await?;
            }
        } else {
            return Err(anyhow!("Path is not a file: {}", path));
        }
    } else {
        let content: Vec<GitHubContentItem> =
            serde_json::from_str(&response_body).context("Failed to parse file/folder info")?;

        let mut tasks = VecDeque::new();
        for item in content {
            let config = Arc::clone(&config);
            let progress = Arc::clone(&progress);
            let output_dir = output_dir.map(|p| p.to_path_buf());

            tasks.push_back((item, config, progress, output_dir));
        }

        while let Some((item, config, progress, output_dir)) = tasks.pop_front() {
            let file_path = output_dir
                .as_ref()
                .map(|dir| dir.join(&item.path))
                .unwrap_or_else(|| PathBuf::from(&item.path));

            match item.item_type.as_str() {
                "file" => {
                    if let Some(download_url) = item.download_url {
                        if let Some(parent) = file_path.parent() {
                            tokio::fs::create_dir_all(parent)
                                .await
                                .context("Failed to create parent directory")?;
                        }
                        download_file_with_retry(
                            &download_url,
                            &file_path,
                            &config,
                            &progress,
                            MAX_RETRIES,
                            INITIAL_DELAY,
                        )
                        .await?;
                    }
                }
                "dir" => {
                    let subdir_url = format!(
                        "{}/repos/{}/{}/contents/{}?ref={}",
                        GITHUB_API_URL, user, repo, item.path, branch
                    );

                    let sub_response = config
                        .client
                        .get(&subdir_url)
                        .headers(config.get_headers())
                        .send()
                        .await
                        .context("Failed to fetch subdirectory info")?;

                    if !sub_response.status().is_success() {
                        return Err(anyhow!(
                            "GitHub API error {}: {}",
                            sub_response.status(),
                            sub_response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
                        ));
                    }

                    let sub_content: Vec<GitHubContentItem> = sub_response
                        .json()
                        .await
                        .context("Failed to parse subdirectory info")?;

                    for sub_item in sub_content {
                        tasks.push_back((sub_item, Arc::clone(&config), Arc::clone(&progress), output_dir.clone()));
                    }
                }
                _ => return Err(anyhow!("Unknown item type: {}", item.item_type)),
            }
        }
    }

    progress.finish_with_message("Download completed");
    Ok(())
}

async fn download_repo_zip(
    user: &str,
    repo: &str,
    branch: &str,
    output_dir: Option<&Path>,
    config: &Config,
    progress: &ProgressBar,
) -> Result<()> {
    let url = format!(
        "{}/repos/{}/{}/zipball/{}",
        GITHUB_API_URL, user, repo, branch
    );

    let output_file = if let Some(dir) = output_dir {
        dir.join(format!("{}-{}.zip", repo, branch))
    } else {
        PathBuf::from(format!("{}-{}.zip", repo, branch))
    };

    // Retry mechanism
    let mut retries = 0;
    let max_retries = 16; // Maximum number of retries
    let initial_delay = Duration::from_secs(3); // Initial delay between retries

    loop {
        match download_repo_zip_internal(&url, &output_file, config, progress).await {
            Ok(_) => {
                progress.finish_with_message(format!(
                    "Repository downloaded as zip: {}",
                    output_file.display()
                ));
                return Ok(());
            }
            Err(e) => {
                if retries >= max_retries {
                    return Err(anyhow!(
                        "Failed after {} retries: {}",
                        max_retries,
                        e
                    ));
                }

                retries += 1;
                progress.println(format!(
                    "Download failed, retrying ({}/{}): {}",
                    retries, max_retries, e
                ));

                // Exponential backoff
                sleep(initial_delay * retries).await;
            }
        }
    }
}

async fn download_repo_zip_internal(
    url: &str,
    output_file: &Path,
    config: &Config,
    progress: &ProgressBar,
) -> Result<()> {
    let response = config
        .client
        .get(url)
        .headers(config.get_headers())
        .send()
        .await
        .context("Failed to send request to GitHub API")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        return Err(anyhow!(
            "Failed to download repository zip: HTTP {} - {}",
            status,
            error_body
        ));
    }

    // Create the output file
    let mut file = tokio::fs::File::create(output_file)
        .await
        .context("Failed to create output file")?;

    // Stream the response directly to the file
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk from response")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk to file")?;
        progress.inc(chunk.len() as u64);
    }

    file.flush().await.context("Failed to flush file")?;

    Ok(())
}

async fn find_file_in_repo(
    user: &str,
    repo: &str,
    filename: &str,
    branch: Option<&str>,
    config: &Config,
    progress: &ProgressBar,
    exact_match: bool,
) -> Result<()> {
    let branches = if let Some(branch) = branch {
        vec![branch.to_string()]
    } else {
        // Fetch all branches if no specific branch is provided
        let url = format!("{}/repos/{}/{}/branches", GITHUB_API_URL, user, repo);
        let response = config
            .client
            .get(&url)
            .headers(config.get_headers())
            .send()
            .await
            .context("Failed to fetch branches")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch branches: {}",
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        let branches: Vec<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse branches")?;

        branches
            .into_iter()
            .filter_map(|b| b.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
            .collect()
    };

    for branch in branches {
        progress.set_message(format!("Searching in branch: {}", branch));
        let tree_items = fetch_tree_recursive(user, repo, &branch, config, progress).await?;

        for item in tree_items {
            // Extract the filename from the path
            if let Some(file_name) = item.path.split('/').last() {
                // Check for exact or partial match based on the flag
                if (exact_match && file_name == filename) || (!exact_match && file_name.contains(filename)) {
                    println!("Found: {} in branch: {}", item.path, branch);
                }
            }
        }
    }

    progress.finish_with_message("Search completed");
    Ok(())
}

#[derive(Parser)]
#[command(name = "ghtree")]
#[command(author = "rhythmcache")]
#[command(version = "0.4.0")]
#[command(about = "Command Line Utility That Uses Github API", long_about = None)]
struct Cli {
    /// GitHub Personal Access Token (can also use GH_TOKEN env var)
    #[arg(long = "pat", global = true)]
    pat: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View repository structure
    View {
        /// Repository URL or owner/repo format
        #[arg(short = 'r', long = "repo", required = true)]
        repo: String,

        /// Branch name (default: repository's default branch)
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        /// Specific folder to view
        #[arg(short = 'f', long = "folder")]
        folder: Option<String>,

        /// Enable colored output with icons
        #[arg(short = 'c', long = "color")]
        color: bool,
    },

    /// Create empty directory structure
    Touch {
        /// Repository URL or owner/repo format
        #[arg(short = 'r', long = "repo", required = true)]
        repo: String,

        /// Output directory path
        #[arg(short = 'o', long = "output", required = true)]
        output: String,

        /// Branch name (default: repository's default branch)
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,
    },

    /// Pull specific file or folder
    Pull {
        /// Repository URL or owner/repo format
        #[arg(short = 'r', long = "repo", required = true)]
        repo: String,

        /// File or folder path to pull
        #[arg(short = 'f', long = "path", required = true)]
        path: String,

        /// Branch name (default: repository's default branch)
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        /// Output directory
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },

    /// Download repository as zip
    Download {
        /// Repository URL or owner/repo format
        #[arg(short = 'r', long = "repo", required = true)]
        repo: String,

        /// Branch name (default: repository's default branch)
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        /// Output directory
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
    },

    /// Find file in repository
    Find {
        /// Repository URL or owner/repo format
        #[arg(short = 'r', long = "repo", required = true)]
        repo: String,

        /// Filename to search for
        #[arg(short = 'f', long = "filename", required = true)]
        filename: String,

        /// Branch name (default: repository's default branch)
        #[arg(short = 'b', long = "branch")]
        branch: Option<String>,

        #[arg(long = "exact")]
        exact: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = if let Some(token) = cli.pat.or_else(|| env::var("GH_TOKEN").ok()) {
        println!("Using provided PAT token to fetch data.");
        Arc::new(Config::with_token(token))
    } else {
        Arc::new(Config::new())
    };

    match cli.command {
        Commands::View {
            repo,
            branch,
            folder,
            color, // Add `color` here
        } => {
            let (user, repo_name) = parse_github_url(&repo)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Fetching repository information...");

            let branch = if let Some(b) = branch {
                b
            } else {
                fetch_repo_info(&user, &repo_name, &config).await?.default_branch
            };

            progress.set_message("Fetching tree data...");
            let tree_items = fetch_tree_recursive(&user, &repo_name, &branch, &config, &progress).await?;

            let filtered_tree_items = if let Some(folder_path) = folder {
                tree_items
                    .into_iter()
                    .filter(|item| item.path.starts_with(&folder_path))
                    .collect()
            } else {
                tree_items
            };

            progress.set_message("Building tree view...");
            print_tree_colored(filtered_tree_items, &progress, color)?; // Use `color` here
        }

        Commands::Touch { repo, output, branch } => {
            let base_path = PathBuf::from(output);
            let (user, repo_name) = parse_github_url(&repo)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Fetching repository information...");

            let branch = if let Some(b) = branch {
                b
            } else {
                fetch_repo_info(&user, &repo_name, &config).await?.default_branch
            };

            progress.set_message("Fetching tree data...");
            let tree_items = fetch_tree_recursive(&user, &repo_name, &branch, &config, &progress).await?;

            progress.set_message("Creating directory structure...");
            create_placeholder_structure(tree_items, &base_path, &progress).await?;
        }

        Commands::Pull { repo, path, branch, output } => {
            let (user, repo_name) = parse_github_url(&repo)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = Arc::new(create_progress_bar("Fetching repository information..."));

            let branch = if let Some(b) = branch {
                b
            } else {
                fetch_repo_info(&user, &repo_name, &config).await?.default_branch
            };

            progress.set_message("Downloading file/folder...");
            pull_file_or_folder(
                &user,
                &repo_name,
                &branch,
                &path,
                output.as_ref().map(PathBuf::from).as_deref(),
                config,
                progress,
            )
            .await?;
        }

        Commands::Download { repo, branch, output } => {
            let (user, repo_name) = parse_github_url(&repo)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Downloading repository zip...");

            let branch = if let Some(b) = branch {
                b
            } else {
                fetch_repo_info(&user, &repo_name, &config).await?.default_branch
            };

            download_repo_zip(
                &user,
                &repo_name,
                &branch,
                output.as_ref().map(PathBuf::from).as_deref(),
                &config,
                &progress,
            )
            .await?;
        }

        Commands::Find {
            repo,
            filename,
            branch,
            exact,
        } => {
            let (user, repo_name) = parse_github_url(&repo)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Searching for file...");

            find_file_in_repo(
                &user,
                &repo_name,
                &filename,
                branch.as_deref(),
                &config,
                &progress,
                exact,
            )
            .await?;
        }
    }

    Ok(())
}
