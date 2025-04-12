use crate::config::SingleConfig;
use reqwest::{Client, Response};
use serde_json::Value;
use tracing::{info, error};
use std::error::Error;
use tokio::time::{sleep, Duration};

use crate::controller::send_to_controller;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

pub fn get_github_api_url(repo_owner: &str, repo_name: &str) -> String {
    format!("https://api.github.com/repos/{}/{}", repo_owner, repo_name)
}

pub fn get_github_repo_url(repo_owner: &str, repo_name: &str) -> String {
    format!("https://github.com/{}/{}", repo_owner, repo_name)
}

async fn request_github_api(url: &str, token: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let response: Response = client
        .get(url)
        .header("User-Agent", "rust-reqwest")
        .header("Authorization", format!("token {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        error!("GitHub API request failed: {:?}", response.status());
    }

    let res = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(res)
}

async fn get_latest_commit(config: &SingleConfig) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "{}/commits",
        get_github_api_url(&config.repo_owner, &config.repo_name)
    );
    let commits = request_github_api(&url, &config.github_token).await?;
    let latest_commit = match commits.get(0) {
        Some(commit) => commit["sha"].as_str().map(String::from),
        None => return Err("No commits found".into()),
    };
    let last_commit_sha = match latest_commit {
        Some(commit) => commit,
        None => return Err("Sha of latest commit not found".into()),
    };
    Ok(last_commit_sha)
}

async fn get_latest_tag(config: &SingleConfig) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "{}/tags",
        get_github_api_url(&config.repo_owner, &config.repo_name)
    );
    let tags = request_github_api(&url, &config.github_token).await?;
    let latest_tag = match tags.get(0) {
        Some(tag) => tag["name"].as_str().map(String::from),
        None => return Err("No tags found".into()),
    };
    let last_tag_name = match latest_tag {
        Some(tag) => tag,
        None => return Err("Name of latest tag not found".into()),
    };
    Ok(last_tag_name)
}

async fn get_latest_pull_request(config: &SingleConfig) -> Option<(u64, String)> {
    let url = format!("{}/pulls", get_github_api_url(&config.repo_owner, &config.repo_name));
    
    // Manually handle the Result
    let pull_requests = match request_github_api(&url, &config.github_token).await {
        Ok(data) => data,
        Err(e) => {
            error!("Error fetching pull requests: {}", e);
            return None; // Return None in case of error
        }
    };
    
    let pull_request_id = pull_requests.get(0)?.get("id")?.as_u64()?;
    let pull_request_title = pull_requests.get(0)?.get("title")?.as_str()?.to_string();
    Some((pull_request_id, pull_request_title))
}

pub async fn listen_to_commits(
    config: &SingleConfig,
    callback: impl Fn() + Send + 'static,
) -> Result<(), Box<dyn Error>> {
    let mut last_commit = get_latest_commit(config).await?;
    info!("Last commit found: {}", last_commit);

    loop {
        sleep(Duration::from_secs(10)).await;
        info!("{}/{} - Checking for new commits...", config.repo_owner, config.repo_name);

        match get_latest_commit(config).await {
            Ok(current_commit) => {
                if last_commit != current_commit {
                    info!("{}/{} - New commit found: {}", config.repo_owner, config.repo_name, current_commit);
                    last_commit = current_commit;
                    callback();
                }
            }
            Err(e) => {
                error!("Error fetching the latest commit: {}", e);
            }
        }
    }
}

pub async fn listen_to_tags(
    config: &SingleConfig,
    callback: impl Fn() + Send + 'static,
) -> Result<(), Box<dyn Error>> {
    let mut last_tag = get_latest_tag(config).await?;
    info!("Last tag found: {}", last_tag);

    loop {
        sleep(Duration::from_secs(10)).await;
        info!("{}/{} - Checking for new tags...", config.repo_owner, config.repo_name);

        match get_latest_tag(config).await {
            Ok(current_tag) => {
                if last_tag != current_tag {
                    info!("{}/{} - New tag found: {}", config.repo_owner, config.repo_name, current_tag);
                    last_tag = current_tag;
                    callback();
                }
            }
            Err(e) => {
                error!("Error fetching the latest tag: {}", e);
            }
        }
    }
}

pub async fn listen_to_pull_requests(
    config: &SingleConfig,
    callback: impl Fn() + Send + 'static
) {
    let mut last_pull_request = get_latest_pull_request(config).await;
    if let Some((id, title)) = &last_pull_request {
        info!("Last pull request found: {} - {}", id, title);
    }

    loop {
        sleep(Duration::from_secs(10)).await;
        info!("{}/{} - Checking for new pull requests...", config.repo_owner, config.repo_name);
        if let Some((current_pull_request, current_title)) = get_latest_pull_request(config).await {
            if Some(&(current_pull_request, current_title.clone())) != last_pull_request.as_ref() {
                info!("{}/{} - New pull request found: {} - {}", config.repo_owner, config.repo_name, current_pull_request, current_title);
                last_pull_request = Some((current_pull_request, current_title));
                callback();
            }
        }
    }
}

pub fn create_commit_listener(
    config: Arc<SingleConfig>,
    repo_url: String,
    controller_endpoint: Arc<String>,
) -> impl Future<Output = ()> {
    async move {
        if config.event == "commit" || config.event == "*" {
            let callback = create_callback(
                Arc::clone(&config),
                repo_url.clone(),
                Arc::clone(&controller_endpoint),
            );
            let _ = listen_to_commits(&config, callback).await;
        }
    }
}

pub fn create_pull_request_listener(
    config: Arc<SingleConfig>,
    repo_url: String,
    controller_endpoint: Arc<String>,
) -> impl Future<Output = ()> {
    async move {
        if config.event == "pull_request" || config.event == "*" {
            let callback = create_callback(
                Arc::clone(&config),
                repo_url.clone(),
                Arc::clone(&controller_endpoint),
            );
            let _ = listen_to_pull_requests(&config, callback).await;
        }
    }
}

pub fn create_tag_listener(
    config: Arc<SingleConfig>,
    repo_url: String,
    controller_endpoint: Arc<String>,
) -> impl Future<Output = ()> {
    async move {
        if config.event == "release" || config.event == "*" {
            let callback = create_callback(
                Arc::clone(&config),
                repo_url.clone(),
                Arc::clone(&controller_endpoint),
            );
            let _ = listen_to_tags(&config, callback).await;
        }
    }
}

pub fn create_callback(
    config: Arc<SingleConfig>,
    repo_url: String,
    controller_endpoint: Arc<String>,
) -> impl Fn() {
    move || {
        info!("Callback triggered");
        let config = Arc::clone(&config);
        let repo_url = repo_url.clone();
        let controller_endpoint_clone = controller_endpoint.clone();
        tokio::spawn(async move {
            info!("Sending pipeline to controller...");
            match send_to_controller(
                &repo_url,
                Path::new(&config.actions_path),
                controller_endpoint_clone,
            )
            .await
            {
                Ok(_) => info!("Pipeline sent successfully"),
                Err(e) => error!("Failed to send pipeline: {}", e),
            }
        });
    }
}
