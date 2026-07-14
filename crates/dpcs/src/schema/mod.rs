//! JSON Schema and OpenAPI helpers (ROADMAP 0.10).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use schemars::schema::RootSchema;
use schemars::JsonSchema;
use serde_json::{json, Value};

use crate::binding::BindingBundle;
use crate::capabilities::{CapabilityProfile, CapabilityReport};
use crate::compatibility::CompatibilityReport;
use crate::conformance::{ConformanceClaim, ConformanceProfile};
use crate::diagnostics::{Diagnostic, ValidationReport};
use crate::error::{Error, Result};
use crate::model::{PipelineContract, RegisteredArtifact, Registry};
use crate::package::PackageManifest;
use crate::plan::PipelinePlan;
use crate::{DPCS_SPEC_VERSION, VERSION};

/// Generate the JSON Schema root for a type.
pub fn json_schema_for<T: JsonSchema>() -> RootSchema {
    schemars::schema_for!(T)
}

/// Serialize a schema to a JSON value.
pub fn schema_to_value(schema: &RootSchema) -> Result<Value> {
    serde_json::to_value(schema)
        .map_err(|err| Error::Serialization(format!("failed to serialize JSON Schema: {err}")))
}

/// All first-class document / report schemas keyed by type name.
pub fn document_schemas() -> Result<BTreeMap<String, Value>> {
    let mut map = BTreeMap::new();
    insert(
        &mut map,
        "PipelineContract",
        json_schema_for::<PipelineContract>(),
    )?;
    insert(&mut map, "PipelinePlan", json_schema_for::<PipelinePlan>())?;
    insert(
        &mut map,
        "CapabilityProfile",
        json_schema_for::<CapabilityProfile>(),
    )?;
    insert(
        &mut map,
        "CapabilityReport",
        json_schema_for::<CapabilityReport>(),
    )?;
    insert(&mut map, "Registry", json_schema_for::<Registry>())?;
    insert(
        &mut map,
        "RegisteredArtifact",
        json_schema_for::<RegisteredArtifact>(),
    )?;
    insert(
        &mut map,
        "ConformanceProfile",
        json_schema_for::<ConformanceProfile>(),
    )?;
    insert(
        &mut map,
        "ConformanceClaim",
        json_schema_for::<ConformanceClaim>(),
    )?;
    insert(
        &mut map,
        "ValidationReport",
        json_schema_for::<ValidationReport>(),
    )?;
    insert(&mut map, "Diagnostic", json_schema_for::<Diagnostic>())?;
    insert(
        &mut map,
        "CompatibilityReport",
        json_schema_for::<CompatibilityReport>(),
    )?;
    insert(
        &mut map,
        "BindingBundle",
        json_schema_for::<BindingBundle>(),
    )?;
    insert(
        &mut map,
        "PackageManifest",
        json_schema_for::<PackageManifest>(),
    )?;
    Ok(map)
}

fn insert(map: &mut BTreeMap<String, Value>, name: &str, schema: RootSchema) -> Result<()> {
    map.insert(name.to_owned(), schema_to_value(&schema)?);
    Ok(())
}

/// Write document JSON Schemas into `out_dir` as `<Name>.schema.json`.
pub fn write_document_schemas(out_dir: impl AsRef<Path>) -> Result<Vec<String>> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir).map_err(|err| Error::Io {
        path: out_dir.to_path_buf(),
        source: err,
    })?;
    let mut written = Vec::new();
    for (name, schema) in document_schemas()? {
        let path = out_dir.join(format!("{name}.schema.json"));
        let body = serde_json::to_string_pretty(&schema)
            .map_err(|err| Error::Serialization(format!("schema stringify failed: {err}")))?;
        fs::write(&path, body).map_err(|err| Error::Io {
            path: path.clone(),
            source: err,
        })?;
        written.push(path.display().to_string());
    }
    Ok(written)
}

/// Kind of OpenAPI document to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiKind {
    /// Document / COM component schemas for tooling.
    Documents,
    /// Reference registry HTTP API.
    Registry,
}

/// Build an OpenAPI 3.0 document.
pub fn openapi_document(kind: OpenApiKind) -> Result<Value> {
    match kind {
        OpenApiKind::Documents => openapi_documents(),
        OpenApiKind::Registry => openapi_registry(),
    }
}

