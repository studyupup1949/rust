use std::{fmt::Display, fs::File, io::Read, path::Path, time::SystemTime};

use futures::{StreamExt, stream::BoxStream};
use thiserror::Error;
use tokio::sync::mpsc;

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use url::Url;
use uuid::Uuid;

mod rest_types;
mod serde_utils;

use rest_types::{AbortChunkedUploadRequest, InitChunkedUploadRequest, InitChunkedUploadResponse};

const MEGABYTE: u64 = 1024 * 1024; // 1MB
pub const MIN_CHUNK_SIZE_BYTES: u64 = 5 * MEGABYTE;
const MAX_CHUNK_SIZE_BYTES: u64 = 100 * MEGABYTE;
const MAX_PARTS_COUNT: u64 = 10000;

const SIZE_500MB: u64 = 500 * MEGABYTE;
const SIZE_10GB: u64 = 10 * 1024 * MEGABYTE;
const SIZE_50GB: u64 = 50 * 1024 * MEGABYTE;

#[derive(Error, Debug)]
pub enum AdaptiveError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("File too small for chunked upload: {size} bytes (minimum: {min_size} bytes)")]
    FileTooSmall { size: u64, min_size: u64 },

    #[error("File too large: {size} bytes exceeds maximum {max_size} bytes")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("GraphQL errors: {0:?}")]
    GraphQLErrors(Vec<graphql_client::Error>),

    #[error("No data returned from GraphQL")]
    NoGraphQLData,

    #[error("Job not found: {0}")]
    JobNotFound(Uuid),

    #[error("Failed to initialize chunked upload: {status} - {body}")]
    ChunkedUploadInitFailed { status: String, body: String },

    #[error("Failed to upload part {part_number}: {status} - {body}")]
    ChunkedUploadPartFailed {
        part_number: u64,
        status: String,
        body: String,
    },

    #[error("Failed to create dataset: {0}")]
    DatasetCreationFailed(String),

    #[error("HTTP status error: {status} - {body}")]
    HttpStatusError { status: String, body: String },

    #[error("Failed to parse JSON response: {error}. Body preview: {body}")]
    JsonParseError { error: String, body: String },
}

type Result<T> = std::result::Result<T, AdaptiveError>;

#[derive(Clone, Debug, Default)]
pub struct ChunkedUploadProgress {
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
}

#[derive(Debug)]
pub enum UploadEvent {
    Progress(ChunkedUploadProgress),
    Complete(
        create_dataset_from_multipart::CreateDatasetFromMultipartCreateDatasetFromMultipartUpload,
    ),
}

pub fn calculate_upload_parts(file_size: u64) -> Result<(u64, u64)> {
    if file_size < MIN_CHUNK_SIZE_BYTES {
        return Err(AdaptiveError::FileTooSmall {
            size: file_size,
            min_size: MIN_CHUNK_SIZE_BYTES,
        });
    }

    let mut chunk_size = if file_size < SIZE_500MB {
        5 * MEGABYTE
    } else if file_size < SIZE_10GB {
        10 * MEGABYTE
    } else if file_size < SIZE_50GB {
        50 * MEGABYTE
    } else {
        100 * MEGABYTE
    };

    let mut total_parts = file_size.div_ceil(chunk_size);

    if total_parts > MAX_PARTS_COUNT {
        chunk_size = file_size.div_ceil(MAX_PARTS_COUNT);

        if chunk_size > MAX_CHUNK_SIZE_BYTES {
            let max_file_size = MAX_CHUNK_SIZE_BYTES * MAX_PARTS_COUNT;
            return Err(AdaptiveError::FileTooLarge {
                size: file_size,
                max_size: max_file_size,
            });
        }

        total_parts = file_size.div_ceil(chunk_size);
    }

    Ok((total_parts, chunk_size))
}

type IdOrKey = String;
#[allow(clippy::upper_case_acronyms)]
type UUID = Uuid;
type JsObject = Map<String, Value>;
type InputDatetime = String;
#[allow(clippy::upper_case_acronyms)]
type JSON = Value;
type KeyInput = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub SystemTime);

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let system_time = serde_utils::deserialize_timestamp_millis(deserializer)?;
        Ok(Timestamp(system_time))
    }
}

const PAGE_SIZE: usize = 20;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Upload(usize);

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/list.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetCustomRecipes;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/job.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetJob;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/jobs.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListJobs;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/cancel.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CancelJob;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/models.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListModels;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/all_models.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListAllModels;

