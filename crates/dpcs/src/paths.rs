//! Shared path containment helpers for package and registry FS operations.

use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

/// Returns `true` when `relative` is a safe relative path (no absolute / `..` / NUL).
pub fn is_safe_relative(relative: &str) -> bool {
    if relative.is_empty() || relative.contains('\0') {
        return false;
    }
    let path = Path::new(relative);
    if path.is_absolute() {
        return false;
    }
    path.components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

/// Join `relative` under `root` and ensure the result cannot escape `root`.
///
/// Does not require the final path to exist. Rejects absolute paths, `..`, and
/// NUL bytes. When `root` exists, also refuses to traverse through symlinks
/// that leave `root`.
pub fn join_under_root(root: impl AsRef<Path>, relative: &str) -> Result<PathBuf> {
    let root = root.as_ref();
    if !is_safe_relative(relative) {
        return Err(Error::Serialization(format!(
            "unsafe relative path `{relative}`"
        )));
    }

    let root_abs = if root.exists() {
        root.canonicalize().map_err(|err| Error::Io {
            path: root.to_path_buf(),
            source: err,
        })?
    } else {
        root.to_path_buf()
    };

    let mut current = root_abs.clone();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(seg) => {
                current.push(seg);
                if current.exists() {
                    let meta = std::fs::symlink_metadata(&current).map_err(|err| Error::Io {
                        path: current.clone(),
                        source: err,
                    })?;
                    if meta.file_type().is_symlink() {
                        let target = current.canonicalize().map_err(|err| Error::Io {
                            path: current.clone(),
                            source: err,
                        })?;
                        if !target.starts_with(&root_abs) {
                            return Err(Error::Serialization(format!(
                                "path escapes package/registry root via symlink: {relative}"
                            )));
                        }
                    }
                }
            }
            Component::CurDir => {}
            _ => {
                return Err(Error::Serialization(format!(
                    "unsafe path component in `{relative}`"
                )));
            }
        }
    }

    if current.exists() {
        let canon = current.canonicalize().map_err(|err| Error::Io {
            path: current.clone(),
            source: err,
        })?;
        if !canon.starts_with(&root_abs) {
            return Err(Error::Serialization(format!(
                "resolved path escapes root: {relative}"
            )));
        }
        return Ok(canon);
    }

    if let Some(parent) = current.parent() {
        if parent.exists() {
            let parent_canon = parent.canonicalize().map_err(|err| Error::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
            if !parent_canon.starts_with(&root_abs) {
                return Err(Error::Serialization(format!(
                    "parent path escapes root: {relative}"
                )));
            }
            return Ok(parent_canon.join(current.file_name().unwrap_or_default()));
        }
    }

    Ok(current)
}

/// Returns true when `id` is safe to embed in a filesystem path segment.
///
/// Allows `+` so SemVer build metadata (for example `1.2.3+build`) can be used
/// in registry artifact ids/versions and content filenames.
#[cfg(feature = "registry-server")]
pub fn is_safe_path_segment(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
}
