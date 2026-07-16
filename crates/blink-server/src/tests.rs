use std::fs;

use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::DevServer;

async fn get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

#[tokio::test]
async fn serves_index_html_at_root() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("index.html"), "<h1>hi</h1>").unwrap();

    let server = DevServer::new(dir.path().to_path_buf(), 0);
    let listener = server.bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { server.serve(listener).await });

    let response = get(port, "/").await;

    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("<h1>hi</h1>"));
}

#[tokio::test]
async fn falls_back_to_placeholder_when_no_index() {
    let dir = TempDir::new().unwrap();

    let server = DevServer::new(dir.path().to_path_buf(), 0);
    let listener = server.bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { server.serve(listener).await });

    let response = get(port, "/").await;

    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("Blink"));
}

#[tokio::test]
async fn returns_404_for_missing_file() {
    let dir = TempDir::new().unwrap();

    let server = DevServer::new(dir.path().to_path_buf(), 0);
    let listener = server.bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { server.serve(listener).await });

    let response = get(port, "/missing.js").await;

    assert!(response.starts_with("HTTP/1.1 404 Not Found"));
}