impl Display for get_job::JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            get_job::JobStatus::PENDING => write!(f, "Pending"),
            get_job::JobStatus::RUNNING => write!(f, "Running"),
            get_job::JobStatus::COMPLETED => write!(f, "Completed"),
            get_job::JobStatus::FAILED => write!(f, "Failed"),
            get_job::JobStatus::CANCELED => write!(f, "Canceled"),
            get_job::JobStatus::Other(_) => write!(f, "Unknown"),
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/publish.graphql",
    response_derives = "Debug, Clone"
)]
pub struct PublishCustomRecipe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/update_recipe.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UpdateCustomRecipe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/upload_dataset.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UploadDataset;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/create_dataset_from_multipart.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateDatasetFromMultipart;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/run.graphql",
    response_derives = "Debug, Clone"
)]
pub struct RunCustomRecipe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/projects.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListProjects;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/pools.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListComputePools;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/recipe.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetRecipe;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/grader.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct GetGrader;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/dataset.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetDataset;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/model_config.graphql",
    response_derives = "Debug, Clone, Serialize"
)]
pub struct GetModelConfig;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/job_progress.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UpdateJobProgress;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/roles.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListRoles;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/create_role.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateRole;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/teams.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListTeams;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/create_team.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateTeam;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/users.graphql",
    response_derives = "Debug, Clone"
)]
pub struct ListUsers;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/create_user.graphql",
    response_derives = "Debug, Clone"
)]
pub struct CreateUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/delete_user.graphql",
    response_derives = "Debug, Clone"
)]
pub struct DeleteUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/add_team_member.graphql",
    response_derives = "Debug, Clone"
)]
pub struct AddTeamMember;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.gql",
    query_path = "src/graphql/remove_team_member.graphql",
    response_derives = "Debug, Clone"
)]
pub struct RemoveTeamMember;

const INIT_CHUNKED_UPLOAD_ROUTE: &str = "v1/upload/init";
const UPLOAD_PART_ROUTE: &str = "v1/upload/part";
const ABORT_CHUNKED_UPLOAD_ROUTE: &str = "v1/upload/abort";

#[derive(Clone)]
pub struct AdaptiveClient {
    client: Client,
    graphql_url: Url,
    rest_base_url: Url,
    auth_token: String,
}

impl AdaptiveClient {
    pub fn new(api_base_url: Url, auth_token: String) -> Self {
        let graphql_url = api_base_url
            .join("graphql")
            .expect("Failed to append graphql to base URL");

        let client = Client::builder()
            .user_agent(format!(
                "adaptive-client-rust/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            graphql_url,
            rest_base_url: api_base_url,
            auth_token,
        }
    }

    async fn execute_query<T>(&self, _query: T, variables: T::Variables) -> Result<T::ResponseData>
    where
        T: GraphQLQuery,
        T::Variables: serde::Serialize,
        T::ResponseData: DeserializeOwned,
    {
        let request_body = T::build_query(variables);

        let response = self
            .client
            .post(self.graphql_url.clone())
            .bearer_auth(&self.auth_token)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(AdaptiveError::HttpStatusError {
                status: status.to_string(),
                body: response_text,
            });
        }

        let response_body: Response<T::ResponseData> = serde_json::from_str(&response_text)
            .map_err(|e| AdaptiveError::JsonParseError {
                error: e.to_string(),
                body: response_text.chars().take(500).collect(),
            })?;

        match response_body.data {
            Some(data) => Ok(data),
            None => {
                if let Some(errors) = response_body.errors {
                    return Err(AdaptiveError::GraphQLErrors(errors));
                }
                Err(AdaptiveError::NoGraphQLData)
            }
        }
    }

