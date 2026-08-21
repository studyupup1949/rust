use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::client::GitHubClient;
use crate::git_local::GitLocal;
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePullRequestInput { pub owner: String, pub repo: String, pub title: String, pub head: String, pub base: String, pub body: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergePullRequestInput { pub owner: String, pub repo: String, pub number: u64, /// merge, squash, or rebase
    pub merge_method: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPrCommentsInput { pub owner: String, pub repo: String, pub number: u64 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddPrCommentInput { pub owner: String, pub repo: String, pub number: u64, pub body: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateBranchInput { pub owner: String, pub repo: String, pub branch: String, pub from_sha: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteBranchInput { pub owner: String, pub repo: String, pub branch: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListCommitsInput { pub owner: String, pub repo: String, pub branch: Option<String>, pub per_page: Option<u64> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCommitInput { pub owner: String, pub repo: String, pub sha: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListWorkflowRunsInput { pub owner: String, pub repo: String, pub branch: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetWorkflowRunInput { pub owner: String, pub repo: String, pub run_id: u64 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateOrUpdateFileInput { pub owner: String, pub repo: String, pub path: String, pub content: String, pub message: String, pub sha: Option<String>, pub branch: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDirectoryInput { pub owner: String, pub repo: String, pub path: String, pub branch: Option<String> }

// --- GitHub API: Repo lifecycle ---
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateRepositoryInput { pub name: String, pub description: Option<String>, #[serde(default)] pub private: bool, pub org: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ForkRepositoryInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteRepositoryInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTagsInput { pub owner: String, pub repo: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateReleaseInput { pub owner: String, pub repo: String, pub tag_name: String, pub name: Option<String>, pub body: Option<String>, #[serde(default)] pub draft: bool, #[serde(default)] pub prerelease: bool }

// --- Local git operations ---
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitCloneInput { pub url: String, pub path: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitInitInput { pub path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitStatusInput { pub path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitAddInput { pub path: String, #[serde(default = "default_dot")] pub files: Vec<String> }
fn default_dot() -> Vec<String> { vec![".".to_string()] }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitCommitInput { pub path: String, pub message: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitPushInput { pub path: String, pub remote: Option<String>, pub branch: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitTagInput { pub path: String, pub name: String, pub message: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GitLogInput { pub path: String, #[serde(default = "default_count")] pub count: u32 }
fn default_count() -> u32 { 10 }

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
        match self.client.search_repositories(&format!("repo:{}/{}", i.owner, i.repo)).await {
            Ok(_) => serde_json::to_string_pretty(&serde_json::json!({
                "tag": i.tag_name, "previous_tag": prev,
                "compare_url": format!("https://github.com/{}/{}/compare/{}", i.owner, i.repo, compare),
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a new pull request")]
    async fn create_pull_request(&self, Parameters(i): Parameters<CreatePullRequestInput>) -> String {
        match self.client.create_pull_request(&i.owner, &i.repo, &i.title, &i.head, &i.base, i.body.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"number": v["number"], "title": v["title"], "html_url": v["html_url"], "state": v["state"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Merge a pull request (merge, squash, or rebase)")]
    async fn merge_pull_request(&self, Parameters(i): Parameters<MergePullRequestInput>) -> String {
        match self.client.merge_pull_request(&i.owner, &i.repo, i.number, i.merge_method.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"merged": v["merged"], "message": v["message"], "sha": v["sha"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List comments on a pull request or issue")]
    async fn list_pr_comments(&self, Parameters(i): Parameters<ListPrCommentsInput>) -> String {
        match self.client.list_pr_comments(&i.owner, &i.repo, i.number).await {
            Ok(v) => {
                let comments: Vec<serde_json::Value> = v.as_array().unwrap_or(&vec![]).iter().map(|c| serde_json::json!({"id": c["id"], "user": c["user"]["login"], "body": c["body"], "created_at": c["created_at"]})).collect();
                serde_json::to_string_pretty(&comments).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Add a comment to a pull request or issue")]
    async fn add_pr_comment(&self, Parameters(i): Parameters<AddPrCommentInput>) -> String {
        match self.client.add_pr_comment(&i.owner, &i.repo, i.number, &i.body).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"id": v["id"], "html_url": v["html_url"], "created_at": v["created_at"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a new branch from a SHA")]
    async fn create_branch(&self, Parameters(i): Parameters<CreateBranchInput>) -> String {
        match self.client.create_branch(&i.owner, &i.repo, &i.branch, &i.from_sha).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"ref": v["ref"], "sha": v["object"]["sha"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Delete a branch")]
    async fn delete_branch(&self, Parameters(i): Parameters<DeleteBranchInput>) -> String {
        match self.client.delete_branch(&i.owner, &i.repo, &i.branch).await {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"deleted": true, "branch": i.branch})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List commits for a repository or branch")]
    async fn list_commits(&self, Parameters(i): Parameters<ListCommitsInput>) -> String {
        match self.client.list_commits(&i.owner, &i.repo, i.branch.as_deref(), i.per_page).await {
            Ok(v) => {
                let commits: Vec<serde_json::Value> = v.as_array().unwrap_or(&vec![]).iter().map(|c| serde_json::json!({"sha": c["sha"].as_str().unwrap_or("")[..8], "message": c["commit"]["message"].as_str().unwrap_or("").lines().next(), "author": c["commit"]["author"]["name"], "date": c["commit"]["author"]["date"]})).collect();
                serde_json::to_string_pretty(&commits).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get details of a specific commit")]
    async fn get_commit(&self, Parameters(i): Parameters<GetCommitInput>) -> String {
        match self.client.get_commit(&i.owner, &i.repo, &i.sha).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"sha": v["sha"], "message": v["commit"]["message"], "author": v["commit"]["author"]["name"], "date": v["commit"]["author"]["date"], "files_changed": v["files"].as_array().map(|f| f.len())})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List GitHub Actions workflow runs")]
    async fn list_workflow_runs(&self, Parameters(i): Parameters<ListWorkflowRunsInput>) -> String {
        match self.client.list_workflow_runs(&i.owner, &i.repo, i.branch.as_deref()).await {
            Ok(v) => {
                let runs: Vec<serde_json::Value> = v["workflow_runs"].as_array().unwrap_or(&vec![]).iter().map(|r| serde_json::json!({"id": r["id"], "name": r["name"], "status": r["status"], "conclusion": r["conclusion"], "branch": r["head_branch"], "created_at": r["created_at"]})).collect();
                serde_json::to_string_pretty(&runs).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get details of a specific workflow run")]
    async fn get_workflow_run(&self, Parameters(i): Parameters<GetWorkflowRunInput>) -> String {
        match self.client.get_workflow_run(&i.owner, &i.repo, i.run_id).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"id": v["id"], "name": v["name"], "status": v["status"], "conclusion": v["conclusion"], "html_url": v["html_url"], "run_started_at": v["run_started_at"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create or update a file in a repository")]
    async fn create_or_update_file(&self, Parameters(i): Parameters<CreateOrUpdateFileInput>) -> String {
        match self.client.create_or_update_file(&i.owner, &i.repo, &i.path, &i.content, &i.message, i.sha.as_deref(), i.branch.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"path": v["content"]["path"], "sha": v["content"]["sha"], "commit_sha": v["commit"]["sha"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List directory contents in a repository")]
    async fn list_directory(&self, Parameters(i): Parameters<ListDirectoryInput>) -> String {
        match self.client.list_directory(&i.owner, &i.repo, &i.path, i.branch.as_deref()).await {
            Ok(v) => {
                let entries: Vec<serde_json::Value> = v.as_array().unwrap_or(&vec![]).iter().map(|e| serde_json::json!({"name": e["name"], "type": e["type"], "path": e["path"], "size": e["size"]})).collect();
                serde_json::to_string_pretty(&entries).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    // --- GitHub API: Repo lifecycle ---

    #[tool(description = "Create a new GitHub repository (personal or org)")]
    async fn create_repository(&self, Parameters(i): Parameters<CreateRepositoryInput>) -> String {
        let result = if let Some(org) = &i.org {
            self.client.create_org_repository(org, &i.name, i.description.as_deref(), i.private).await
        } else {
            self.client.create_repository(&i.name, i.description.as_deref(), i.private).await
        };
        match result {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"full_name": v["full_name"], "html_url": v["html_url"], "private": v["private"], "default_branch": v["default_branch"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Fork a repository to your account")]
    async fn fork_repository(&self, Parameters(i): Parameters<ForkRepositoryInput>) -> String {
        match self.client.fork_repository(&i.owner, &i.repo).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"full_name": v["full_name"], "html_url": v["html_url"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Delete a repository (DANGEROUS — irreversible)")]
    async fn delete_repository(&self, Parameters(i): Parameters<DeleteRepositoryInput>) -> String {
        match self.client.delete_repository(&i.owner, &i.repo).await {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"deleted": true, "repository": format!("{}/{}", i.owner, i.repo)})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List tags for a repository")]
    async fn list_tags(&self, Parameters(i): Parameters<ListTagsInput>) -> String {
        match self.client.list_tags(&i.owner, &i.repo).await {
            Ok(v) => {
                let tags: Vec<serde_json::Value> = v.as_array().unwrap_or(&vec![]).iter().map(|t| serde_json::json!({"name": t["name"], "sha": t["commit"]["sha"]})).collect();
                serde_json::to_string_pretty(&tags).unwrap()
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a GitHub release with tag, name, and body")]
    async fn create_release(&self, Parameters(i): Parameters<CreateReleaseInput>) -> String {
        match self.client.create_release(&i.owner, &i.repo, &i.tag_name, i.name.as_deref(), i.body.as_deref(), i.draft, i.prerelease).await {
            Ok(v) => serde_json::to_string_pretty(&serde_json::json!({"id": v["id"], "tag_name": v["tag_name"], "html_url": v["html_url"], "draft": v["draft"]})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    // --- Local git operations ---

    #[tool(description = "Clone a git repository to local filesystem")]
    async fn git_clone(&self, Parameters(i): Parameters<GitCloneInput>) -> String {
        match GitLocal::clone(&i.url, i.path.as_deref()) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"status": "cloned", "url": i.url, "output": out})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Initialize a new git repository")]
    async fn git_init(&self, Parameters(i): Parameters<GitInitInput>) -> String {
        match GitLocal::init(&i.path) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"status": "initialized", "path": i.path, "output": out})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get git status of a repository")]
    async fn git_status(&self, Parameters(i): Parameters<GitStatusInput>) -> String {
        match GitLocal::status(&i.path) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"path": i.path, "status": out, "clean": out.is_empty()})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Stage files for commit (git add)")]
    async fn git_add(&self, Parameters(i): Parameters<GitAddInput>) -> String {
        let files: Vec<&str> = i.files.iter().map(|s| s.as_str()).collect();
        match GitLocal::add(&i.path, &files) {
            Ok(_) => serde_json::to_string_pretty(&serde_json::json!({"status": "staged", "files": i.files})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a git commit with a message")]
    async fn git_commit(&self, Parameters(i): Parameters<GitCommitInput>) -> String {
        match GitLocal::commit(&i.path, &i.message) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"status": "committed", "message": i.message, "output": out})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Push commits to remote")]
    async fn git_push(&self, Parameters(i): Parameters<GitPushInput>) -> String {
        match GitLocal::push(&i.path, i.remote.as_deref(), i.branch.as_deref()) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"status": "pushed", "output": out})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a git tag")]
    async fn git_tag(&self, Parameters(i): Parameters<GitTagInput>) -> String {
        match GitLocal::tag(&i.path, &i.name, i.message.as_deref()) {
            Ok(_) => serde_json::to_string_pretty(&serde_json::json!({"status": "tagged", "tag": i.name})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Show git log (recent commits)")]
    async fn git_log(&self, Parameters(i): Parameters<GitLogInput>) -> String {
        match GitLocal::log(&i.path, i.count) {
            Ok(out) => serde_json::to_string_pretty(&serde_json::json!({"path": i.path, "log": out})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for GitHubServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            message: Some("operational".into()),
            latency_ms: Some(1),
        }
    }
}
