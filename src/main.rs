/*
Copyright 2024 [rhythmcache]

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



/*
This implementation originates from
https://github.com/rhythmcache/gh-tree
*/



// Cargo.toml:
/*
[package]
name = "ghtree"
version = "0.2.0"
edition = "2021"

[dependencies]
tokio = { version = "1.36", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
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

/*

Alternatively we can also try this incase static linking is failing

[dependencies]
reqwest = { version = "0.11", features = ["json", "stream", "rustls-tls"], default-features = false }
tokio = { version = "1.36", features = ["full"] }
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

use anyhow::{anyhow, Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    time::Duration,
};

const GITHUB_API_URL: &str = "https://api.github.com";
const USER_AGENT: &str = "rhythmcache.t.me/gh-tree/0.2.0";

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
    // sha: Option<String>,
}


struct Config {
    api_token: Option<String>,
    client: reqwest::Client,
    colored_output: bool,
}

impl Config {
    fn new(colored_output: bool) -> Self {
        Self {
            api_token: None,
            client: reqwest::Client::new(),
            colored_output,
        }
    }

    fn with_token(token: String, colored_output: bool) -> Self {
        Self {
            api_token: Some(token),
            client: reqwest::Client::new(),
            colored_output,
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
    let url = repo_url.trim_end_matches(".git");
    let without_protocol = url.split("://").last()?;
    let parts: Vec<&str> = without_protocol.split('/').collect();
    
    if parts.len() < 3 || !parts[0].contains("github.com") {
        return None;
    }

    Some((parts[1].to_string(), parts[2].to_string()))
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
        return Err(anyhow!("Rate limit exceeded. Try using a GitHub token with --pat"));
    } else if status == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("Repository not found. Check the URL and permissions"));
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

    let tree_response: TreeResponse = response
        .json()
        .await
        .context("Failed to parse tree data")?;

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
    fs::create_dir_all(base_path).context("Failed to create base directory")?;
    
    for item in tree_items {
        let path = base_path.join(&item.path);
        
        match item.item_type.as_str() {
            "tree" => {
                fs::create_dir_all(&path)
                    .with_context(|| format!("Failed to create directory: {}", path.display()))?;
            }
            "blob" => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
                }
                File::create(&path)
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
            current = current
                .children
                .entry(part.to_string())
                .or_default();
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
            let connector = if is_last_item { "└── " } else { "├── " };
            
            let (icon, name_colored) = match child.item_type.as_deref() {
                Some("tree") => {
                    let icon = if colored { "📁".blue().to_string() } else { "".to_string() };
                    let name = if colored { name.blue().to_string() } else { name.to_string() };
                    (icon, name)
                }
                Some("blob") => {
                    let icon = if colored { "📄".green().to_string() } else { "".to_string() };
                    let name = if colored { name.green().to_string() } else { name.to_string() };
                    (icon, name)
                }
                _ => {
                    let icon = if colored { "❓".yellow().to_string() } else { "".to_string() };
                    let name = if colored { name.yellow().to_string() } else { name.to_string() };
                    (icon, name)
                }
            };
            
            println!("{}{}{} {}", prefix, connector, icon, name_colored);
            
            let new_prefix = format!(
                "{}{}",
                prefix,
                if is_last_item { "    " } else { "│   " }
            );
            print_nested(child, &new_prefix, is_last_item, colored);
        }
    }

    print_nested(&structure, "", true, colored);
    
    let summary = format!(
        "\nTotal folders: {}\nTotal files: {}",
        if colored { folder_count.to_string().blue().to_string() } else { folder_count.to_string() },
        if colored { file_count.to_string().green().to_string() } else { file_count.to_string() }
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


use std::io::Write;

// function to download a file or folder
async fn pull_file_or_folder(
    user: &str,
    repo: &str,
    branch: &str,
    path: &str,
    output_dir: Option<&Path>,
    config: &Config,
    progress: &ProgressBar,
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

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("GitHub API error {}: {}", status, error_body));
    }

    let content: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse file/folder info")?;

    let output_path = if let Some(dir) = output_dir {
        dir.join(path)
    } else {
        PathBuf::from(path)
    };

    if content.is_array() {
        // recursively download contents
        for item in content.as_array().unwrap() {
            let item_path = item["path"].as_str().unwrap();
            let item_type = item["type"].as_str().unwrap();

            match item_type {
                "file" => {
                    let download_url = item["download_url"].as_str().unwrap();
                    let file_path = if let Some(dir) = output_dir {
                        dir.join(item_path)
                    } else {
                        PathBuf::from(item_path)
                    };

                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).context("Failed to create parent directory")?;
                    }

                    let file_response = config
                        .client
                        .get(download_url)
                        .send()
                        .await
                        .context("Failed to download file")?;

                    let mut file = File::create(&file_path).context("Failed to create file")?;
                    let content = file_response
                        .bytes()
                        .await
                        .context("Failed to read file content")?;
                    file.write_all(&content).context("Failed to write file content")?;

                    progress.inc(1);
                    progress.set_message(format!("Downloaded: {}", item_path));
                }
                "dir" => {
                    Box::pin(pull_file_or_folder(
                        user,
                        repo,
                        branch,
                        item_path,
                        output_dir,
                        config,
                        progress,
                    ))
                    .await?;
                }
                _ => {
                    return Err(anyhow!("Unknown item type: {}", item_type));
                }
            }
        }
    } else {
        // It's a single file
        let download_url = content["download_url"].as_str().unwrap();
        let file_response = config
            .client
            .get(download_url)
            .send()
            .await
            .context("Failed to download file")?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }

        let mut file = File::create(&output_path).context("Failed to create file")?;
        let content = file_response
            .bytes()
            .await
            .context("Failed to read file content")?;
        file.write_all(&content).context("Failed to write file content")?;

        progress.inc(1);
        progress.set_message(format!("Downloaded: {}", path));
    }

    progress.finish_with_message("Download completed");
    Ok(())
}


// Function to Download repo as zip file
use std::io::copy;
// use reqwest::StatusCode;
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

    let response = config
        .client
        .get(&url)
        .headers(config.get_headers())
        .send()
        .await
        .context("Failed to download repository zip")?;

    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!(
            "Failed to download repository zip: {}",
            response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
        ));
    }

    let output_file = if let Some(dir) = output_dir {
        dir.join(format!("{}-{}.zip", repo, branch))
    } else {
        PathBuf::from(format!("{}-{}.zip", repo, branch))
    };

    let mut file = File::create(&output_file).context("Failed to create output file")?;

    // Download the zip file
    let content = response.bytes().await.context("Failed to read zip content")?;
    copy(&mut content.as_ref(), &mut file).context("Failed to write zip content")?;

    progress.finish_with_message(format!(
        "Repository downloaded as zip: {}",
        output_file.display()
    ));

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  ghtree touch <output-path> <GitHub Repository URL> [branch]");
    println!("  ghtree view <GitHub Repository URL> [branch] [-f <folder>] [-c]");
    println!("  ghtree pull [-o <output-directory>] <GitHub Repository URL> [branch] <file/folder to pull>");
    println!("  ghtree -dl [-o <output-directory>] <GitHub Repository URL> [branch]");
    println!("  ghtree --pat <token> [commands...]\n");
    println!("Options:");
    println!("  -c    Enable colored output with icons");
    println!("  -f    View a specific folder in the repository");
}

use std::env;
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    // Check for color flag
    let colored_output = args.iter().any(|arg| arg == "-c");

    // Read GH_TOKEN from environment
    let env_token = env::var("GH_TOKEN").ok();

    // Handle PAT token (--pat argument overrides GH_TOKEN)
    let (config, arg_offset) = if args.len() > 2 && args[1] == "--pat" {
        let pat_token = args[2].clone();
        println!("Using provided PAT token to fetch data.");
        (Config::with_token(pat_token, colored_output), 2)
    } else if let Some(token) = env_token {
        println!("Using GH_TOKEN from environment to fetch data.");
        (Config::with_token(token, colored_output), 0)
    } else {
        // println!("No token provided. Using unauthenticated requests (rate limits may apply).");
        (Config::new(colored_output), 0)
    };

    let effective_args: Vec<_> = args
        .iter()
        .skip(1 + arg_offset)
        .filter(|&arg| arg != "-c")
        .collect();

    if effective_args.is_empty() {
        print_usage();
        return Ok(());
    }
  
    match effective_args[0].as_str() {
        "touch" => {
            if effective_args.len() < 3 {
                return Err(anyhow!("Missing arguments for 'touch' command"));
            }

            let base_path = PathBuf::from(&effective_args[1]);
            let repo_url = &effective_args[2];
            let branch = effective_args.get(3).map(|s| s.as_str());

            let (user, repo) = parse_github_url(repo_url)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Fetching repository information...");

            let branch = if let Some(b) = branch {
                b.to_string()
            } else {
                fetch_repo_info(&user, &repo, &config)
                    .await?
                    .default_branch
            };

            progress.set_message("Fetching tree data...");
            let tree_items = fetch_tree_recursive(&user, &repo, &branch, &config, &progress).await?;

            progress.set_message("Creating directory structure...");
            create_placeholder_structure(tree_items, &base_path, &progress).await?;
        }
        
        "view" => {
    if effective_args.len() < 2 {
        return Err(anyhow!("Missing arguments for 'view' command"));
    }

    let repo_url = &effective_args[1];
    let mut branch = None;
    let mut folder = None;

    let mut args_iter = effective_args.iter().skip(2);
    while let Some(arg) = args_iter.next() {
        if *arg == "-f" {
            folder = args_iter.next().map(|s| s.as_str());
        } else if branch.is_none() {
            branch = Some(arg.as_str());
        }
    }

    let (user, repo) = parse_github_url(repo_url)
        .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

    let progress = create_progress_bar("Fetching repository information...");

    let branch = if let Some(b) = branch {
        b.to_string()
    } else {
        fetch_repo_info(&user, &repo, &config)
            .await?
            .default_branch
    };

    progress.set_message("Fetching tree data...");
    let tree_items = fetch_tree_recursive(&user, &repo, &branch, &config, &progress).await?;

    // Filter tree items if a folder is specified
    let filtered_tree_items = if let Some(folder_path) = folder {
        tree_items
            .into_iter()
            .filter(|item| item.path.starts_with(folder_path))
            .collect()
    } else {
        tree_items
    };

    progress.set_message("Building tree view...");
    print_tree_colored(filtered_tree_items, &progress, config.colored_output)?;
}
        
        "pull" => {
            if effective_args.len() < 3 {
                return Err(anyhow!("Missing arguments for 'pull' command"));
            }

            let output_dir = if effective_args[1] == "-o" {
                Some(PathBuf::from(&effective_args[2]))
            } else {
                None
            };

            let repo_url = if output_dir.is_some() {
                &effective_args[3]
            } else {
                &effective_args[1]
            };

            let branch = if output_dir.is_some() {
                effective_args.get(4).map(|s| s.as_str())
            } else {
                effective_args.get(2).map(|s| s.as_str())
            };

            let file_to_pull = if output_dir.is_some() {
                effective_args.get(5).map(|s| s.as_str())
            } else {
                effective_args.get(3).map(|s| s.as_str())
            };

            let (user, repo) = parse_github_url(repo_url)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Fetching repository information...");

            let branch = if let Some(b) = branch {
                b.to_string()
            } else {
                fetch_repo_info(&user, &repo, &config)
                    .await?
                    .default_branch
            };

            let file_to_pull = file_to_pull.ok_or_else(|| anyhow!("Missing file/folder to pull"))?;

            progress.set_message("Downloading file/folder...");
            pull_file_or_folder(
                &user,
                &repo,
                &branch,
                file_to_pull,
                output_dir.as_deref(),
                &config,
                &progress,
            )
            .await?;
        }

        "-dl" => {
            if effective_args.len() < 2 {
                return Err(anyhow!("Missing arguments for '-dl' command"));
            }

            let output_dir = if effective_args.len() > 2 && effective_args[1] == "-o" {
                Some(PathBuf::from(&effective_args[2]))
            } else {
                None
            };

            let repo_url = if output_dir.is_some() {
                &effective_args[3]
            } else {
                &effective_args[1]
            };

            let branch = if output_dir.is_some() {
                effective_args.get(4).map(|s| s.as_str())
            } else {
                effective_args.get(2).map(|s| s.as_str())
            };

            let (user, repo) = parse_github_url(repo_url)
                .ok_or_else(|| anyhow!("Invalid GitHub URL"))?;

            let progress = create_progress_bar("Downloading repository zip...");

            let branch = branch.unwrap_or("main");

            download_repo_zip(&user, &repo, branch, output_dir.as_deref(), &config, &progress)
                .await?;
        }

        cmd => return Err(anyhow!("Unsupported command: {}", cmd)),
    }

    Ok(())
}
