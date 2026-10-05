mod ui;

use std::process::Command;

use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RepoData {
    pub(crate) name: String,
    #[serde(rename = "nameWithOwner")]
    pub(crate) name_with_owner: String,
    pub(crate) visibility: RepoType,
    pub(crate) description: Option<String>,
    #[serde(rename = "stargazerCount")]
    pub(crate) stargazer_count: u64,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RepoType {
    Public,
    Private,
}

impl RepoType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::Private => "PRIVATE",
        }
    }

    fn as_cli_value(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

fn repo_edit_args(repo: &RepoData) -> [&str; 6] {
    [
        "repo",
        "edit",
        &repo.name_with_owner,
        "--visibility",
        repo.visibility.as_cli_value(),
        "--accept-visibility-change-consequences",
    ]
}

fn get_repos() -> Result<Vec<RepoData>> {
    let output = Command::new("gh")
        .args([
            "repo",
            "list",
            "--limit",
            "1000",
            "--json",
            "name,nameWithOwner,visibility,description,stargazerCount",
        ])
        .output()
        .wrap_err("failed to execute gh command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("failed to execute gh repo list: {}", stderr.trim()));
    }

    serde_json::from_slice(&output.stdout).wrap_err("failed to parse json")
}

pub(crate) fn apply_changes<'a>(diff_data: impl Iterator<Item = &'a RepoData>) -> Result<()> {
    for repo in diff_data {
        let output = Command::new("gh")
            .args(repo_edit_args(repo))
            .output()
            .wrap_err_with(|| format!("failed to update visibility for {}", repo.name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(eyre!(
                "gh repo edit failed for {}: {}",
                repo.name,
                stderr.trim()
            ));
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut repos = get_repos()?;
    ratatui::run(|terminal| ui::run(terminal, &mut repos))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repository_data_and_builds_owner_qualified_edit_args() {
        let repo: RepoData = serde_json::from_str(
            r#"{
                "name": "repo-vision",
                "nameWithOwner": "kurosiko/repo-vision",
                "visibility": "PRIVATE",
                "description": null,
                "stargazerCount": 3
            }"#,
        )
        .expect("GitHub repository JSON should deserialize");

        assert_eq!(repo.name, "repo-vision");
        assert_eq!(repo.name_with_owner, "kurosiko/repo-vision");
        assert_eq!(repo.visibility.as_cli_value(), "private");
        assert_eq!(
            repo_edit_args(&repo),
            [
                "repo",
                "edit",
                "kurosiko/repo-vision",
                "--visibility",
                "private",
                "--accept-visibility-change-consequences",
            ]
        );
    }
}
