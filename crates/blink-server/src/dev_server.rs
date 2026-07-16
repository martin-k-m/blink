use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::error::{Result, ServerError};

const PLACEHOLDER_PAGE: &str = include_str!("placeholder.html");

/// A minimal static file server used by `blink run` for local development.
/// It serves files directly from the project root, falling back to a
/// placeholder page when no `index.html` is present.
pub struct DevServer {
    root: PathBuf,
    port: u16,
}

impl DevServer {
    pub fn new(root: PathBuf, port: u16) -> Self {
        Self { root, port }
    }

    pub fn local_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Bind the listening socket without yet accepting connections, so
    /// callers can report readiness only once the port is actually open.
    pub async fn bind(&self) -> Result<TcpListener> {
        let addr = format!("127.0.0.1:{}", self.port);
        TcpListener::bind(&addr)
            .await
            .map_err(|source| ServerError::Bind { addr, source })
    }

    /// Accept and serve connections forever.
    pub async fn serve(&self, listener: TcpListener) {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            let root = self.root.clone();
            tokio::spawn(async move {
                let _ = handle_connection(stream, &root).await;
            });
        }
    }
}

async fn handle_connection(mut stream: TcpStream, root: &Path) -> std::io::Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let (status, content_type, body) = resolve(root, path);

    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(&body);

    stream.write_all(&response).await?;
    stream.flush().await
}

fn resolve(root: &Path, request_path: &str) -> (&'static str, &'static str, Vec<u8>) {
    let clean_path = request_path.split(['?', '#']).next().unwrap_or("/");
    let relative = clean_path.trim_start_matches('/');

    let candidate = if relative.is_empty() {
        root.join("index.html")
    } else {
        root.join(relative)
    };

    if candidate.is_file() {
        if let Ok(bytes) = std::fs::read(&candidate) {
            let content_type = content_type_for(&candidate);
            return ("200 OK", content_type, bytes);
        }
    }

    if relative.is_empty() {
        return (
            "200 OK",
            "text/html; charset=utf-8",
            PLACEHOLDER_PAGE.as_bytes().to_vec(),
        );
    }

    (
        "404 Not Found",
        "text/plain; charset=utf-8",
        b"404 Not Found".to_vec(),
    )
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
