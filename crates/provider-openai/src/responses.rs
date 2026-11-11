//! Structured requests to OpenAI's Responses API, shared by the translator
//! and the reply generator.
//!
//! Written against the Responses API reference as published in November
//! 2026. A request carries fixed instructions, one JSON object as the user
//! message, and a JSON schema the reply must match.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};

use crate::http::{HttpClient, HttpResponse, ReqwestClient};

const ENDPOINT: &str = "https://api.openai.com/v1/responses";
/// OpenAI's model for focused, high-volume tasks.
const MODEL: &str = "gpt-6-luna";
/// Upper bound on the reply, which also covers the model's reasoning.
const MAX_OUTPUT_TOKENS: u32 = 4_000;
/// A request is tried this many times in total when the failure looks temporary.
const MAX_ATTEMPTS: u32 = 2;
const RETRY_DELAY: Duration = Duration::from_millis(500);

/// Why a Responses API request produced no usable reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResponsesError {
    AuthenticationFailed,
    RateLimited,
    ConnectionFailed { detail: String },
    Timeout,
    MalformedResponse,
    Refused,
    Provider { detail: String },
}

impl ResponsesError {
    /// Whether sending the same request again is likely to help.
    fn is_transient(&self) -> bool {
        matches!(self, Self::ConnectionFailed { .. } | Self::Timeout)
    }
}

/// One kind of structured request: what the model is told, and the shape of its reply.
pub(crate) struct Task {
    pub(crate) instructions: String,
    /// Name of the reply schema, as OpenAI requires one.
    pub(crate) schema_name: &'static str,
    /// JSON schema of the reply.
    pub(crate) schema: Value,
}

/// Sends structured requests authenticated with the user's API key.
pub(crate) struct ResponsesClient {
    api_key: String,
    http: Arc<dyn HttpClient>,
}

impl ResponsesClient {
    pub(crate) fn new(api_key: String) -> Self {
        Self::with_http(api_key, Arc::new(ReqwestClient::new()))
    }

    pub(crate) fn with_http(api_key: String, http: Arc<dyn HttpClient>) -> Self {
        Self { api_key, http }
    }

    /// Runs `task` on `input` and returns the model's reply as parsed JSON.
    /// A failure that looks temporary is tried once more.
    pub(crate) async fn run(&self, task: &Task, input: &Value) -> Result<Value, ResponsesError> {
        let body = request_body(task, input);
        let mut attempt = 1;
        loop {
            let response = self
                .http
                .post_json(ENDPOINT, &self.api_key, body.clone())
                .await;
            match response.and_then(|response| read_response(&response)) {
                Err(error) if error.is_transient() && attempt < MAX_ATTEMPTS => {
                    attempt += 1;
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                result => return result,
            }
        }
    }
}

fn request_body(task: &Task, input: &Value) -> String {
    json!({
        "model": MODEL,
        // Do not keep the request or the reply on OpenAI's side.
        "store": false,
        "reasoning": { "effort": "low" },
        "max_output_tokens": MAX_OUTPUT_TOKENS,
        "instructions": task.instructions,
        // The input is one JSON document, so text inside it stays data.
        "input": input.to_string(),
        "text": {
            "format": {
                "type": "json_schema",
                "name": task.schema_name,
                "strict": true,
                "schema": task.schema,
            }
        },
    })
    .to_string()
}

/// Extracts the model's reply from an HTTP response, or the reason there is none.
fn read_response(response: &HttpResponse) -> Result<Value, ResponsesError> {
    let body: Value = serde_json::from_str(&response.body).unwrap_or(Value::Null);
    match response.status {
        200..=299 => read_reply(&body),
        401 | 403 => Err(ResponsesError::AuthenticationFailed),
        429 => Err(ResponsesError::RateLimited),
        // Server-side trouble is worth one more try.
        500..=599 => Err(ResponsesError::ConnectionFailed {
            detail: format!("the server answered {}", response.status),
        }),
        _ => Err(ResponsesError::Provider {
            detail: body["error"]["message"]
                .as_str()
                .unwrap_or("the request was rejected")
                .to_owned(),
        }),
    }
}

/// Finds the model's text in a successful response and parses it as JSON.
///
/// The response holds a list of output items; the text is in the content of
/// the item of type `message`, which is not necessarily the first one.
fn read_reply(body: &Value) -> Result<Value, ResponsesError> {
    let contents = body["output"]
        .as_array()
        .ok_or(ResponsesError::MalformedResponse)?
        .iter()
        .filter(|item| item["type"] == "message")
        .filter_map(|item| item["content"].as_array())
        .flatten();

    for content in contents {
        if content["type"] == "refusal" {
            return Err(ResponsesError::Refused);
        }
        if content["type"] == "output_text" {
            let text = content["text"]
                .as_str()
                .ok_or(ResponsesError::MalformedResponse)?;
            return serde_json::from_str(text).map_err(|_| ResponsesError::MalformedResponse);
        }
    }
    // No text at all: the reply was cut off or filtered before any was produced.
    Err(ResponsesError::MalformedResponse)
}

/// A fake HTTP client and response builders for the adapters' tests.
#[cfg(test)]
pub(crate) mod testing {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::*;

    /// Returns prepared responses in order and records what was sent.
    pub(crate) struct FakeHttp {
        responses: Mutex<Vec<Result<HttpResponse, ResponsesError>>>,
        requests: Mutex<Vec<(String, String, Value)>>,
    }

