//! Registry client/server integration tests.

use dpcs::{serve_listener, PublishRequest, RegisteredArtifact, RegistryCache, RegistryClient};
use std::time::Duration;

async fn spawn_server(
    root: std::path::PathBuf,
    token: Option<String>,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        serve_listener(listener, root, token).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    (format!("http://{addr}"), server)
}

fn sample_artifact(id: &str, version: &str) -> RegisteredArtifact {
    RegisteredArtifact {
        id: id.into(),
        artifact_type: "pipelineContract".into(),
        version: version.into(),
        compatibility: None,
        publication_status: Some("published".into()),
        location: None,
        extensions: Default::default(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_serve_and_client_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        let registry = client.get_registry().unwrap();
        assert_eq!(registry.id, "local");

        let published = client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "0.1.0"),
                    content: Some("id: demo\n".into()),
                    content_encoding: Some("utf-8".into()),
                },
            )
            .unwrap();
        assert_eq!(published.id, "demo");

        let mut client = RegistryClient::new(&url).unwrap().with_token("secret");
        let looked = client.lookup("demo", Some("0.1.0")).unwrap();
        assert_eq!(looked.version, "0.1.0");
        let content = client.fetch_content("demo", Some("0.1.0")).unwrap();
        assert!(content.contains("demo"));
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_rejects_unauthenticated_mutates() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap();
        let err = client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "0.1.0"),
                    content: None,
                    content_encoding: None,
                },
            )
            .unwrap_err();
        match err {
            dpcs::RegistryClientError::Http { status, .. } => assert_eq!(status, 401),
            other => panic!("expected HTTP 401, got {other}"),
        }
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_rejects_path_traversal_location() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        let mut artifact = sample_artifact("demo", "0.1.0");
        artifact.location = Some("../escape.yaml".into());
        let err = client
            .publish(
                "demo",
                &PublishRequest {
                    artifact,
                    content: Some("x".into()),
                    content_encoding: Some("utf-8".into()),
                },
            )
            .unwrap_err();
        match err {
            dpcs::RegistryClientError::Http { status, .. } => {
                assert!(status == 400 || status == 404, "status={status}");
            }
            other => panic!("expected HTTP error, got {other}"),
        }
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_rejects_non_utf8_content_encoding() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        let err = client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "0.1.0"),
                    content: Some("x".into()),
                    content_encoding: Some("base64".into()),
                },
            )
            .unwrap_err();
        match err {
            dpcs::RegistryClientError::Http { status, .. } => assert_eq!(status, 400),
            other => panic!("expected HTTP 400, got {other}"),
        }
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_status_filter_is_case_insensitive() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "0.1.0"),
                    content: Some("id: demo\n".into()),
                    content_encoding: Some("utf-8".into()),
                },
            )
            .unwrap();
        let listed = client.list(None, Some("PUBLISHED")).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "demo");
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_deprecate_version_targets_specific_row() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        for version in ["0.1.0", "0.2.0"] {
            client
                .publish(
                    "demo",
                    &PublishRequest {
                        artifact: sample_artifact("demo", version),
                        content: Some(format!("version: {version}\n")),
                        content_encoding: Some("utf-8".into()),
                    },
                )
                .unwrap();
        }
        let deprecated = client.deprecate("demo", Some("0.1.0")).unwrap();
        assert_eq!(deprecated.version, "0.1.0");
        assert_eq!(deprecated.publication_status.as_deref(), Some("deprecated"));
        let mut client = RegistryClient::new(&url).unwrap().with_token("secret");
        let other = client.lookup("demo", Some("0.2.0")).unwrap();
        assert_eq!(other.publication_status.as_deref(), Some("published"));
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_client_encodes_special_query_chars() {
    let root = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;

    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "1.0.0+build.1"),
                    content: Some("id: demo\n".into()),
                    content_encoding: Some("utf-8".into()),
                },
            )
            .unwrap();
        let mut client = RegistryClient::new(&url).unwrap().with_token("secret");
        let looked = client.lookup("demo", Some("1.0.0+build.1")).unwrap();
        assert_eq!(looked.version, "1.0.0+build.1");
    })
    .await
    .unwrap();

    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_disk_cache_serves_lookup_hits() {
    let root = tempfile::tempdir().unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let (url, server) = spawn_server(root.path().to_path_buf(), Some("secret".into())).await;
    let cache_path = cache_dir.path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut client = RegistryClient::new(&url)
            .unwrap()
            .with_token("secret")
            .with_cache(RegistryCache::disk(&cache_path).unwrap());
        client
            .publish(
                "demo",
                &PublishRequest {
                    artifact: sample_artifact("demo", "0.1.0"),
                    content: Some("id: demo\n".into()),
                    content_encoding: Some("utf-8".into()),
                },
            )
            .unwrap();
        let first = client.lookup("demo", Some("0.1.0")).unwrap();
        assert_eq!(first.id, "demo");
        // Same base URL namespace: second client answers from disk without re-publish.
        let mut cached = RegistryClient::new(&url)
            .unwrap()
            .with_cache(RegistryCache::disk(&cache_path).unwrap());
        let hit = cached.lookup("demo", Some("0.1.0")).unwrap();
        assert_eq!(hit.version, "0.1.0");
        assert!(cache_path.read_dir().unwrap().next().is_some());
    })
    .await
    .unwrap();

    server.abort();
}
