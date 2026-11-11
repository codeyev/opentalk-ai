//! A small HTTP server on a loopback port, standing in for a provider in
//! tests. The client under test makes real HTTP requests to it, so the
//! tests cover the real request and response handling without leaving the
//! machine.

use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// A canned response for one path.
pub struct Route {
    path: &'static str,
    status: u16,
    body: String,
}

impl Route {
    pub fn new(path: &'static str, status: u16, body: impl Into<String>) -> Self {
        Self {
            path,
            status,
            body: body.into(),
        }
    }
}

/// A request the server received.
struct Request {
    path: String,
    /// The request line and headers, as sent.
    head: String,
    body: String,
}

pub struct MockServer {
    base_url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    task: JoinHandle<()>,
}

impl MockServer {
    /// Starts a server on a free loopback port that answers `routes`, and
    /// 404 for anything else.
    pub async fn start(routes: Vec<Route>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
        let address = listener.local_addr().expect("a local address");
        let base_url = format!("http://{address}");
        let requests = Arc::new(Mutex::new(Vec::new()));

        let recorded = Arc::clone(&requests);
        let routes = Arc::new(routes);
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let _ = answer(stream, &routes, &recorded).await;
            }
        });
        Self {
            base_url,
            requests,
            task,
        }
    }

    /// The server's address, such as `http://127.0.0.1:49152`.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Every path that was requested, in order.
    pub fn requested_paths(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|request| request.path.clone())
            .collect()
    }

    /// The JSON bodies of every request to `path`, in order.
    #[must_use]
    pub fn requests_to(&self, path: &str) -> Vec<Value> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|request| request.path == path)
            .map(|request| {
                serde_json::from_str(&request.body).expect("the request body must be JSON")
            })
            .collect()
    }

    /// The JSON body of the first request to `path`.
    ///
    /// # Panics
    ///
    /// Panics if no request was made to `path`.
    #[must_use]
    pub fn request_to(&self, path: &str) -> Value {
        self.requests_to(path)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no request was made to {path}"))
    }

    /// The value of a header in the first request to `path`, if it was sent.
    #[must_use]
    pub fn header_of(&self, path: &str, name: &str) -> Option<String> {
        let requests = self.requests.lock().unwrap();
        let request = requests.iter().find(|request| request.path == path)?;
        request
            .head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim().to_owned())
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Reads one HTTP request and writes the matching canned response.
async fn answer(
    mut stream: TcpStream,
    routes: &[Route],
    requests: &Mutex<Vec<Request>>,
) -> std::io::Result<()> {
    let mut received = Vec::new();
    let mut buffer = [0_u8; 4096];
    let (head, body_start) = loop {
        let count = stream.read(&mut buffer).await?;
        if count == 0 {
            return Ok(());
        }
        received.extend_from_slice(&buffer[..count]);
        if let Some(end) = received.windows(4).position(|window| window == b"\r\n\r\n") {
            break (
                String::from_utf8_lossy(&received[..end]).into_owned(),
                end + 4,
            );
        }
    };

    let content_length: usize = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse().ok())
        .unwrap_or(0);
    while received.len() < body_start + content_length {
        let count = stream.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        received.extend_from_slice(&buffer[..count]);
    }

    let path = head
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    let body = String::from_utf8_lossy(&received[body_start..]).into_owned();
    requests.lock().unwrap().push(Request {
        path: path.clone(),
        head,
        body,
    });

    let (status, response_body) = routes
        .iter()
        .find(|route| route.path == path)
        .map_or((404, r#"{"error":"not found"}"#), |route| {
            (route.status, route.body.as_str())
        });
    let response = format!(
        "HTTP/1.1 {status} Status\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
        response_body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await
}