fn openapi_documents() -> Result<Value> {
    let schemas = document_schemas()?;
    let mut components = serde_json::Map::new();
    for (name, schema) in schemas {
        components.insert(name, schema);
    }
    Ok(json!({
        "openapi": "3.0.3",
        "info": {
            "title": "DPCS Document Schemas",
            "version": VERSION,
            "description": format!(
                "Informative OpenAPI component schemas for DPCS {DPCS_SPEC_VERSION} documents and reports."
            )
        },
        "paths": {
            "/validate": {
                "post": {
                    "summary": "Validate a Pipeline Contract (example tooling path)",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/PipelineContract" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Validation report",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ValidationReport" }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": { "schemas": components }
    }))
}

fn openapi_registry() -> Result<Value> {
    let registry = schema_to_value(&json_schema_for::<Registry>())?;
    let artifact = schema_to_value(&json_schema_for::<RegisteredArtifact>())?;
    let diagnostic = schema_to_value(&json_schema_for::<Diagnostic>())?;
    let report = schema_to_value(&json_schema_for::<ValidationReport>())?;
    Ok(json!({
        "openapi": "3.0.3",
        "info": {
            "title": "DPCS Reference Registry API",
            "version": "1.0.0",
            "description": "Implementation-defined reference registry HTTP API for DPCS 0.10 (see docs/REGISTRY_API.md)."
        },
        "paths": {
            "/v1/health": {
                "get": {
                    "summary": "Liveness",
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/v1/registry": {
                "get": {
                    "summary": "Get full registry document",
                    "responses": {
                        "200": {
                            "description": "Registry document",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/Registry" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/artifacts": {
                "get": {
                    "summary": "Discover artifacts",
                    "parameters": [
                        { "name": "type", "in": "query", "schema": { "type": "string" } },
                        { "name": "status", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Artifact list",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": { "$ref": "#/components/schemas/RegisteredArtifact" }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/artifacts/{id}": {
                "get": {
                    "summary": "Lookup artifact metadata",
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "version", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Artifact metadata",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/RegisteredArtifact" }
                                }
                            }
                        },
                        "404": { "description": "Not found" }
                    }
                },
                "put": {
                    "summary": "Publish or update artifact",
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "$ref": "#/components/schemas/PublishRequest" }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Updated artifact",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/RegisteredArtifact" }
                                }
                            }
                        },
                        "400": {
                            "description": "Validation failure",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/ValidationReport" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/artifacts/{id}/content": {
                "get": {
                    "summary": "Fetch artifact payload",
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } },
                        { "name": "version", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": { "description": "Raw artifact content" },
                        "404": { "description": "Not found" }
                    }
                }
            },
            "/v1/artifacts/{id}/deprecate": {
                "post": {
                    "summary": "Deprecate artifact",
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Updated artifact",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/RegisteredArtifact" }
                                }
                            }
                        }
                    }
                }
            },
            "/v1/artifacts/{id}/retire": {
                "post": {
                    "summary": "Retire artifact",
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": {
                        "200": {
                            "description": "Updated artifact",
                            "content": {
                                "application/json": {
                                    "schema": { "$ref": "#/components/schemas/RegisteredArtifact" }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "Registry": registry,
                "RegisteredArtifact": artifact,
                "Diagnostic": diagnostic,
                "ValidationReport": report,
                "PublishRequest": {
                    "type": "object",
                    "required": ["artifact"],
                    "properties": {
                        "artifact": { "$ref": "#/components/schemas/RegisteredArtifact" },
                        "content": { "type": "string", "description": "Optional artifact payload" },
                        "contentEncoding": { "type": "string", "enum": ["utf-8", "base64"] }
                    }
                }
            },
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer"
                }
            }
        }
    }))
}

/// Write OpenAPI documents into `out_dir`.
pub fn write_openapi_documents(out_dir: impl AsRef<Path>) -> Result<Vec<String>> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir).map_err(|err| Error::Io {
        path: out_dir.to_path_buf(),
        source: err,
    })?;
    let mut written = Vec::new();
    for (name, kind) in [
        ("documents.openapi.json", OpenApiKind::Documents),
        ("registry.openapi.json", OpenApiKind::Registry),
    ] {
        let path = out_dir.join(name);
        let doc = openapi_document(kind)?;
        let body = serde_json::to_string_pretty(&doc)
            .map_err(|err| Error::Serialization(format!("openapi stringify failed: {err}")))?;
        fs::write(&path, body).map_err(|err| Error::Io {
            path: path.clone(),
            source: err,
        })?;
        written.push(path.display().to_string());
    }
    Ok(written)
}