    pub async fn list_recipes(
        &self,
        project: &str,
    ) -> Result<Vec<get_custom_recipes::GetCustomRecipesCustomRecipes>> {
        let variables = get_custom_recipes::Variables {
            project: IdOrKey::from(project),
        };

        let response_data = self.execute_query(GetCustomRecipes, variables).await?;
        Ok(response_data.custom_recipes)
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<get_job::GetJobJob> {
        let variables = get_job::Variables { id: job_id };

        let response_data = self.execute_query(GetJob, variables).await?;

        match response_data.job {
            Some(job) => Ok(job),
            None => Err(AdaptiveError::JobNotFound(job_id)),
        }
    }

    pub async fn upload_dataset<P: AsRef<Path>>(
        &self,
        project: &str,
        name: &str,
        dataset: P,
    ) -> Result<upload_dataset::UploadDatasetCreateDataset> {
        let variables = upload_dataset::Variables {
            project: IdOrKey::from(project),
            file: Upload(0),
            name: Some(name.to_string()),
        };

        let operations = UploadDataset::build_query(variables);
        let operations = serde_json::to_string(&operations)?;

        let file_map = r#"{ "0": ["variables.file"] }"#;

        let dataset_file = reqwest::multipart::Part::file(dataset).await?;

        let form = reqwest::multipart::Form::new()
            .text("operations", operations)
            .text("map", file_map)
            .part("0", dataset_file);

        let response = self
            .client
            .post(self.graphql_url.clone())
            .bearer_auth(&self.auth_token)
            .multipart(form)
            .send()
            .await?;

        let response: Response<<UploadDataset as graphql_client::GraphQLQuery>::ResponseData> =
            response.json().await?;

        match response.data {
            Some(data) => Ok(data.create_dataset),
            None => {
                if let Some(errors) = response.errors {
                    return Err(AdaptiveError::GraphQLErrors(errors));
                }
                Err(AdaptiveError::NoGraphQLData)
            }
        }
    }

    pub async fn publish_recipe<P: AsRef<Path>>(
        &self,
        project: &str,
        name: &str,
        key: &str,
        recipe: P,
    ) -> Result<publish_custom_recipe::PublishCustomRecipeCreateCustomRecipe> {
        let variables = publish_custom_recipe::Variables {
            project: IdOrKey::from(project),
            file: Upload(0),
            name: Some(name.to_string()),
            key: Some(key.to_string()),
        };

        let operations = PublishCustomRecipe::build_query(variables);
        let operations = serde_json::to_string(&operations)?;

        let file_map = r#"{ "0": ["variables.file"] }"#;

        let recipe_file = reqwest::multipart::Part::file(recipe).await?;

        let form = reqwest::multipart::Form::new()
            .text("operations", operations)
            .text("map", file_map)
            .part("0", recipe_file);

        let response = self
            .client
            .post(self.graphql_url.clone())
            .bearer_auth(&self.auth_token)
            .multipart(form)
            .send()
            .await?;
        let response: Response<
            <PublishCustomRecipe as graphql_client::GraphQLQuery>::ResponseData,
        > = response.json().await?;

        match response.data {
            Some(data) => Ok(data.create_custom_recipe),
            None => {
                if let Some(errors) = response.errors {
                    return Err(AdaptiveError::GraphQLErrors(errors));
                }
                Err(AdaptiveError::NoGraphQLData)
            }
        }
    }

    pub async fn update_recipe<P: AsRef<Path>>(
        &self,
        project: &str,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        labels: Option<Vec<update_custom_recipe::LabelInput>>,
        recipe_file: Option<P>,
    ) -> Result<update_custom_recipe::UpdateCustomRecipeUpdateCustomRecipe> {
        let input = update_custom_recipe::UpdateRecipeInput {
            name,
            description,
            labels,
        };

        match recipe_file {
            Some(file_path) => {
                let variables = update_custom_recipe::Variables {
                    project: IdOrKey::from(project),
                    id: IdOrKey::from(id),
                    input,
                    file: Some(Upload(0)),
                };

                let operations = UpdateCustomRecipe::build_query(variables);
                let operations = serde_json::to_string(&operations)?;

                let file_map = r#"{ "0": ["variables.file"] }"#;

                let recipe_file = reqwest::multipart::Part::file(file_path).await?;

                let form = reqwest::multipart::Form::new()
                    .text("operations", operations)
                    .text("map", file_map)
                    .part("0", recipe_file);

                let response = self
                    .client
                    .post(self.graphql_url.clone())
                    .bearer_auth(&self.auth_token)
                    .multipart(form)
                    .send()
                    .await?;
                let response: Response<
                    <UpdateCustomRecipe as graphql_client::GraphQLQuery>::ResponseData,
                > = response.json().await?;

                match response.data {
                    Some(data) => Ok(data.update_custom_recipe),
                    None => {
                        if let Some(errors) = response.errors {
                            return Err(AdaptiveError::GraphQLErrors(errors));
                        }
                        Err(AdaptiveError::NoGraphQLData)
                    }
                }
            }
            None => {
                let variables = update_custom_recipe::Variables {
                    project: IdOrKey::from(project),
                    id: IdOrKey::from(id),
                    input,
                    file: None,
                };

                let response_data = self.execute_query(UpdateCustomRecipe, variables).await?;
                Ok(response_data.update_custom_recipe)
            }
        }
    }

    pub async fn run_recipe(
        &self,
        project: &str,
        recipe_id: &str,
        parameters: Map<String, Value>,
        name: Option<String>,
        compute_pool: Option<String>,
        num_gpus: u32,
        use_experimental_runner: bool,
    ) -> Result<run_custom_recipe::RunCustomRecipeCreateJob> {
        let variables = run_custom_recipe::Variables {
            input: run_custom_recipe::JobInput {
                recipe: recipe_id.to_string(),
                project: project.to_string(),
                args: parameters,
                name,
                compute_pool,
                num_gpus: num_gpus as i64,
                use_experimental_runner,
                max_cpu: None,
                max_ram_gb: None,
                max_duration_secs: None,
            },
        };

        let response_data = self.execute_query(RunCustomRecipe, variables).await?;
        Ok(response_data.create_job)
    }

    pub async fn list_jobs(
        &self,
        project: Option<String>,
    ) -> Result<Vec<list_jobs::ListJobsJobsNodes>> {
        let mut jobs = Vec::new();
        let mut page = self.list_jobs_page(project.clone(), None).await?;
        jobs.extend(page.nodes.iter().cloned());
        while page.page_info.has_next_page {
            page = self
                .list_jobs_page(project.clone(), page.page_info.end_cursor)
                .await?;
            jobs.extend(page.nodes.iter().cloned());
        }
        Ok(jobs)
    }

    async fn list_jobs_page(
        &self,
        project: Option<String>,
        after: Option<String>,
    ) -> Result<list_jobs::ListJobsJobs> {
        let variables = list_jobs::Variables {
            filter: Some(list_jobs::ListJobsFilterInput {
                project,
                kind: Some(vec![list_jobs::JobKind::CUSTOM]),
                status: Some(vec![
                    list_jobs::JobStatus::RUNNING,
                    list_jobs::JobStatus::PENDING,
                ]),
                timerange: None,
                custom_recipes: None,
                artifacts: None,
            }),
            cursor: Some(list_jobs::CursorPageInput {
                first: Some(PAGE_SIZE as i64),
                after,
                before: None,
                last: None,
                offset: None,
            }),
        };

        let response_data = self.execute_query(ListJobs, variables).await?;
        Ok(response_data.jobs)
    }

    pub async fn cancel_job(&self, job_id: Uuid) -> Result<cancel_job::CancelJobCancelJob> {
        let variables = cancel_job::Variables { job_id };

        let response_data = self.execute_query(CancelJob, variables).await?;
        Ok(response_data.cancel_job)
    }

    pub async fn update_job_progress(
        &self,
        job_id: Uuid,
        event: update_job_progress::JobProgressEventInput,
    ) -> Result<update_job_progress::UpdateJobProgressUpdateJobProgress> {
        let variables = update_job_progress::Variables { job_id, event };

        let response_data = self.execute_query(UpdateJobProgress, variables).await?;
        Ok(response_data.update_job_progress)
    }

    pub async fn list_models(
        &self,
        project: String,
    ) -> Result<Vec<list_models::ListModelsProjectModelServices>> {
        let variables = list_models::Variables { project };

        let response_data = self.execute_query(ListModels, variables).await?;
        Ok(response_data
            .project
            .map(|project| project.model_services)
            .unwrap_or(Vec::new()))
    }

    pub async fn list_all_models(&self) -> Result<Vec<list_all_models::ListAllModelsModels>> {
        let variables = list_all_models::Variables {};

        let response_data = self.execute_query(ListAllModels, variables).await?;
        Ok(response_data.models)
    }

    pub async fn list_projects(&self) -> Result<Vec<list_projects::ListProjectsProjects>> {
        let variables = list_projects::Variables {};

        let response_data = self.execute_query(ListProjects, variables).await?;
        Ok(response_data.projects)
    }

    pub async fn list_pools(
        &self,
    ) -> Result<Vec<list_compute_pools::ListComputePoolsComputePools>> {
        let variables = list_compute_pools::Variables {};

        let response_data = self.execute_query(ListComputePools, variables).await?;
        Ok(response_data.compute_pools)
    }

    pub async fn list_roles(&self) -> Result<Vec<list_roles::ListRolesRoles>> {
        let variables = list_roles::Variables {};

        let response_data = self.execute_query(ListRoles, variables).await?;
        Ok(response_data.roles)
    }

    pub async fn create_role(
        &self,
        name: &str,
        key: Option<&str>,
        permissions: Vec<String>,
    ) -> Result<create_role::CreateRoleCreateRole> {
        let variables = create_role::Variables {
            input: create_role::RoleCreate {
                name: name.to_string(),
                key: key.map(|k| k.to_string()),
                permissions,
            },
        };

        let response_data = self.execute_query(CreateRole, variables).await?;
        Ok(response_data.create_role)
    }

    pub async fn list_teams(&self) -> Result<Vec<list_teams::ListTeamsTeams>> {
        let variables = list_teams::Variables {};

        let response_data = self.execute_query(ListTeams, variables).await?;
        Ok(response_data.teams)
    }

    pub async fn create_team(
        &self,
        name: &str,
        key: Option<&str>,
    ) -> Result<create_team::CreateTeamCreateTeam> {
        let variables = create_team::Variables {
            input: create_team::TeamCreate {
                name: name.to_string(),
                key: key.map(|k| k.to_string()),
            },
        };

        let response_data = self.execute_query(CreateTeam, variables).await?;
        Ok(response_data.create_team)
    }

    pub async fn list_users(&self) -> Result<Vec<list_users::ListUsersUsers>> {
        let variables = list_users::Variables {};

        let response_data = self.execute_query(ListUsers, variables).await?;
        Ok(response_data.users)
    }

    pub async fn create_user(
        &self,
        name: &str,
        email: Option<&str>,
        teams: Vec<create_user::UserCreateTeamWithRole>,
        user_type: Option<create_user::UserType>,
        generate_api_key: Option<bool>,
    ) -> Result<create_user::CreateUserCreateUser> {
        let variables = create_user::Variables {
            input: create_user::UserCreate {
                name: name.to_string(),
                email: email.map(|e| e.to_string()),
                teams,
                user_type: user_type.unwrap_or(create_user::UserType::HUMAN),
                generate_api_key,
            },
        };

        let response_data = self.execute_query(CreateUser, variables).await?;
        Ok(response_data.create_user)
    }

    pub async fn delete_user(&self, user: &str) -> Result<delete_user::DeleteUserDeleteUser> {
        let variables = delete_user::Variables {
            user: user.to_string(),
        };

        let response_data = self.execute_query(DeleteUser, variables).await?;
        Ok(response_data.delete_user)
    }

    pub async fn add_team_member(
        &self,
        user: &str,
        team: &str,
        role: &str,
    ) -> Result<add_team_member::AddTeamMemberSetTeamMember> {
        let variables = add_team_member::Variables {
            input: add_team_member::TeamMemberSet {
                user: user.to_string(),
                team: team.to_string(),
                role: role.to_string(),
            },
        };

        let response_data = self.execute_query(AddTeamMember, variables).await?;
        Ok(response_data.set_team_member)
    }

    pub async fn remove_team_member(
        &self,
        user: &str,
        team: &str,
    ) -> Result<remove_team_member::RemoveTeamMemberRemoveTeamMember> {
        let variables = remove_team_member::Variables {
            input: remove_team_member::TeamMemberRemove {
                user: user.to_string(),
                team: team.to_string(),
            },
        };

        let response_data = self.execute_query(RemoveTeamMember, variables).await?;
        Ok(response_data.remove_team_member)
    }

    pub async fn get_recipe(
        &self,
        project: String,
        id_or_key: String,
    ) -> Result<Option<get_recipe::GetRecipeCustomRecipe>> {
        let variables = get_recipe::Variables { project, id_or_key };

        let response_data = self.execute_query(GetRecipe, variables).await?;
        Ok(response_data.custom_recipe)
    }

    pub async fn get_grader(
        &self,
        id_or_key: &str,
        project: &str,
    ) -> Result<get_grader::GetGraderGrader> {
        let variables = get_grader::Variables {
            id: id_or_key.to_string(),
            project: project.to_string(),
        };

        let response_data = self.execute_query(GetGrader, variables).await?;
        Ok(response_data.grader)
    }

    pub async fn get_dataset(
        &self,
        id_or_key: &str,
        project: &str,
    ) -> Result<Option<get_dataset::GetDatasetDataset>> {
        let variables = get_dataset::Variables {
            id_or_key: id_or_key.to_string(),
            project: project.to_string(),
        };

        let response_data = self.execute_query(GetDataset, variables).await?;
        Ok(response_data.dataset)
    }

    pub async fn get_model_config(
        &self,
        id_or_key: &str,
    ) -> Result<Option<get_model_config::GetModelConfigModel>> {
        let variables = get_model_config::Variables {
            id_or_key: id_or_key.to_string(),
        };

        let response_data = self.execute_query(GetModelConfig, variables).await?;
        Ok(response_data.model)
    }

    pub fn base_url(&self) -> &Url {
        &self.rest_base_url
    }

    /// Upload bytes using the chunked upload API and return the session_id.
    /// This can be used to link the uploaded file to an artifact.
    pub async fn upload_bytes(&self, data: &[u8], content_type: &str) -> Result<String> {
        let file_size = data.len() as u64;

        // Calculate chunk size (same logic as calculate_upload_parts but inline for small files)
        let chunk_size = if file_size < 5 * 1024 * 1024 {
            // For files < 5MB, use the whole file as one chunk
            file_size.max(1)
        } else if file_size < 500 * 1024 * 1024 {
            5 * 1024 * 1024
        } else if file_size < 10 * 1024 * 1024 * 1024 {
            10 * 1024 * 1024
        } else {
            100 * 1024 * 1024
        };

        let total_parts = file_size.div_ceil(chunk_size).max(1);

        // Initialize upload session
        let session_id = self
            .init_chunked_upload_with_content_type(total_parts, content_type)
            .await?;

        // Upload parts
        for part_number in 1..=total_parts {
            let start = ((part_number - 1) * chunk_size) as usize;
            let end = (part_number * chunk_size).min(file_size) as usize;
            let chunk = data[start..end].to_vec();

            if let Err(e) = self
                .upload_part_simple(&session_id, part_number, chunk)
                .await
            {
                let _ = self.abort_chunked_upload(&session_id).await;
                return Err(e);
            }
        }

        Ok(session_id)
    }

    /// Initialize a chunked upload session.
    pub async fn init_chunked_upload_with_content_type(
        &self,
        total_parts: u64,
        content_type: &str,
    ) -> Result<String> {
        let url = self.rest_base_url.join(INIT_CHUNKED_UPLOAD_ROUTE)?;

        let request = InitChunkedUploadRequest {
            content_type: content_type.to_string(),
            metadata: None,
            total_parts_count: total_parts,
        };

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AdaptiveError::ChunkedUploadInitFailed {
                status: response.status().to_string(),
                body: response.text().await.unwrap_or_default(),
            });
        }

