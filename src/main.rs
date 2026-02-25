use anyhow::{Context, Result};
use colored::*;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::env;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    public_repos: u32,
    bio: Option<String>,
    location: Option<String>,
    followers: u32,
    following: u32,
    avatar_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let username = match env::args().nth(1) {
        Some(u) => u,
        None => {
            eprintln!("Usage: githubfetch <username>");
            std::process::exit(1);
        }
    };

    let client = build_client()?;

    let user = fetch_user(&client, &username).await?;
    let starred_count = fetch_starred_count(&client, &username).await?;

    display_avatar(&user.avatar_url)?;
    display_user_info(&user, starred_count, &username);

    Ok(())
}

fn build_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("githubfetch-rust"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

async fn fetch_user(client: &reqwest::Client, username: &str) -> Result<GitHubUser> {
    let url = format!("https://api.github.com/users/{username}");

    let response = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .context("Failed to fetch user data")?;

    let user = response.json::<GitHubUser>().await?;
    Ok(user)
}

async fn fetch_starred_count(client: &reqwest::Client, username: &str) -> Result<usize> {
    let url = format!("https://api.github.com/users/{username}/starred");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Ok(0);
    }

    let starred: Vec<serde_json::Value> = response.json().await?;
    Ok(starred.len())
}

fn display_avatar(avatar_url: &str) -> Result<()> {
    let status = Command::new("kitty")
        .args([
            "+kitten",
            "icat",
            "--align",
            "left",
            "--scale-up",
            "--place",
            "20x20@0x2",
            avatar_url,
        ])
        .status();

    match status {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("{}", "Kitty terminal not installed!".red());
            std::process::exit(1);
        }
    }
}

fn display_user_info(user: &GitHubUser, starred: usize, username: &str) {
    let indent = " ".repeat(22);
    let github_url = format!("{username}@github.com");

    println!();
    println!("{} {}", indent, github_url);
    println!("{} {}", indent, "-".repeat(github_url.len()));

    println!(
        "{} {} {}",
        indent,
        "Username:".bright_blue(),
        user.login
    );

    println!(
        "{} {} {}",
        indent,
        "Repos:".yellow(),
        user.public_repos
    );

    println!(
        "{} {} {}",
        indent,
        "Bio:".green(),
        user.bio.clone().unwrap_or_else(|| "N/A".into())
    );

    println!(
        "{} {} {}",
        indent,
        "From:".red(),
        user.location.clone().unwrap_or_else(|| "Not Provided".into())
    );

    println!(
        "{} {} {}",
        indent,
        "Followers:".bright_red(),
        user.followers
    );

    println!(
        "{} {} {}",
        indent,
        "Following:".bright_blue(),
        user.following
    );

    println!(
        "{} {} {}",
        indent,
        "Starred repos:".yellow(),
        starred
    );

    println!();
}