//! Local cache for registry client lookups.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::model::RegisteredArtifact;

/// Disk/memory cache keyed by registry id + artifact id@version.
#[derive(Debug, Default, Clone)]
pub struct RegistryCache {
    memory: HashMap<String, CachedEntry>,
    root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct CachedEntry {
    artifact: RegisteredArtifact,
    content: Option<String>,
    etag: Option<String>,
}

impl RegistryCache {
    /// Create an in-memory cache.
    pub fn memory() -> Self {
        Self::default()
    }

    /// Create a cache that also persists under `root`.
    pub fn disk(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|err| Error::Io {
            path: root.clone(),
            source: err,
        })?;
        Ok(Self {
            memory: HashMap::new(),
            root: Some(root),
        })
    }

    fn key(registry_id: &str, artifact_id: &str, version: &str) -> String {
        format!("{registry_id}|{artifact_id}|{version}")
    }

    /// Store artifact metadata (and optional content).
    pub fn put(
        &mut self,
        registry_id: &str,
        artifact: RegisteredArtifact,
        content: Option<String>,
        etag: Option<String>,
    ) -> Result<()> {
        let key = Self::key(registry_id, &artifact.id, &artifact.version);
        if let Some(root) = &self.root {
            let path = root.join(sanitize(&key));
            let payload = serde_json::json!({
                "artifact": artifact,
                "content": content,
                "etag": etag,
            });
            fs::write(
                &path,
                serde_json::to_string_pretty(&payload).map_err(|err| {
                    Error::Serialization(format!("cache serialize failed: {err}"))
                })?,
            )
            .map_err(|err| Error::Io {
                path: path.clone(),
                source: err,
            })?;
        }
        self.memory.insert(
            key,
            CachedEntry {
                artifact,
                content,
                etag,
            },
        );
        Ok(())
    }

    /// Lookup cached artifact.
    pub fn get(
        &self,
        registry_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Option<(RegisteredArtifact, Option<String>, Option<String>)> {
        let key = Self::key(registry_id, artifact_id, version);
        if let Some(entry) = self.memory.get(&key) {
            return Some((
                entry.artifact.clone(),
                entry.content.clone(),
                entry.etag.clone(),
            ));
        }
        let root = self.root.as_ref()?;
        let path = root.join(sanitize(&key));
        let raw = fs::read_to_string(path).ok()?;
        let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let artifact: RegisteredArtifact =
            serde_json::from_value(value.get("artifact")?.clone()).ok()?;
        let content = value
            .get("content")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let etag = value
            .get("etag")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        Some((artifact, content, etag))
    }

    /// Clear memory entries; also removes disk files when configured.
    pub fn clear(&mut self) -> Result<()> {
        self.memory.clear();
        if let Some(root) = &self.root {
            if root.is_dir() {
                for entry in fs::read_dir(root).map_err(|err| Error::Io {
                    path: root.clone(),
                    source: err,
                })? {
                    let entry = entry.map_err(|err| Error::Io {
                        path: root.clone(),
                        source: err,
                    })?;
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        Ok(())
    }

    /// Cache root directory when disk-backed.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
}

fn sanitize(key: &str) -> String {
    key.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
