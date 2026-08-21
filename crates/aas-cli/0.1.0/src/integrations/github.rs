use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubPR {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

pub struct GitHubClient {
    client: Client,
    token: String,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: &str) -> Self {
        GitHubClient {
            client: Client::new(),
            token: token.to_string(),
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        issue: &GitHubIssue,
    ) -> Result<String, String> {
        let url = format!("{}/repos/{}/{}/issues", self.base_url, owner, repo);

        let body = serde_json::json!({
            "title": issue.title,
            "body": issue.body,
            "labels": issue.labels,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "aas-agent")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("GitHub API error: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data["html_url"].as_str().unwrap_or("unknown").to_string())
    }

    pub async fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        pr: &GitHubPR,
    ) -> Result<String, String> {
        let url = format!("{}/repos/{}/{}/pulls", self.base_url, owner, repo);

        let body = serde_json::json!({
            "title": pr.title,
            "body": pr.body,
            "head": pr.head,
            "base": pr.base,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "aas-agent")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("GitHub API error: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data["html_url"].as_str().unwrap_or("unknown").to_string())
    }

    pub async fn test_connection(&self) -> Result<String, String> {
        let url = format!("{}/user", self.base_url);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "aas-agent")
            .send()
            .await
            .map_err(|e| format!("GitHub API error: {}", e))?;

        if resp.status().is_success() {
            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(format!(
                "Connected as {}",
                data["login"].as_str().unwrap_or("unknown")
            ))
        } else {
            Err(format!("GitHub API returned {}", resp.status()))
        }
    }
}
