//! Pipeline package format (`.dpcspkg`) — ROADMAP 0.10.

mod manifest;
mod resolve;
mod validate;

pub use manifest::{PackageArtifactEntry, PackageManifest};
pub use resolve::{artifact_entry, pack, resolve_artifact, unpack, write_manifest, PackageLayout};
pub use validate::validate_package;
