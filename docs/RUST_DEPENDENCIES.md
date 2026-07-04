# Recommended Rust Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
thiserror = "2"
miette = { version = "7", features = ["fancy"] }
semver = { version = "1", features = ["serde"] }
indexmap = { version = "2", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
uuid = { version = "1", features = ["serde", "v4"] }
petgraph = "0.6"
url = { version = "2", features = ["serde"] }
```

Use `petgraph` for graph validation if useful, but keep the public model independent of `petgraph`.
