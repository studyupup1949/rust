use crate::types::*;
use reqwest::Client;
use serde_json::Value;

pub struct GitHubClient {
    client: Client,
    token: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self { client: Client::new(), token }
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let url = if path.starts_with("https://") { path.to_string() } else { format!("https://api.github.com{}", path) };
        self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "mcp-github/1.0")
            .header("Accept", "application/vnd.github+json")
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("https://api.github.com{}", path);
        self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "mcp-github/1.0")
            .header("Accept", "application/vnd.github+json")
            .json(body)
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())
    }

    async fn patch(&self, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("https://api.github.com{}", path);
        self.client.patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "mcp-github/1.0")
            .header("Accept", "application/vnd.github+json")
            .json(body)
            .send().await.map_err(|e| e.to_string())?
            .json::<Value>().await.map_err(|e| e.to_string())
    }

    pub async fn search_repositories(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let v = self.get(&format!("/search/repositories?q={}&per_page=10", urlencoded(query))).await?;
        Ok(v["items"].as_array().unwrap_or(&vec![]).iter().map(|i| SearchResult {
            full_name: i["full_name"].as_str().unwrap_or("").to_string(),
            description: i["description"].as_str().map(|s| s.to_string()),
            html_url: i["html_url"].as_str().unwrap_or("").to_string(),
            stargazers_count: i["stargazers_count"].as_u64().unwrap_or(0),
        }).collect())
    }

    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository, String> {
        let v = self.get(&format!("/repos/{}/{}", owner, repo)).await?;
        Ok(Repository {
            full_name: v["full_name"].as_str().unwrap_or("").to_string(),
            description: v["description"].as_str().map(|s| s.to_string()),
            html_url: v["html_url"].as_str().unwrap_or("").to_string(),
            default_branch: v["default_branch"].as_str().unwrap_or("main").to_string(),
            language: v["language"].as_str().map(|s| s.to_string()),
            stargazers_count: v["stargazers_count"].as_u64().unwrap_or(0),
            open_issues_count: v["open_issues_count"].as_u64().unwrap_or(0),
            private: v["private"].as_bool().unwrap_or(false),
        })
    }

    pub async fn list_branches(&self, owner: &str, repo: &str) -> Result<Vec<Branch>, String> {
        let v = self.get(&format!("/repos/{}/{}/branches?per_page=30", owner, repo)).await?;
        Ok(v.as_array().unwrap_or(&vec![]).iter().map(|b| Branch {
            name: b["name"].as_str().unwrap_or("").to_string(),
            protected: b["protected"].as_bool().unwrap_or(false),
        }).collect())
    }

    pub async fn get_file_contents(&self, owner: &str, repo: &str, path: &str, branch: Option<&str>) -> Result<FileContent, String> {
        let ref_param = branch.map(|b| format!("?ref={}", b)).unwrap_or_default();
        let v = self.get(&format!("/repos/{}/{}/contents/{}{}", owner, repo, path, ref_param)).await?;
        let content = v["content"].as_str().unwrap_or("").replace('\n', "");
        let decoded = String::from_utf8(base64_decode(&content)).unwrap_or_else(|_| content.clone());
        Ok(FileContent {
            path: v["path"].as_str().unwrap_or(path).to_string(),
            content: decoded,
            sha: v["sha"].as_str().unwrap_or("").to_string(),
            size: v["size"].as_u64().unwrap_or(0),
        })
    }

    pub async fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>, String> {
        let v = self.get(&format!("/search/code?q={}&per_page=10", urlencoded(query))).await?;
        Ok(v["items"].as_array().unwrap_or(&vec![]).iter().map(|i| CodeSearchResult {
            path: i["path"].as_str().unwrap_or("").to_string(),
            repository: i["repository"]["full_name"].as_str().unwrap_or("").to_string(),
            html_url: i["html_url"].as_str().unwrap_or("").to_string(),
            fragment: i["text_matches"].as_array().and_then(|m| m.first()).and_then(|m| m["fragment"].as_str()).map(|s| s.to_string()),
        }).collect())
    }

    pub async fn list_pull_requests(&self, owner: &str, repo: &str, state: Option<&str>) -> Result<Vec<PullRequest>, String> {
        let st = state.unwrap_or("open");
        let v = self.get(&format!("/repos/{}/{}/pulls?state={}&per_page=20", owner, repo, st)).await?;
        Ok(v.as_array().unwrap_or(&vec![]).iter().map(|p| PullRequest {
            number: p["number"].as_u64().unwrap_or(0),
            title: p["title"].as_str().unwrap_or("").to_string(),
            state: p["state"].as_str().unwrap_or("").to_string(),
            html_url: p["html_url"].as_str().unwrap_or("").to_string(),
            head: p["head"]["ref"].as_str().unwrap_or("").to_string(),
            base: p["base"]["ref"].as_str().unwrap_or("").to_string(),
            user: p["user"]["login"].as_str().map(|s| s.to_string()),
            mergeable: p["mergeable"].as_bool(),
            additions: p["additions"].as_u64(),
            deletions: p["deletions"].as_u64(),
            changed_files: p["changed_files"].as_u64(),
        }).collect())
    }

    pub async fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest, String> {
        let p = self.get(&format!("/repos/{}/{}/pulls/{}", owner, repo, number)).await?;
        Ok(PullRequest {
            number: p["number"].as_u64().unwrap_or(0),
            title: p["title"].as_str().unwrap_or("").to_string(),
            state: p["state"].as_str().unwrap_or("").to_string(),
            html_url: p["html_url"].as_str().unwrap_or("").to_string(),
            head: p["head"]["ref"].as_str().unwrap_or("").to_string(),
            base: p["base"]["ref"].as_str().unwrap_or("").to_string(),
            user: p["user"]["login"].as_str().map(|s| s.to_string()),
            mergeable: p["mergeable"].as_bool(),
            additions: p["additions"].as_u64(),
            deletions: p["deletions"].as_u64(),
            changed_files: p["changed_files"].as_u64(),
        })
    }

    pub async fn get_pull_request_diff(&self, owner: &str, repo: &str, number: u64) -> Result<String, String> {
        let url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, repo, number);
        let resp = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "mcp-github/1.0")
            .header("Accept", "application/vnd.github.diff")
            .send().await.map_err(|e| e.to_string())?;
        resp.text().await.map_err(|e| e.to_string())
    }

    pub async fn create_pull_request_review(&self, owner: &str, repo: &str, number: u64, body: &str, event: &str) -> Result<Value, String> {
        self.post(&format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, number), &serde_json::json!({"body": body, "event": event})).await
    }

    pub async fn create_issue(&self, owner: &str, repo: &str, title: &str, body: Option<&str>, labels: Vec<String>) -> Result<Issue, String> {
        let mut payload = serde_json::json!({"title": title});
        if let Some(b) = body { payload["body"] = serde_json::Value::String(b.to_string()); }
        if !labels.is_empty() { payload["labels"] = serde_json::json!(labels); }
        let v = self.post(&format!("/repos/{}/{}/issues", owner, repo), &payload).await?;
        Ok(Issue {
            number: v["number"].as_u64().unwrap_or(0),
            title: v["title"].as_str().unwrap_or("").to_string(),
            state: v["state"].as_str().unwrap_or("open").to_string(),
            html_url: v["html_url"].as_str().unwrap_or("").to_string(),
            body: v["body"].as_str().map(|s| s.to_string()),
            user: v["user"]["login"].as_str().map(|s| s.to_string()),
            labels: v["labels"].as_array().unwrap_or(&vec![]).iter().filter_map(|l| l["name"].as_str().map(|s| s.to_string())).collect(),
        })
    }

    pub async fn update_issue(&self, owner: &str, repo: &str, number: u64, title: Option<&str>, body: Option<&str>, state: Option<&str>, labels: Option<Vec<String>>) -> Result<Issue, String> {
        let mut payload = serde_json::json!({});
        if let Some(t) = title { payload["title"] = serde_json::Value::String(t.to_string()); }
        if let Some(b) = body { payload["body"] = serde_json::Value::String(b.to_string()); }
        if let Some(s) = state { payload["state"] = serde_json::Value::String(s.to_string()); }
        if let Some(l) = labels { payload["labels"] = serde_json::json!(l); }
        let v = self.patch(&format!("/repos/{}/{}/issues/{}", owner, repo, number), &payload).await?;
        Ok(Issue {
            number: v["number"].as_u64().unwrap_or(0),
            title: v["title"].as_str().unwrap_or("").to_string(),
            state: v["state"].as_str().unwrap_or("").to_string(),
            html_url: v["html_url"].as_str().unwrap_or("").to_string(),
            body: v["body"].as_str().map(|s| s.to_string()),
            user: v["user"]["login"].as_str().map(|s| s.to_string()),
            labels: v["labels"].as_array().unwrap_or(&vec![]).iter().filter_map(|l| l["name"].as_str().map(|s| s.to_string())).collect(),
        })
    }

    pub async fn list_releases(&self, owner: &str, repo: &str) -> Result<Vec<Release>, String> {
        let v = self.get(&format!("/repos/{}/{}/releases?per_page=10", owner, repo)).await?;
        Ok(v.as_array().unwrap_or(&vec![]).iter().map(|r| Release {
            tag_name: r["tag_name"].as_str().unwrap_or("").to_string(),
            name: r["name"].as_str().map(|s| s.to_string()),
            html_url: r["html_url"].as_str().unwrap_or("").to_string(),
            published_at: r["published_at"].as_str().map(|s| s.to_string()),
            draft: r["draft"].as_bool().unwrap_or(false),
            prerelease: r["prerelease"].as_bool().unwrap_or(false),
        }).collect())
    }
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "+").replace('/', "%2F").replace('@', "%40")
}

fn base64_decode(s: &str) -> Vec<u8> {
    let s = s.replace(['\n', '\r', ' '], "");
    let mut out = Vec::new();
    let chars: Vec<u8> = s.bytes().collect();
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in chars.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = table.iter().position(|&c| c == b).unwrap_or(0) as u8;
        }
        out.push((buf[0] << 2) | (buf[1] >> 4));
        if chunk.len() > 2 && chunk[2] != b'=' { out.push((buf[1] << 4) | (buf[2] >> 2)); }
        if chunk.len() > 3 && chunk[3] != b'=' { out.push((buf[2] << 6) | buf[3]); }
    }
    out
}
