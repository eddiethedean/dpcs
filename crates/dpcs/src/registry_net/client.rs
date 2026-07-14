//! HTTP client for the DPCS reference registry API.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{RegisteredArtifact, Registry};
use crate::registry_net::RegistryCache;

/// Client errors for registry network operations.
#[derive(Debug, thiserror::Error)]
pub enum RegistryClientError {
    /// Transport or HTTP stack failure.
    #[error("registry transport error: {0}")]
    Transport(String),
    /// Unexpected HTTP status.
    #[error("registry HTTP {status}: {message}")]
    Http {
        /// Status code.
        status: u16,
        /// Response message / body excerpt.
        message: String,
        /// Optional structured diagnostics from the server.
        report: Option<ValidationReport>,
    },
    /// Response body could not be decoded.
    #[error("registry decode error: {0}")]
    Decode(String),
}

impl RegistryClientError {
    /// Convert into package/client diagnostics.
    pub fn into_diagnostics(self) -> ValidationReport {
        let mut report = ValidationReport::new();
        match self {
            Self::Transport(message) => report.push(
                Diagnostic::error("DPCS-REGC-001", categories::REGISTRY_CLIENT, message)
                    .with_remediation("Check network connectivity and registry base URL"),
            ),
            Self::Http {
                status,
                message,
                report: server_report,
            } => {
                if let Some(server_report) = server_report {
                    report.extend(server_report);
                }
                report.push(
                    Diagnostic::error(
                        "DPCS-REGC-002",
                        categories::REGISTRY_CLIENT,
                        format!("HTTP {status}: {message}"),
                    )
                    .with_remediation("Inspect registry server logs and request payload"),
                );
            }
            Self::Decode(message) => report.push(
                Diagnostic::error("DPCS-REGC-003", categories::REGISTRY_CLIENT, message)
                    .with_remediation("Ensure the server speaks the DPCS reference registry API"),
            ),
        }
        report
    }
}

/// Publish / update request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishRequest {
    /// Artifact metadata (id in path must match `artifact.id` when present).
    pub artifact: RegisteredArtifact,
    /// Optional UTF-8 payload content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Content encoding (`utf-8` default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
}

/// Blocking HTTP client for the reference registry API.
#[derive(Debug, Clone)]
pub struct RegistryClient {
    base: Url,
    http: Client,
    token: Option<String>,
    cache: RegistryCache,
}

impl RegistryClient {
    /// Create a client targeting `base_url` (for example `http://127.0.0.1:8080`).
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, RegistryClientError> {
        let base = Url::parse(base_url.as_ref())
            .map_err(|err| RegistryClientError::Transport(format!("invalid base URL: {err}")))?;
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Ok(Self {
            base,
            http,
            token: None,
            cache: RegistryCache::memory(),
        })
    }

    /// Attach an optional bearer token.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Replace the cache backend.
    pub fn with_cache(mut self, cache: RegistryCache) -> Self {
        self.cache = cache;
        self
    }

    /// Mutable access to the cache.
    pub fn cache_mut(&mut self) -> &mut RegistryCache {
        &mut self.cache
    }

    fn url(&self, path: &str) -> Result<Url, RegistryClientError> {
        self.base
            .join(path.trim_start_matches('/'))
            .map_err(|err| RegistryClientError::Transport(err.to_string()))
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<reqwest::blocking::RequestBuilder, RegistryClientError> {
        let url = self.url(path)?;
        let mut req = self.http.request(method, url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        Ok(req)
    }

    /// `GET /v1/registry`
    pub fn get_registry(&self) -> Result<Registry, RegistryClientError> {
        let response = self
            .request(reqwest::Method::GET, "/v1/registry")?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `GET /v1/artifacts`
    pub fn list(
        &self,
        artifact_type: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<RegisteredArtifact>, RegistryClientError> {
        let mut path = "/v1/artifacts".to_owned();
        let mut query = Vec::new();
        if let Some(t) = artifact_type {
            query.push(format!("type={t}"));
        }
        if let Some(s) = status {
            query.push(format!("status={s}"));
        }
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query.join("&"));
        }
        let response = self
            .request(reqwest::Method::GET, &path)?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `GET /v1/artifacts/{id}`
    pub fn lookup(
        &mut self,
        id: &str,
        version: Option<&str>,
    ) -> Result<RegisteredArtifact, RegistryClientError> {
        let mut path = format!("/v1/artifacts/{id}");
        if let Some(v) = version {
            path.push_str(&format!("?version={v}"));
        }
        let response = self
            .request(reqwest::Method::GET, &path)?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        let artifact: RegisteredArtifact = Self::json(response)?;
        let registry = self.get_registry().ok();
        let registry_id = registry
            .as_ref()
            .map(|r| r.id.as_str())
            .unwrap_or("unknown");
        let _ = self.cache.put(registry_id, artifact.clone(), None, None);
        Ok(artifact)
    }

    /// `GET /v1/artifacts/{id}/content`
    pub fn fetch_content(
        &mut self,
        id: &str,
        version: Option<&str>,
    ) -> Result<String, RegistryClientError> {
        let mut path = format!("/v1/artifacts/{id}/content");
        if let Some(v) = version {
            path.push_str(&format!("?version={v}"));
        }
        let response = self
            .request(reqwest::Method::GET, &path)?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        if !status.is_success() {
            return Err(RegistryClientError::Http {
                status: status.as_u16(),
                message: body,
                report: None,
            });
        }
        Ok(body)
    }

    /// `PUT /v1/artifacts/{id}`
    pub fn publish(
        &self,
        id: &str,
        request: &PublishRequest,
    ) -> Result<RegisteredArtifact, RegistryClientError> {
        let response = self
            .request(reqwest::Method::PUT, &format!("/v1/artifacts/{id}"))?
            .json(request)
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `POST /v1/artifacts/{id}/deprecate`
    pub fn deprecate(&self, id: &str) -> Result<RegisteredArtifact, RegistryClientError> {
        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/v1/artifacts/{id}/deprecate"),
            )?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `POST /v1/artifacts/{id}/retire`
    pub fn retire(&self, id: &str) -> Result<RegisteredArtifact, RegistryClientError> {
        let response = self
            .request(reqwest::Method::POST, &format!("/v1/artifacts/{id}/retire"))?
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    fn json<T: for<'de> Deserialize<'de>>(
        response: reqwest::blocking::Response,
    ) -> Result<T, RegistryClientError> {
        let status = response.status();
        let body = response
            .text()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        if status == StatusCode::BAD_REQUEST {
            let report = serde_json::from_str::<ValidationReport>(&body).ok();
            return Err(RegistryClientError::Http {
                status: status.as_u16(),
                message: body,
                report,
            });
        }
        if !status.is_success() {
            return Err(RegistryClientError::Http {
                status: status.as_u16(),
                message: body,
                report: None,
            });
        }
        serde_json::from_str(&body).map_err(|err| RegistryClientError::Decode(err.to_string()))
    }
}
