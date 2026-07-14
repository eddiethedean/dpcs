//! HTTP client for the DPCS reference registry API.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;

use crate::diagnostics::{categories, Diagnostic, ValidationReport};
use crate::model::{RegisteredArtifact, Registry};
use crate::registry_net::types::PublishRequest;
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

    /// True when the failure is transport-level (network / DNS / timeout).
    pub fn is_transport(&self) -> bool {
        matches!(self, Self::Transport(_))
    }

    /// True when the HTTP status indicates a server / gateway failure.
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Http { status, .. } if *status >= 500)
    }
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
    /// Create a client targeting `base_url` (for example `http://127.0.0.1:8080/`).
    ///
    /// The base URL path is normalized to end with `/` so joins append under the
    /// configured path prefix (for example `/proxy/` stays as a prefix).
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, RegistryClientError> {
        let mut base = Url::parse(base_url.as_ref())
            .map_err(|err| RegistryClientError::Transport(format!("invalid base URL: {err}")))?;
        if !base.path().ends_with('/') {
            let path = format!("{}/", base.path().trim_end_matches('/'));
            base.set_path(&path);
        }
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

    /// Stable namespace for cache keys derived from the base URL.
    fn cache_namespace(&self) -> String {
        format!(
            "{}{}",
            self.base.host_str().unwrap_or("localhost"),
            self.base.path()
        )
    }

    fn absolute(
        &self,
        segments: &[&str],
        query: &[(&str, &str)],
    ) -> Result<Url, RegistryClientError> {
        let mut url = self.base.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| RegistryClientError::Transport("cannot-be-a-base URL".into()))?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }

    fn artifact_url(
        &self,
        id: &str,
        extra_segments: &[&str],
        query: &[(&str, &str)],
    ) -> Result<Url, RegistryClientError> {
        let mut segments = vec!["v1", "artifacts", id];
        segments.extend_from_slice(extra_segments);
        self.absolute(&segments, query)
    }

    fn request_url(&self, method: reqwest::Method, url: Url) -> reqwest::blocking::RequestBuilder {
        let mut req = self.http.request(method, url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    /// `GET /v1/registry`
    pub fn get_registry(&self) -> Result<Registry, RegistryClientError> {
        let url = self.absolute(&["v1", "registry"], &[])?;
        let response = self
            .request_url(reqwest::Method::GET, url)
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
        let mut query = Vec::new();
        if let Some(t) = artifact_type {
            query.push(("type", t));
        }
        if let Some(s) = status {
            query.push(("status", s));
        }
        let url = self.absolute(&["v1", "artifacts"], &query)?;
        let response = self
            .request_url(reqwest::Method::GET, url)
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
        let ns = self.cache_namespace();
        if let Some(version) = version {
            if let Some((artifact, _, _)) = self.cache.get(&ns, id, version) {
                return Ok(artifact);
            }
        }

        let mut query = Vec::new();
        if let Some(v) = version {
            query.push(("version", v));
        }
        let url = self.artifact_url(id, &[], &query)?;
        let response = self
            .request_url(reqwest::Method::GET, url)
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        let artifact: RegisteredArtifact = Self::json(response)?;
        let _ = self.cache.put(&ns, artifact.clone(), None, None);
        Ok(artifact)
    }

    /// `GET /v1/artifacts/{id}/content`
    pub fn fetch_content(
        &mut self,
        id: &str,
        version: Option<&str>,
    ) -> Result<String, RegistryClientError> {
        let ns = self.cache_namespace();
        if let Some(version) = version {
            if let Some((_, Some(content), _)) = self.cache.get(&ns, id, version) {
                return Ok(content);
            }
        }

        let mut query = Vec::new();
        if let Some(v) = version {
            query.push(("version", v));
        }
        let url = self.artifact_url(id, &["content"], &query)?;
        let response = self
            .request_url(reqwest::Method::GET, url)
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
        if let Some(version) = version {
            if let Some((artifact, _, etag)) = self.cache.get(&ns, id, version) {
                let _ = self.cache.put(&ns, artifact, Some(body.clone()), etag);
            }
        }
        Ok(body)
    }

    /// `PUT /v1/artifacts/{id}`
    pub fn publish(
        &self,
        id: &str,
        request: &PublishRequest,
    ) -> Result<RegisteredArtifact, RegistryClientError> {
        let url = self.artifact_url(id, &[], &[])?;
        let response = self
            .request_url(reqwest::Method::PUT, url)
            .json(request)
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `POST /v1/artifacts/{id}/deprecate`
    pub fn deprecate(
        &self,
        id: &str,
        version: Option<&str>,
    ) -> Result<RegisteredArtifact, RegistryClientError> {
        let mut query = Vec::new();
        if let Some(v) = version {
            query.push(("version", v));
        }
        let url = self.artifact_url(id, &["deprecate"], &query)?;
        let response = self
            .request_url(reqwest::Method::POST, url)
            .send()
            .map_err(|err| RegistryClientError::Transport(err.to_string()))?;
        Self::json(response)
    }

    /// `POST /v1/artifacts/{id}/retire`
    pub fn retire(
        &self,
        id: &str,
        version: Option<&str>,
    ) -> Result<RegisteredArtifact, RegistryClientError> {
        let mut query = Vec::new();
        if let Some(v) = version {
            query.push(("version", v));
        }
        let url = self.artifact_url(id, &["retire"], &query)?;
        let response = self
            .request_url(reqwest::Method::POST, url)
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
