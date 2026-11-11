//! Plain HTTPS requests to OpenAI, separated from the request logic so
//! tests can replace the network with a fake.

use std::time::Duration;

use async_trait::async_trait;

use crate::responses::ResponsesError;

/// Longest a single request may take, including reading the response.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// The parts of an HTTP response the adapters use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) body: String,
}

/// Sends JSON requests authenticated with an API key.
#[async_trait]
pub(crate) trait HttpClient: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        api_key: &str,
        body: String,
    ) -> Result<HttpResponse, ResponsesError>;
}

/// The real client, over TLS.
pub(crate) struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub(crate) fn new() -> Self {
        // Building only fails if the TLS backend cannot start; the default
        // client reports the same problem on the first request instead.
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn post_json(
        &self,
        url: &str,
        api_key: &str,
        body: String,
    ) -> Result<HttpResponse, ResponsesError> {
        let response = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(request_error)?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(request_error)?;
        Ok(HttpResponse { status, body })
    }
}

fn request_error(error: reqwest::Error) -> ResponsesError {
    if error.is_timeout() {
        ResponsesError::Timeout
    } else {
        ResponsesError::ConnectionFailed {
            // The URL is dropped from the message; it adds nothing for the user.
            detail: error.without_url().to_string(),
        }
    }
}
