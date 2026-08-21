use crate::client::GitHubClient;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchRepositoriesInput { pub query: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetRepositoryInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListBranchesInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFileContentsInput { pub owner: String, pub repo: String, pub path: String, pub branch: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchCodeInput { pub query: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPullRequestsInput { pub owner: String, pub repo: String, pub state: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPullRequestInput { pub owner: String, pub repo: String, pub number: u64 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPullRequestDiffInput { pub owner: String, pub repo: String, pub number: u64 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePullRequestReviewInput { pub owner: String, pub repo: String, pub number: u64, pub body: String, /// APPROVE, REQUEST_CHANGES, or COMMENT
    pub event: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateIssueInput { pub owner: String, pub repo: String, pub title: String, pub body: Option<String>, #[serde(default)] pub labels: Vec<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateIssueInput { pub owner: String, pub repo: String, pub number: u64, pub title: Option<String>, pub body: Option<String>, pub state: Option<String>, pub labels: Option<Vec<String>> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListReleasesInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateReleaseNoteInput { pub owner: String, pub repo: String, pub tag_name: String, pub previous_tag: Option<String> }

#[derive(Clone)]
pub struct GitHubServer {
    pub client: Arc<GitHubClient>,
}

#[tool_router(server_handler)]
impl GitHubServer {
    #[tool(description = "Search GitHub repositories by query")]
    async fn search_repositories(&self, Parameters(i): Parameters<SearchRepositoriesInput>) -> String {
        match self.client.search_repositories(&i.query).await {
            Ok(results) => serde_json::to_string_pretty(&results).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get repository details including default branch, language, and stats")]
    async fn get_repository(&self, Parameters(i): Parameters<GetRepositoryInput>) -> String {
        match self.client.get_repository(&i.owner, &i.repo).await {
            Ok(repo) => serde_json::to_string_pretty(&repo).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List branches for a repository")]
    async fn list_branches(&self, Parameters(i): Parameters<ListBranchesInput>) -> String {
        match self.client.list_branches(&i.owner, &i.repo).await {
            Ok(branches) => serde_json::to_string_pretty(&branches).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get file contents from a repository (decoded from base64)")]
    async fn get_file_contents(&self, Parameters(i): Parameters<GetFileContentsInput>) -> String {
        match self.client.get_file_contents(&i.owner, &i.repo, &i.path, i.branch.as_deref()).await {
            Ok(file) => serde_json::to_string_pretty(&file).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Search code across GitHub repositories")]
    async fn search_code(&self, Parameters(i): Parameters<SearchCodeInput>) -> String {
        match self.client.search_code(&i.query).await {
            Ok(results) => serde_json::to_string_pretty(&results).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List pull requests for a repository")]
    async fn list_pull_requests(&self, Parameters(i): Parameters<ListPullRequestsInput>) -> String {
        match self.client.list_pull_requests(&i.owner, &i.repo, i.state.as_deref()).await {
            Ok(prs) => serde_json::to_string_pretty(&prs).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get a specific pull request with merge status and stats")]
    async fn get_pull_request(&self, Parameters(i): Parameters<GetPullRequestInput>) -> String {
        match self.client.get_pull_request(&i.owner, &i.repo, i.number).await {
            Ok(pr) => serde_json::to_string_pretty(&pr).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get the diff for a pull request")]
    async fn get_pull_request_diff(&self, Parameters(i): Parameters<GetPullRequestDiffInput>) -> String {
        match self.client.get_pull_request_diff(&i.owner, &i.repo, i.number).await {
            Ok(diff) => {
                let lines: Vec<&str> = diff.lines().take(200).collect();
                if diff.lines().count() > 200 {
                    format!("{}\n\n... truncated ({} total lines)", lines.join("\n"), diff.lines().count())
                } else {
                    diff
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a review on a pull request (APPROVE, REQUEST_CHANGES, or COMMENT)")]
    async fn create_pull_request_review(&self, Parameters(i): Parameters<CreatePullRequestReviewInput>) -> String {
        match self.client.create_pull_request_review(&i.owner, &i.repo, i.number, &i.body, &i.event).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"id": v["id"], "state": v["state"], "html_url": v["html_url"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a new issue in a repository")]
    async fn create_issue(&self, Parameters(i): Parameters<CreateIssueInput>) -> String {
        match self.client.create_issue(&i.owner, &i.repo, &i.title, i.body.as_deref(), i.labels).await {
            Ok(issue) => serde_json::to_string_pretty(&issue).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Update an existing issue (title, body, state, labels)")]
    async fn update_issue(&self, Parameters(i): Parameters<UpdateIssueInput>) -> String {
        match self.client.update_issue(&i.owner, &i.repo, i.number, i.title.as_deref(), i.body.as_deref(), i.state.as_deref(), i.labels).await {
            Ok(issue) => serde_json::to_string_pretty(&issue).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List releases for a repository")]
    async fn list_releases(&self, Parameters(i): Parameters<ListReleasesInput>) -> String {
        match self.client.list_releases(&i.owner, &i.repo).await {
            Ok(releases) => serde_json::to_string_pretty(&releases).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Generate release notes between two tags")]
    async fn create_release_note(&self, Parameters(i): Parameters<CreateReleaseNoteInput>) -> String {
        let prev = i.previous_tag.as_deref().unwrap_or("");
        let compare = if prev.is_empty() { i.tag_name.clone() } else { format!("{}...{}", prev, i.tag_name) };
        // Use compare API to get commits between tags
        match self.client.search_repositories(&format!("repo:{}/{}", i.owner, i.repo)).await {
            Ok(_) => serde_json::to_string_pretty(&serde_json::json!({
                "tag": i.tag_name,
                "previous_tag": prev,
                "compare_url": format!("https://github.com/{}/{}/compare/{}", i.owner, i.repo, compare),
                "message": "Use compare_url to view changes between tags."
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }
}
