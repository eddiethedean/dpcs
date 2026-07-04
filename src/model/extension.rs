//! Extension helpers.

/// Returns `true` when `key` is a reserved DPCS root field name.
pub fn is_reserved_root_field(key: &str) -> bool {
    matches!(
        key,
        "dpcsVersion"
            | "id"
            | "name"
            | "version"
            | "metadata"
            | "interface"
            | "graph"
            | "steps"
            | "contractReferences"
            | "dataFlow"
            | "controlFlow"
            | "execution"
            | "scheduling"
            | "qualityGates"
            | "failureSemantics"
            | "lineage"
            | "compatibility"
    )
}
