//! File-backed reference registry HTTP server.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::model::{validate_registry, RegisteredArtifact, Registry};
use crate::registry_net::client::PublishRequest;

/// Options for `dpcs registry serve`.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// Root directory containing `registry.yaml` and artifact files.
    pub root: PathBuf,
    /// Bind address (for example `127.0.0.1:8080`).
    pub bind: SocketAddr,
    /// Optional bearer token required for mutating operations.
    pub token: Option<String>,
}

/// Shared axum state for the file-backed registry server.
#[derive(Clone)]
pub struct AppState {
    root: PathBuf,
    token: Option<String>,
    lock: Arc<Mutex<()>>,
}

impl AppState {
    /// Create server state for a file-backed registry root.
    pub fn new(root: PathBuf, token: Option<String>) -> Self {
        Self {
            root,
            token,
            lock: Arc::new(Mutex::new(())),
        }
    }
}

/// Run the reference registry server until interrupted.
pub async fn serve(options: ServeOptions) -> Result<(), String> {
    let state = AppState::new(options.root.clone(), options.token.clone());
    ensure_registry(&state.root)?;
    let app = router(state);
    let listener = TcpListener::bind(options.bind)
        .await
        .map_err(|err| format!("bind failed: {err}"))?;
    axum::serve(listener, app)
        .await
        .map_err(|err| format!("server error: {err}"))
}

/// Serve on an already-bound listener (useful for tests).
pub async fn serve_listener(
    listener: TcpListener,
    root: PathBuf,
    token: Option<String>,
) -> Result<(), String> {
    let state = AppState::new(root.clone(), token);
    ensure_registry(&state.root)?;
    let app = router(state);
    axum::serve(listener, app)
        .await
        .map_err(|err| format!("server error: {err}"))
}

/// Build the axum router (useful for tests).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(|| async { StatusCode::OK }))
        .route("/v1/registry", get(get_registry))
        .route("/v1/artifacts", get(list_artifacts))
        .route(
            "/v1/artifacts/{id}",
            get(lookup_artifact).put(publish_artifact),
        )
        .route("/v1/artifacts/{id}/content", get(fetch_content))
        .route("/v1/artifacts/{id}/deprecate", post(deprecate_artifact))
        .route("/v1/artifacts/{id}/retire", post(retire_artifact))
        .with_state(state)
}

fn ensure_registry(root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|err| err.to_string())?;
    let path = registry_path(root);
    if !path.is_file() {
        let registry = Registry {
            id: "local".into(),
            version: "0.1.0".into(),
            dpcs_version: crate::DPCS_SPEC_VERSION.into(),
            owner: "local".into(),
            publication_status: Some("draft".into()),
            published_at: None,
            governance: None,
            security: None,
            artifacts: Vec::new(),
            extensions: Default::default(),
        };
        write_registry(root, &registry)?;
    }
    Ok(())
}

fn registry_path(root: &Path) -> PathBuf {
    let yaml = root.join("registry.yaml");
    if yaml.is_file() {
        yaml
    } else if root.join("registry.yml").is_file() {
        root.join("registry.yml")
    } else if root.join("registry.json").is_file() {
        root.join("registry.json")
    } else {
        yaml
    }
}

fn read_registry(root: &Path) -> Result<Registry, String> {
    let path = registry_path(root);
    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        let raw = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
        serde_json::from_str(&raw).map_err(|err| err.to_string())
    } else {
        Registry::from_file(&path).map_err(|err| err.to_string())
    }
}

fn write_registry(root: &Path, registry: &Registry) -> Result<(), String> {
    let path = root.join("registry.yaml");
    let yaml = serde_yaml::to_string(registry).map_err(|err| err.to_string())?;
    std::fs::write(path, yaml).map_err(|err| err.to_string())
}

fn authorize(headers: &HeaderMap, token: &Option<String>) -> Result<(), ApiError> {
    let Some(expected) = token else {
        return Ok(());
    };
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let got = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if got == expected {
        Ok(())
    } else {
        Err(ApiError::unauthorized())
    }
}

async fn get_registry(State(state): State<AppState>) -> Result<Json<Registry>, ApiError> {
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    Ok(Json(read_registry(&state.root)?))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(rename = "type")]
    artifact_type: Option<String>,
    status: Option<String>,
}

