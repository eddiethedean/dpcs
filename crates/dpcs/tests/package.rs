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
