//! Pack / unpack / resolve pipeline packages.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::manifest::{PackageArtifactEntry, PackageManifest};
use super::validate::validate_package;
use crate::error::{Error, Result};
use crate::model::ExtensionMap;
use crate::paths::{is_safe_relative, join_under_root};

/// On-disk layout of an unpacked `.dpcspkg`.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageLayout {
    /// Absolute (canonical when possible) path to the package directory.
    pub root: PathBuf,
    /// Parsed manifest.
    pub manifest: PackageManifest,
}

impl PackageLayout {
    /// Load an unpacked package directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root_input = root.as_ref();
        let root = if root_input.exists() {
            root_input.canonicalize().map_err(|err| Error::Io {
                path: root_input.to_path_buf(),
                source: err,
            })?
        } else {
            root_input.to_path_buf()
        };
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
    ///
    /// When `version` is omitted and multiple entries share an id, the last
    /// matching manifest entry is used (same “latest listed” convention as the
    /// reference registry).
    ///
    /// Returns `None` when the id/version is missing or the declared path escapes
    /// the package root.
    pub fn resolve_path(&self, id: &str, version: Option<&str>) -> Option<PathBuf> {
        let entry = self
            .manifest
            .artifacts
            .iter()
            .rev()
            .find(|a| a.id == id && version.map(|v| a.version == v).unwrap_or(true))?;
        join_under_root(&self.root, &entry.path).ok()
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
            "artifact `{id}`{} not found in package (or path escapes package root)",
            version.map(|v| format!("@{v}")).unwrap_or_default()
        ))
    })
}

/// Create a package directory from a manifest and source files already at `root`.
///
/// `root` must already contain a manifest and referenced artifact files.
/// When `archive` is `Some`, also writes a zip archive at that path (must not be
/// under the package root).
pub fn pack(root: impl AsRef<Path>, archive: Option<&Path>) -> Result<PackageLayout> {
    let layout = PackageLayout::open(root)?;
    let report = validate_package(&layout.root);
    if !report.is_valid() {
        return Err(Error::InvalidDocument { report });
    }
    validate_layout_paths(&layout)?;
    if let Some(archive_path) = archive {
        ensure_archive_outside_root(&layout.root, archive_path)?;
        write_zip(&layout.root, archive_path)?;
    }
    Ok(layout)
}

/// Unpack a `.dpcspkg` zip archive (or copy a directory) into `dest`.
pub fn unpack(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<PackageLayout> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    if source.is_dir() {
        ensure_dest_not_under_src(source, dest)?;
        copy_dir(source, dest)?;
    } else {
        extract_zip(source, dest)?;
    }
    PackageLayout::open(dest)
}

fn ensure_archive_outside_root(root: &Path, archive: &Path) -> Result<()> {
    let root_abs = root.canonicalize().map_err(|err| Error::Io {
        path: root.to_path_buf(),
        source: err,
    })?;
    let candidate = resolve_archive_candidate(archive)?;
    if candidate.starts_with(&root_abs) {
        return Err(Error::Serialization(
            "archive path must be outside the package root".to_owned(),
        ));
    }
    Ok(())
}

/// Resolve an archive path to an absolute location without requiring the full
/// path to exist. Uses the deepest existing ancestor's canonical path so
/// platform symlink prefixes (for example `/var` → `/private/var` on macOS)
/// compare consistently with a canonicalized package root.
fn resolve_archive_candidate(archive: &Path) -> Result<PathBuf> {
    use std::ffi::OsString;
    use std::path::Component;

    let joined = if archive.is_absolute() {
        archive.to_path_buf()
    } else {
        let cwd = std::env::current_dir().map_err(|err| Error::Io {
            path: PathBuf::from("."),
            source: err,
        })?;
        cwd.join(archive)
    };

    let mut suffix: Vec<OsString> = Vec::new();
    let mut cursor = joined.as_path();
    loop {
        if cursor.exists() {
            let mut abs = cursor.canonicalize().map_err(|err| Error::Io {
                path: cursor.to_path_buf(),
                source: err,
            })?;
            for segment in suffix.iter().rev() {
                abs.push(segment);
            }
            return Ok(abs);
        }
        match cursor.file_name() {
            Some(name) => {
                suffix.push(name.to_os_string());
                match cursor.parent() {
                    Some(parent) if !parent.as_os_str().is_empty() => cursor = parent,
                    _ => break,
                }
            }
            None => break,
        }
    }

    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(Error::Serialization(
                        "archive path escapes filesystem root".to_owned(),
                    ));
                }
            }
            Component::Normal(seg) => out.push(seg),
        }
    }
    Ok(out)
}

