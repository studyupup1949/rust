//! Project management functions.
//!
//! Functions for listing accessible projects via ACE API.
//! Uses user token authentication (ace_user_xxx).
//!
//! Endpoint:
//! - GET /api/v1/auth/projects

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};

use crate::errors::AceError;
use crate::types::{Project, ProjectsResponse, CORE_VERSION};

/// Build authenticated headers for project API calls.
fn build_auth_headers(token: &str) -> Result<HeaderMap, AceError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| AceError::Other(e.to_string()))?,
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&format!("ace-sdk-rust/{}", CORE_VERSION))
            .map_err(|e| AceError::Other(e.to_string()))?,
    );
    Ok(headers)
}

/// List all projects accessible to the current user.
///
/// Returns projects across ALL organizations the user has access to.
///
/// GET /api/v1/auth/projects
pub async fn list_projects(server_url: &str, token: &str) -> Result<Vec<Project>, AceError> {
    let url = format!("{}/api/v1/auth/projects", server_url);
    let headers = build_auth_headers(token)?;
    let client = reqwest::Client::new();

    let response = client.get(&url).headers(headers).send().await?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().await.unwrap_or_default();
        return Err(AceError::from_http_response(status, &body));
    }

    let resp: ProjectsResponse = response.json().await?;
    Ok(resp.projects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_projects() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/projects")
            .match_header("authorization", "Bearer test_token")
            .with_status(200)
            .with_body(
                r#"{
                    "projects": [
                        {
                            "project_id": "prj_abc123",
                            "project_name": "My Project",
                            "org_id": "org_test",
                            "org_name": "Test Org",
                            "created_at": "2024-01-01T00:00:00Z"
                        },
                        {
                            "project_id": "prj_def456",
                            "project_name": "Another Project",
                            "org_id": "org_test",
                            "org_name": "Test Org",
                            "created_at": "2024-02-01T00:00:00Z"
                        }
                    ],
                    "count": 2
                }"#,
            )
            .create_async()
            .await;

        let projects = list_projects(&server.url(), "test_token").await.unwrap();
        mock.assert_async().await;
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].project_id, "prj_abc123");
        assert_eq!(projects[0].project_name, "My Project");
        assert_eq!(projects[0].org_id, "org_test");
        assert_eq!(projects[0].org_name, "Test Org");
    }

    #[tokio::test]
    async fn test_list_projects_empty() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/projects")
            .with_status(200)
            .with_body(r#"{"projects": [], "count": 0}"#)
            .create_async()
            .await;

        let projects = list_projects(&server.url(), "test_token").await.unwrap();
        mock.assert_async().await;
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_list_projects_auth_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/auth/projects")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create_async()
            .await;

        let result = list_projects(&server.url(), "bad_token").await;
        mock.assert_async().await;
        assert!(result.is_err());
    }
}
