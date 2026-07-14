//! Package format tests.

use dpcs::{
    artifact_entry, pack, resolve_artifact, unpack, validate_package, write_manifest,
    PackageManifest,
};
use std::fs;

#[test]
fn pack_unpack_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("demo.dpcspkg");
    fs::create_dir_all(pkg.join("artifacts")).unwrap();
    let contract = include_str!("../../../examples/minimal.dpcs.yaml");
    fs::write(pkg.join("artifacts/minimal.dpcs.yaml"), contract).unwrap();
    let mut manifest = PackageManifest {
        id: "demo".into(),
        version: "0.1.0".into(),
        ..Default::default()
    };
    manifest.artifacts.push(artifact_entry(
        "minimal",
        "pipelineContract",
        "0.1.0",
        "artifacts/minimal.dpcs.yaml",
    ));
    write_manifest(&pkg, &manifest).unwrap();
    let report = validate_package(&pkg);
    assert!(report.is_valid(), "{:?}", report.diagnostics);
    let archive = root.path().join("demo.dpcspkg.zip");
    pack(&pkg, Some(&archive)).unwrap();
    let dest = root.path().join("out");
    let layout = unpack(&archive, &dest).unwrap();
    assert_eq!(layout.manifest.id, "demo");
    let resolved = resolve_artifact(&dest, "minimal", Some("0.1.0")).unwrap();
    assert!(resolved.is_file());
}

#[test]
fn example_package_validates() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
        .join("examples/packages/minimal.dpcspkg");
    let report = validate_package(&path);
    assert!(report.is_valid(), "{:?}", report.diagnostics);
}

#[test]
fn rejects_path_escape_in_manifest() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("bad.dpcspkg");
    fs::create_dir_all(&pkg).unwrap();
    let mut manifest = PackageManifest {
        id: "bad".into(),
        version: "0.1.0".into(),
        ..Default::default()
    };
    manifest.artifacts.push(artifact_entry(
        "escape",
        "pipelineContract",
        "0.1.0",
        "../outside.yaml",
    ));
    write_manifest(&pkg, &manifest).unwrap();
    let report = validate_package(&pkg);
    assert!(!report.is_valid());
}

#[test]
fn pack_rejects_archive_under_package_root() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("demo.dpcspkg");
    fs::create_dir_all(pkg.join("artifacts")).unwrap();
    let contract = include_str!("../../../examples/minimal.dpcs.yaml");
    fs::write(pkg.join("artifacts/minimal.dpcs.yaml"), contract).unwrap();
    let mut manifest = PackageManifest {
        id: "demo".into(),
        version: "0.1.0".into(),
        ..Default::default()
    };
    manifest.artifacts.push(artifact_entry(
        "minimal",
        "pipelineContract",
        "0.1.0",
        "artifacts/minimal.dpcs.yaml",
    ));
    write_manifest(&pkg, &manifest).unwrap();
    let nested = pkg.join("nested/out.zip");
    let err = pack(&pkg, Some(&nested)).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("outside the package root"),
        "unexpected error: {msg}"
    );
}

#[test]
fn resolve_without_version_uses_last_match() {
    let root = tempfile::tempdir().unwrap();
    let pkg = root.path().join("demo.dpcspkg");
    fs::create_dir_all(pkg.join("artifacts")).unwrap();
    let contract = include_str!("../../../examples/minimal.dpcs.yaml");
    fs::write(pkg.join("artifacts/a.yaml"), contract).unwrap();
    fs::write(pkg.join("artifacts/b.yaml"), contract).unwrap();
    let mut manifest = PackageManifest {
        id: "demo".into(),
        version: "0.1.0".into(),
        ..Default::default()
    };
    manifest.artifacts.push(artifact_entry(
        "dup",
        "pipelineContract",
        "0.1.0",
        "artifacts/a.yaml",
    ));
    manifest.artifacts.push(artifact_entry(
        "dup",
        "pipelineContract",
        "0.2.0",
        "artifacts/b.yaml",
    ));
    write_manifest(&pkg, &manifest).unwrap();
    let resolved = resolve_artifact(&pkg, "dup", None).unwrap();
    assert!(resolved.ends_with("b.yaml"));
}