async fn list_artifacts(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<RegisteredArtifact>>, ApiError> {
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    let registry = read_registry(&state.root)?;
    let items = registry
        .artifacts
        .into_iter()
        .filter(|a| {
            query
                .artifact_type
                .as_ref()
                .map(|t| &a.artifact_type == t)
                .unwrap_or(true)
                && query
                    .status
                    .as_ref()
                    .map(|s| a.publication_status.as_ref() == Some(s))
                    .unwrap_or(true)
        })
        .collect();
    Ok(Json(items))
}

#[derive(Debug, Deserialize)]
struct VersionQuery {
    version: Option<String>,
}

async fn lookup_artifact(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<VersionQuery>,
) -> Result<Json<RegisteredArtifact>, ApiError> {
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    let registry = read_registry(&state.root)?;
    find_artifact(&registry, &id, query.version.as_deref())
        .map(Json)
        .ok_or_else(ApiError::not_found)
}

async fn fetch_content(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<VersionQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    let registry = read_registry(&state.root)?;
    let artifact =
        find_artifact(&registry, &id, query.version.as_deref()).ok_or_else(ApiError::not_found)?;
    let location = artifact.location.as_ref().ok_or_else(ApiError::not_found)?;
    let path = state.root.join(location);
    let body = std::fs::read_to_string(&path).map_err(|_| ApiError::not_found())?;
    Ok((StatusCode::OK, body))
}

async fn publish_artifact(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    Json(request): Json<PublishRequest>,
) -> Result<Json<RegisteredArtifact>, ApiError> {
    authorize(&headers, &state.token)?;
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    let mut artifact = request.artifact;
    if artifact.id != id {
        artifact.id = id.clone();
    }
    if let Some(content) = request.content {
        let rel = format!("artifacts/{id}-{}.yaml", artifact.version);
        let path = state.root.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| ApiError::internal(err.to_string()))?;
        }
        std::fs::write(&path, content).map_err(|err| ApiError::internal(err.to_string()))?;
        artifact.location = Some(rel);
    }
    let mut registry = read_registry(&state.root)?;
    if let Some(existing) = registry
        .artifacts
        .iter_mut()
        .find(|a| a.id == artifact.id && a.version == artifact.version)
    {
        *existing = artifact.clone();
    } else {
        registry.artifacts.push(artifact.clone());
    }
    let report = validate_registry(&registry);
    if !report.is_valid() {
        return Err(ApiError::bad_request(report));
    }
    write_registry(&state.root, &registry)?;
    Ok(Json(artifact))
}

async fn deprecate_artifact(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<RegisteredArtifact>, ApiError> {
    set_status(state, headers, id, "deprecated").await
}

async fn retire_artifact(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<RegisteredArtifact>, ApiError> {
    set_status(state, headers, id, "retired").await
}

async fn set_status(
    state: AppState,
    headers: HeaderMap,
    id: String,
    status: &str,
) -> Result<Json<RegisteredArtifact>, ApiError> {
    authorize(&headers, &state.token)?;
    let _guard = state.lock.lock().map_err(|_| ApiError::internal("lock"))?;
    let mut registry = read_registry(&state.root)?;
    let artifact = registry
        .artifacts
        .iter_mut()
        .rev()
        .find(|a| a.id == id)
        .ok_or_else(ApiError::not_found)?;
    artifact.publication_status = Some(status.to_owned());
    let out = artifact.clone();
    let report = validate_registry(&registry);
    if !report.is_valid() {
        return Err(ApiError::bad_request(report));
    }
    write_registry(&state.root, &registry)?;
    Ok(Json(out))
}

fn find_artifact(
    registry: &Registry,
    id: &str,
    version: Option<&str>,
) -> Option<RegisteredArtifact> {
    registry
        .artifacts
        .iter()
        .rev()
        .find(|a| a.id == id && version.map(|v| a.version == v).unwrap_or(true))
        .cloned()
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    body: ApiBody,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ApiBody {
    Message { error: String },
    Report(crate::diagnostics::ValidationReport),
}

impl ApiError {
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ApiBody::Message {
                error: "not found".into(),
            },
        }
    }
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: ApiBody::Message {
                error: "unauthorized".into(),
            },
        }
    }
    fn bad_request(report: crate::diagnostics::ValidationReport) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiBody::Report(report),
        }
    }
    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiBody::Message {
                error: message.into(),
            },
        }
    }
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        ApiError::internal(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}