fn ensure_dest_not_under_src(src: &Path, dest: &Path) -> Result<()> {
    let src_abs = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    let dest_abs = if dest.exists() {
        dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf())
    } else if let Some(parent) = dest.parent() {
        parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf())
            .join(dest.file_name().unwrap_or_default())
    } else {
        dest.to_path_buf()
    };
    if dest_abs.starts_with(&src_abs) {
        return Err(Error::Serialization(
            "destination must not be inside the source package directory".to_owned(),
        ));
    }
    Ok(())
}

fn validate_layout_paths(layout: &PackageLayout) -> Result<()> {
    for entry in &layout.manifest.artifacts {
        if !is_safe_relative(&entry.path) {
            return Err(Error::Serialization(format!(
                "artifact path must be relative without '..': {}",
                entry.path
            )));
        }
        let path = join_under_root(&layout.root, &entry.path)?;
        let meta = fs::symlink_metadata(&path).map_err(|err| Error::Io {
            path: path.clone(),
            source: err,
        })?;
        if meta.file_type().is_symlink() {
            return Err(Error::Serialization(format!(
                "artifact path must not be a symlink: {}",
                entry.path
            )));
        }
        if !meta.is_file() {
            return Err(Error::Io {
                path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "artifact file missing"),
            });
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
    let archive_abs = archive
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.join(archive.file_name().unwrap_or_default()));
    let file = File::create(archive).map_err(|err| Error::Io {
        path: archive.to_path_buf(),
        source: err,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in WalkDir::new(root) {
        let entry = entry.map_err(|err| {
            Error::Serialization(format!("failed walking package directory: {err}"))
        })?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if let Some(archive_abs) = &archive_abs {
            if path == archive_abs {
                continue;
            }
        }
        let meta = fs::symlink_metadata(path).map_err(|err| Error::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        if meta.file_type().is_symlink() {
            return Err(Error::Serialization(format!(
                "refusing to pack symlink: {}",
                path.display()
            )));
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
    let dest_abs = dest.canonicalize().map_err(|err| Error::Io {
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
        let name_str = name.to_string_lossy().replace('\\', "/");
        let out = join_under_root(&dest_abs, &name_str)?;
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
            path: out.clone(),
            source: err,
        })?;
        let canon = out.canonicalize().map_err(|err| Error::Io {
            path: out.clone(),
            source: err,
        })?;
        if !canon.starts_with(&dest_abs) {
            let _ = fs::remove_file(&canon);
            return Err(Error::Serialization(
                "zip entry resolved outside destination".to_owned(),
            ));
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|err| Error::Io {
        path: dest.to_path_buf(),
        source: err,
    })?;
    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|err| {
            Error::Serialization(format!("failed walking source directory: {err}"))
        })?;
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .map_err(|_| Error::Serialization("failed to relativize source path".to_owned()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }
        let target = join_under_root(dest, &rel_str)?;
        if path.is_dir() {
            fs::create_dir_all(&target).map_err(|err| Error::Io {
                path: target,
                source: err,
            })?;
        } else {
            let meta = fs::symlink_metadata(path).map_err(|err| Error::Io {
                path: path.to_path_buf(),
                source: err,
            })?;
            if meta.file_type().is_symlink() {
                return Err(Error::Serialization(format!(
                    "refusing to copy symlink: {}",
                    path.display()
                )));
            }
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
