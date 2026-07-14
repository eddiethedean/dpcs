//! Pack / unpack / resolve pipeline packages.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use walkdir::WalkDir;

use super::manifest::{PackageArtifactEntry, PackageManifest};
use crate::error::{Error, Result};
use crate::model::ExtensionMap;

/// On-disk layout of an unpacked `.dpcspkg`.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageLayout {
    /// Absolute path to the package directory.
    pub root: PathBuf,
    /// Parsed manifest.
    pub manifest: PackageManifest,
}

impl PackageLayout {
    /// Load an unpacked package directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let manifest_path = root.join("manifest.yaml");
        let alt = root.join("manifest.yml");
        let json = root.join("manifest.json");
        let contents = if manifest_path.is_file() {
            fs::read_to_string(&manifest_path).map_err(|err| Error::Io {
                path: manifest_path.clone(),
                source: err,
            })?
        } else if alt.is_file() {
            fs::read_to_string(&alt).map_err(|err| Error::Io {
                path: alt.clone(),
                source: err,
            })?
        } else if json.is_file() {
            fs::read_to_string(&json).map_err(|err| Error::Io {
                path: json.clone(),
                source: err,
            })?
        } else {
            return Err(Error::Serialization(
                "package root missing manifest.yaml / manifest.yml / manifest.json".to_owned(),
            ));
        };

        let manifest: PackageManifest =
            if json.is_file() && !manifest_path.is_file() && !alt.is_file() {
                serde_json::from_str(&contents).map_err(|err| {
                    Error::Serialization(format!("failed to parse package manifest JSON: {err}"))
                })?
            } else {
                serde_yaml::from_str(&contents).map_err(|err| {
                    Error::Serialization(format!("failed to parse package manifest YAML: {err}"))
                })?
            };

        Ok(Self { root, manifest })
    }

    /// Resolve an artifact path by id (and optional version).
    pub fn resolve_path(&self, id: &str, version: Option<&str>) -> Option<PathBuf> {
        self.manifest
            .artifacts
            .iter()
            .find(|a| a.id == id && version.map(|v| a.version == v).unwrap_or(true))
            .map(|a| self.root.join(&a.path))
    }
}

/// Resolve an artifact file path within a package by id/version.
pub fn resolve_artifact(
    package_root: impl AsRef<Path>,
    id: &str,
    version: Option<&str>,
) -> Result<PathBuf> {
    let layout = PackageLayout::open(package_root)?;
    layout.resolve_path(id, version).ok_or_else(|| {
        Error::Serialization(format!(
            "artifact `{id}`{} not found in package",
            version.map(|v| format!("@{v}")).unwrap_or_default()
        ))
    })
}

/// Create a package directory from a manifest and source files already at `root`.
///
/// `root` must already contain `manifest.yaml` and referenced artifact files.
/// When `archive` is `Some`, also writes a zip archive at that path.
pub fn pack(root: impl AsRef<Path>, archive: Option<&Path>) -> Result<PackageLayout> {
    let layout = PackageLayout::open(root)?;
    validate_layout_paths(&layout)?;
    if let Some(archive_path) = archive {
        write_zip(&layout.root, archive_path)?;
    }
    Ok(layout)
}

/// Unpack a `.dpcspkg` zip archive (or copy a directory) into `dest`.
pub fn unpack(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<PackageLayout> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    if source.is_dir() {
        copy_dir(source, dest)?;
    } else {
        extract_zip(source, dest)?;
    }
    PackageLayout::open(dest)
}

fn validate_layout_paths(layout: &PackageLayout) -> Result<()> {
    for entry in &layout.manifest.artifacts {
        validate_relative_path(&entry.path)?;
        let path = layout.root.join(&entry.path);
        if !path.is_file() {
            return Err(Error::Io {
                path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "artifact file missing"),
            });
        }
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<()> {
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(Error::Serialization(format!(
            "artifact path must be relative: {path}"
        )));
    }
    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err(Error::Serialization(format!(
                "artifact path must not contain '..': {path}"
            )));
        }
    }
    Ok(())
}

fn write_zip(root: &Path, archive: &Path) -> Result<()> {
    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent).map_err(|err| Error::Io {
            path: parent.to_path_buf(),
            source: err,
        })?;
    }
    let file = File::create(archive).map_err(|err| Error::Io {
        path: archive.to_path_buf(),
        source: err,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let rel = path.strip_prefix(root).map_err(|_| {
            Error::Serialization(format!(
                "failed to relativize {} under {}",
                path.display(),
                root.display()
            ))
        })?;
        let name = rel.to_string_lossy().replace('\\', "/");
        zip.start_file(name, options)
            .map_err(|err| Error::Serialization(format!("zip start_file failed: {err}")))?;
        let mut f = File::open(path).map_err(|err| Error::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|err| Error::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        zip.write_all(&buf)
            .map_err(|err| Error::Serialization(format!("zip write failed: {err}")))?;
    }
    zip.finish()
        .map_err(|err| Error::Serialization(format!("zip finish failed: {err}")))?;
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|err| Error::Io {
        path: dest.to_path_buf(),
        source: err,
    })?;
    let file = File::open(archive).map_err(|err| Error::Io {
        path: archive.to_path_buf(),
        source: err,
    })?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|err| Error::Serialization(format!("invalid zip archive: {err}")))?;
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|err| Error::Serialization(format!("zip entry error: {err}")))?;
        let name = file
            .enclosed_name()
            .ok_or_else(|| Error::Serialization("zip entry has unsafe path".to_owned()))?
            .to_path_buf();
        let out = dest.join(&name);
        if file.is_dir() {
            fs::create_dir_all(&out).map_err(|err| Error::Io {
                path: out.clone(),
                source: err,
            })?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|err| Error::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
        }
        let mut outfile = File::create(&out).map_err(|err| Error::Io {
            path: out.clone(),
            source: err,
        })?;
        std::io::copy(&mut file, &mut outfile).map_err(|err| Error::Io {
            path: out,
            source: err,
        })?;
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|err| Error::Io {
        path: dest.to_path_buf(),
        source: err,
    })?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .map_err(|_| Error::Serialization("failed to relativize source path".to_owned()))?;
        let target = dest.join(rel);
        if path.is_dir() {
            fs::create_dir_all(&target).map_err(|err| Error::Io {
                path: target,
                source: err,
            })?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|err| Error::Io {
                    path: parent.to_path_buf(),
                    source: err,
                })?;
            }
            fs::copy(path, &target).map_err(|err| Error::Io {
                path: target,
                source: err,
            })?;
        }
    }
    Ok(())
}

/// Convenience builder to create a minimal package directory.
pub fn write_manifest(root: impl AsRef<Path>, manifest: &PackageManifest) -> Result<()> {
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|err| Error::Io {
        path: root.to_path_buf(),
        source: err,
    })?;
    let path = root.join("manifest.yaml");
    let yaml = serde_yaml::to_string(manifest)
        .map_err(|err| Error::Serialization(format!("failed to serialize manifest: {err}")))?;
    fs::write(&path, yaml).map_err(|err| Error::Io { path, source: err })?;
    Ok(())
}

/// Helper to construct a simple artifact entry.
pub fn artifact_entry(
    id: impl Into<String>,
    artifact_type: impl Into<String>,
    version: impl Into<String>,
    path: impl Into<String>,
) -> PackageArtifactEntry {
    PackageArtifactEntry {
        id: id.into(),
        artifact_type: artifact_type.into(),
        version: version.into(),
        path: path.into(),
        extensions: ExtensionMap::default(),
    }
}
