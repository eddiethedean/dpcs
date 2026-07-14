//! Schema helper smoke tests.

use dpcs::{document_schemas, openapi_document, OpenApiKind};

#[test]
fn document_schemas_include_contract() {
    let schemas = document_schemas().unwrap();
    assert!(schemas.contains_key("PipelineContract"));
    assert!(schemas.contains_key("PackageManifest"));
}

#[test]
fn openapi_kinds_emit_openapi_version() {
    for kind in [OpenApiKind::Documents, OpenApiKind::Registry] {
        let doc = openapi_document(kind).unwrap();
        assert_eq!(doc["openapi"], "3.0.3");
    }
}