        let init_response: InitChunkedUploadResponse = response.json().await?;
        Ok(init_response.session_id)
    }

    /// Upload a single part of a chunked upload.
    pub async fn upload_part_simple(
        &self,
        session_id: &str,
        part_number: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        let url = self.rest_base_url.join(UPLOAD_PART_ROUTE)?;

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_token)
            .query(&[
                ("session_id", session_id),
                ("part_number", &part_number.to_string()),
            ])
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AdaptiveError::ChunkedUploadPartFailed {
                part_number,
                status: response.status().to_string(),
                body: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    async fn init_chunked_upload(&self, total_parts: u64) -> Result<String> {
        let url = self.rest_base_url.join(INIT_CHUNKED_UPLOAD_ROUTE)?;

        let request = InitChunkedUploadRequest {
            content_type: "application/jsonl".to_string(),
            metadata: None,
            total_parts_count: total_parts,
        };

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AdaptiveError::ChunkedUploadInitFailed {
                status: response.status().to_string(),
                body: response.text().await.unwrap_or_default(),
            });
        }

        let init_response: InitChunkedUploadResponse = response.json().await?;
        Ok(init_response.session_id)
    }

    async fn upload_part(
        &self,
        session_id: &str,
        part_number: u64,
        data: Vec<u8>,
        progress_tx: mpsc::Sender<u64>,
    ) -> Result<()> {
        const SUB_CHUNK_SIZE: usize = 64 * 1024;

        let url = self.rest_base_url.join(UPLOAD_PART_ROUTE)?;

        let chunks: Vec<Vec<u8>> = data
            .chunks(SUB_CHUNK_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();

        let stream = futures::stream::iter(chunks).map(move |chunk| {
            let len = chunk.len() as u64;
            let tx = progress_tx.clone();
            let _ = tx.try_send(len);
            Ok::<_, std::io::Error>(chunk)
        });

        let body = reqwest::Body::wrap_stream(stream);

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_token)
            .query(&[
                ("session_id", session_id),
                ("part_number", &part_number.to_string()),
            ])
            .header("Content-Type", "application/octet-stream")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AdaptiveError::ChunkedUploadPartFailed {
                part_number,
                status: response.status().to_string(),
                body: response.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Abort a chunked upload session.
    pub async fn abort_chunked_upload(&self, session_id: &str) -> Result<()> {
        let url = self.rest_base_url.join(ABORT_CHUNKED_UPLOAD_ROUTE)?;

        let request = AbortChunkedUploadRequest {
            session_id: session_id.to_string(),
        };

        let _ = self
            .client
            .delete(url)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await;

        Ok(())
    }

    async fn create_dataset_from_multipart(
        &self,
        project: &str,
        name: &str,
        key: &str,
        session_id: &str,
    ) -> Result<
        create_dataset_from_multipart::CreateDatasetFromMultipartCreateDatasetFromMultipartUpload,
    > {
        let variables = create_dataset_from_multipart::Variables {
            input: create_dataset_from_multipart::DatasetCreateFromMultipartUpload {
                project: project.to_string(),
                name: name.to_string(),
                key: Some(key.to_string()),
                source: None,
                upload_session_id: session_id.to_string(),
            },
        };

        let response_data = self
            .execute_query(CreateDatasetFromMultipart, variables)
            .await?;
        Ok(response_data.create_dataset_from_multipart_upload)
    }

    pub fn chunked_upload_dataset<'a, P: AsRef<Path> + Send + 'a>(
        &'a self,
        project: &'a str,
        name: &'a str,
        key: &'a str,
        dataset: P,
    ) -> Result<BoxStream<'a, Result<UploadEvent>>> {
        let file_size = std::fs::metadata(dataset.as_ref())?.len();

        let (total_parts, chunk_size) = calculate_upload_parts(file_size)?;

        let stream = async_stream::try_stream! {
            yield UploadEvent::Progress(ChunkedUploadProgress {
                bytes_uploaded: 0,
                total_bytes: file_size,
            });

            let session_id = self.init_chunked_upload(total_parts).await?;

            let mut file = File::open(dataset.as_ref())?;
            let mut buffer = vec![0u8; chunk_size as usize];
            let mut bytes_uploaded = 0u64;

            let (progress_tx, mut progress_rx) = mpsc::channel::<u64>(64);

            for part_number in 1..=total_parts {
                let bytes_read = file.read(&mut buffer)?;
                let chunk_data = buffer[..bytes_read].to_vec();

                let upload_fut = self.upload_part(&session_id, part_number, chunk_data, progress_tx.clone());
                tokio::pin!(upload_fut);

                let upload_result: Result<()> = loop {
                    tokio::select! {
                        biased;
                        result = &mut upload_fut => {
                            break result;
                        }
                        Some(bytes) = progress_rx.recv() => {
                            bytes_uploaded += bytes;
                            yield UploadEvent::Progress(ChunkedUploadProgress {
                                bytes_uploaded,
                                total_bytes: file_size,
                            });
                        }
                    }
                };

                if let Err(e) = upload_result {
                    let _ = self.abort_chunked_upload(&session_id).await;
                    Err(e)?;
                }
            }

            let create_result = self
                .create_dataset_from_multipart(project, name, key, &session_id)
                .await;

            match create_result {
                Ok(response) => {
                    yield UploadEvent::Complete(response);
                }
                Err(e) => {
                    let _ = self.abort_chunked_upload(&session_id).await;
                    Err(AdaptiveError::DatasetCreationFailed(e.to_string()))?;
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Download a file from the given URL and write it to the specified path.
    /// The URL can be absolute or relative to the API base URL.
    pub async fn download_file_to_path(&self, url: &str, dest_path: &Path) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let full_url = if url.starts_with("http://") || url.starts_with("https://") {
            Url::parse(url)?
        } else {
            self.rest_base_url.join(url)?
        };

        let response = self
            .client
            .get(full_url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AdaptiveError::HttpError(
                response.error_for_status().unwrap_err(),
            ));
        }

        let mut file = tokio::fs::File::create(dest_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(())
    }
}
