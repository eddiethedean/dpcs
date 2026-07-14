//! Registry client/server integration test.

use dpcs::{serve_listener, RegisteredArtifact, RegistryClient};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_serve_and_client_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let root_path = root.path().to_path_buf();
    let server = tokio::spawn(async move {
        serve_listener(listener, root_path, Some("secret".into()))
            .await
            .unwrap();
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let url = format!("http://{addr}");
    tokio::task::spawn_blocking(move || {
        let client = RegistryClient::new(&url).unwrap().with_token("secret");
        let registry = client.get_registry().unwrap();
        assert_eq!(registry.id, "local");

        let artifact = RegisteredArtifact {
            id: "demo".into(),
            artifact_type: "pipelineContract".into(),
            version: "0.1.0".into(),
            compatibility: None,
            publication_status: Some("published".into()),
            location: None,
            extensions: Default::default(),
        };
        let published = client
            .publish(
                "demo",
                &dpcs::PublishRequest {
                    artifact,
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