    impl FakeHttp {
        pub(crate) fn new(responses: Vec<Result<HttpResponse, ResponsesError>>) -> Arc<Self> {
            Arc::new(Self {
                responses: Mutex::new(responses),
                requests: Mutex::new(Vec::new()),
            })
        }

        pub(crate) fn request_count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }

        /// The URL, API key and JSON body of the first request.
        pub(crate) fn first_request(&self) -> (String, String, Value) {
            self.requests.lock().unwrap()[0].clone()
        }

        /// The JSON document the first request passed to the model as its input.
        pub(crate) fn first_input(&self) -> Value {
            let (_, _, body) = self.first_request();
            serde_json::from_str(body["input"].as_str().unwrap()).unwrap()
        }
    }

    #[async_trait]
    impl HttpClient for FakeHttp {
        async fn post_json(
            &self,
            url: &str,
            api_key: &str,
            body: String,
        ) -> Result<HttpResponse, ResponsesError> {
            let body = serde_json::from_str(&body).expect("the request body must be JSON");
            self.requests
                .lock()
                .unwrap()
                .push((url.to_owned(), api_key.to_owned(), body));
            self.responses.lock().unwrap().remove(0)
        }
    }

    pub(crate) fn status(status: u16, body: &Value) -> Result<HttpResponse, ResponsesError> {
        Ok(HttpResponse {
            status,
            body: body.to_string(),
        })
    }

    /// A successful response whose message carries `reply` as its JSON text.
    pub(crate) fn replied(reply: &Value) -> Result<HttpResponse, ResponsesError> {
        status(
            200,
            &json!({
                "status": "completed",
                "output": [
                    { "type": "reasoning", "summary": [] },
                    {
                        "type": "message",
                        "role": "assistant",
                        "content": [{ "type": "output_text", "text": reply.to_string(), "annotations": [] }],
                    },
                ],
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{FakeHttp, replied, status};
    use super::*;

    fn task() -> Task {
        Task {
            instructions: "Do the task.".to_owned(),
            schema_name: "answer",
            schema: json!({ "type": "object" }),
        }
    }

    async fn run(http: &Arc<FakeHttp>) -> Result<Value, ResponsesError> {
        ResponsesClient::with_http("test-key".to_owned(), http.clone())
            .run(&task(), &json!({ "text": "hello" }))
            .await
    }

    #[tokio::test]
    async fn returns_the_json_from_the_message_item() {
        let http = FakeHttp::new(vec![replied(&json!({ "answer": 42 }))]);
        assert_eq!(run(&http).await, Ok(json!({ "answer": 42 })));
    }

    #[tokio::test]
    async fn sends_an_unstored_structured_request_with_the_key() {
        let http = FakeHttp::new(vec![replied(&json!({}))]);
        run(&http).await.unwrap();

        let (url, api_key, body) = http.first_request();
        assert_eq!(url, "https://api.openai.com/v1/responses");
        assert_eq!(api_key, "test-key");
        assert_eq!(body["model"], "gpt-6-luna");
        assert_eq!(body["store"], false);
        assert_eq!(body["instructions"], "Do the task.");
        assert_eq!(body["text"]["format"]["type"], "json_schema");
        assert_eq!(body["text"]["format"]["name"], "answer");
        assert_eq!(body["text"]["format"]["strict"], true);
        assert_eq!(http.first_input(), json!({ "text": "hello" }));
    }

    #[tokio::test]
    async fn a_refusal_is_reported_as_such() {
        let body = json!({ "output": [{ "type": "message", "content": [{ "type": "refusal", "refusal": "no" }] }] });
        let http = FakeHttp::new(vec![status(200, &body)]);
        assert_eq!(run(&http).await, Err(ResponsesError::Refused));
    }

    #[tokio::test]
    async fn a_reply_without_usable_text_is_malformed() {
        let not_json = json!({ "output": [{ "type": "message", "content": [{ "type": "output_text", "text": "not json" }] }] });
        for body in [json!({}), json!({ "output": [] }), not_json] {
            let http = FakeHttp::new(vec![status(200, &body)]);
            assert_eq!(run(&http).await, Err(ResponsesError::MalformedResponse));
        }
    }

    #[tokio::test]
    async fn a_rejected_key_and_a_rate_limit_are_not_retried() {
        for (code, expected) in [
            (401, ResponsesError::AuthenticationFailed),
            (429, ResponsesError::RateLimited),
        ] {
            let http = FakeHttp::new(vec![status(code, &json!({ "error": { "message": "x" } }))]);
            assert_eq!(run(&http).await, Err(expected));
            assert_eq!(http.request_count(), 1);
        }
    }

    #[tokio::test]
    async fn a_bad_request_reports_the_provider_message() {
        let http = FakeHttp::new(vec![status(
            400,
            &json!({ "error": { "message": "unknown model" } }),
        )]);
        assert_eq!(
            run(&http).await,
            Err(ResponsesError::Provider {
                detail: "unknown model".to_owned()
            })
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_server_error_is_tried_once_more() {
        let http = FakeHttp::new(vec![
            status(503, &json!({})),
            replied(&json!({ "ok": true })),
        ]);
        assert_eq!(run(&http).await, Ok(json!({ "ok": true })));
        assert_eq!(http.request_count(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_timeout_that_repeats_is_reported() {
        let http = FakeHttp::new(vec![
            Err(ResponsesError::Timeout),
            Err(ResponsesError::Timeout),
        ]);
        assert_eq!(run(&http).await, Err(ResponsesError::Timeout));
        assert_eq!(http.request_count(), 2);
    }
}
