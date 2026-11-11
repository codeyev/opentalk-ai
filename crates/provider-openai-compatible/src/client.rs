//! Structured requests to a Chat Completions endpoint.
//!
//! Written against the Chat Completions format as documented by OpenAI and
//! by OpenRouter in November 2026.

use std::time::Duration;

use serde_json::{Value, json};

use crate::endpoint::Endpoint;

/// Longest a single request may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Upper bound on the reply, which for reasoning models includes the reasoning.
const MAX_TOKENS: u32 = 4_000;

/// Why a request produced no usable reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChatError {
    AuthenticationFailed,
    /// Rate limited, or the account is out of credit.
    RateLimited,
    /// The service does not know the model that was asked for.
    ModelUnavailable,
    ConnectionFailed {
        detail: String,
    },
    Timeout,
    MalformedResponse,
    Service {
        detail: String,
    },
}

/// How strictly the service is asked to shape its reply.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ReplyFormat {
    /// The reply must match the task's JSON schema.
    Schema,
    /// The reply must be JSON; the schema is only described in the prompt.
    AnyJson,
}

/// A structured task: instructions, and the shape of the reply.
pub(crate) struct Task<'a> {
    pub(crate) instructions: &'a str,
    pub(crate) schema_name: &'a str,
    pub(crate) schema: &'a Value,
}

/// A connection to one model on an OpenAI-compatible service.
pub struct ChatClient {
    endpoint: Endpoint,
    /// `None` for services that need no key, such as a local server.
    api_key: Option<String>,
    model: String,
    http: reqwest::Client,
}

impl ChatClient {
    #[must_use]
    pub fn new(endpoint: Endpoint, api_key: Option<String>, model: String) -> Self {
        // Building only fails if the TLS backend cannot start; the default
        // client reports the same problem on the first request instead.
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            endpoint,
            api_key,
            model,
            http,
        }
    }

    /// Runs `task` on `input` and returns the model's reply as JSON.
    ///
    /// The service is first asked to enforce the reply schema. Services and
    /// models that cannot do that reject the request; it is then sent once
    /// more asking only for JSON, with the schema described in the prompt.
    pub(crate) async fn run(&self, task: &Task<'_>, input: &Value) -> Result<Value, ChatError> {
        match self.request(task, input, ReplyFormat::Schema).await {
            Err(ChatError::Service { .. }) => self.request(task, input, ReplyFormat::AnyJson).await,
            result => result,
        }
    }

    async fn request(
        &self,
        task: &Task<'_>,
        input: &Value,
        format: ReplyFormat,
    ) -> Result<Value, ChatError> {
        let response_format = match format {
            ReplyFormat::Schema => json!({
                "type": "json_schema",
                "json_schema": { "name": task.schema_name, "strict": true, "schema": task.schema },
            }),
            ReplyFormat::AnyJson => json!({ "type": "json_object" }),
        };
        let body = json!({
            "model": self.model,
            "stream": false,
            // No sampling settings are sent: some services reject a
            // temperature other than their default.
            "max_tokens": MAX_TOKENS,
            "response_format": response_format,
            "messages": [
                {
                    "role": "system",
                    // Repeating the schema helps models that treat it only as a hint.
                    "content": format!(
                        "{}\n\nReply with JSON that matches this schema:\n{}",
                        task.instructions, task.schema
                    ),
                },
                // The input is one JSON document, so text inside it stays data.
                { "role": "user", "content": input.to_string() },
            ],
        });

        let mut request = self
            .http
            .post(self.endpoint.chat_completions_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_string());
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }
        let response = request.send().await.map_err(request_error)?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(request_error)?;
        read_response(status, &serde_json::from_str(&text).unwrap_or(Value::Null))
    }
}

fn request_error(error: reqwest::Error) -> ChatError {
    if error.is_timeout() {
        ChatError::Timeout
    } else {
        ChatError::ConnectionFailed {
            // The URL is dropped from the message; it adds nothing for the user.
            detail: error.without_url().to_string(),
        }
    }
}

/// Extracts the model's reply from a response, or the reason there is none.
fn read_response(status: u16, body: &Value) -> Result<Value, ChatError> {
    // Some gateways report an error inside a successful response.
    let error = &body["error"];
    if (200..300).contains(&status) && error.is_null() {
        return read_reply(body);
    }
    let code = error["code"]
        .as_u64()
        .and_then(|code| u16::try_from(code).ok());
    let detail = error["message"]
        .as_str()
        .unwrap_or("the request was rejected")
        .to_owned();
    Err(match code.unwrap_or(status) {
        401 => ChatError::AuthenticationFailed,
        // 402 is "out of credit" on services that bill per request.
        402 | 429 => ChatError::RateLimited,
        404 => ChatError::ModelUnavailable,
        408 => ChatError::Timeout,
        500..=599 => ChatError::ConnectionFailed { detail },
        _ => ChatError::Service { detail },
    })
}

/// Finds the model's text in a successful response and parses it as JSON.
fn read_reply(body: &Value) -> Result<Value, ChatError> {
    let choice = &body["choices"][0];
    if let Some(message) = choice["error"]["message"].as_str() {
        return Err(ChatError::Service {
            detail: message.to_owned(),
        });
    }
    let content = choice["message"]["content"]
        .as_str()
        .ok_or(ChatError::MalformedResponse)?;
    serde_json::from_str(without_code_fence(content)).map_err(|_| ChatError::MalformedResponse)
}

/// Removes a Markdown code fence around a reply. Some models wrap JSON in
/// one even when asked for JSON only.
fn without_code_fence(content: &str) -> &str {
    let trimmed = content.trim();
    trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|rest| rest.strip_suffix("```"))
        .map_or(trimmed, str::trim)
}

#[cfg(test)]
mod tests {
    use opentalk_test_support::{MockServer, Route};

    use super::*;

    const CHAT_PATH: &str = "/v1/chat/completions";

    fn reply(content: &str) -> String {
        json!({ "choices": [{ "message": { "role": "assistant", "content": content }, "finish_reason": "stop" }] })
            .to_string()
    }

    async fn server(status: u16, body: String) -> MockServer {
        MockServer::start(vec![Route::new(CHAT_PATH, status, body)]).await
    }

    async fn run(server: &MockServer) -> Result<Value, ChatError> {
        let endpoint = Endpoint::parse(&format!("{}/v1", server.base_url())).unwrap();
        let client = ChatClient::new(
            endpoint,
            Some("test-key".to_owned()),
            "vendor/model".to_owned(),
        );
        let schema = json!({ "type": "object" });
        let task = Task {
            instructions: "Do the task.",
            schema_name: "answer",
            schema: &schema,
        };
        client.run(&task, &json!({ "text": "hello" })).await
    }

    #[tokio::test]
    async fn returns_the_models_json_reply() {
        let server = server(200, reply(r#"{"answer":42}"#)).await;
        assert_eq!(run(&server).await, Ok(json!({ "answer": 42 })));
    }

    #[tokio::test]
    async fn asks_for_one_structured_unstreamed_reply() {
        let server = server(200, reply("{}")).await;
        run(&server).await.unwrap();

        let request = server.request_to(CHAT_PATH);
        assert_eq!(request["model"], "vendor/model");
        assert_eq!(request["stream"], false);
        assert_eq!(request["response_format"]["type"], "json_schema");
        assert_eq!(request["response_format"]["json_schema"]["name"], "answer");
        assert_eq!(request["response_format"]["json_schema"]["strict"], true);
        assert!(
            request["messages"][0]["content"]
                .as_str()
                .unwrap()
                .starts_with("Do the task.")
        );
        // The input stays a separate message of data.
        assert_eq!(
            request["messages"][1],
            json!({ "role": "user", "content": r#"{"text":"hello"}"# })
        );
    }

    #[tokio::test]
    async fn a_reply_wrapped_in_a_code_fence_is_still_read() {
        let server = server(200, reply("```json\n{\"answer\": 1}\n```")).await;
        assert_eq!(run(&server).await, Ok(json!({ "answer": 1 })));
    }

    #[tokio::test]
    async fn a_service_without_schema_support_is_asked_again_for_plain_json() {
        let unsupported = json!({ "error": { "code": 400, "message": "response_format json_schema is not supported" } });
        let server = server(400, unsupported.to_string()).await;
        assert!(matches!(run(&server).await, Err(ChatError::Service { .. })));

        // Both attempts were made, the second asking only for JSON.
        assert_eq!(server.requested_paths(), vec![CHAT_PATH, CHAT_PATH]);
        assert_eq!(
            server.requests_to(CHAT_PATH)[1]["response_format"],
            json!({ "type": "json_object" })
        );
    }

    #[tokio::test]
    async fn http_errors_are_told_apart() {
        for (status, expected) in [
            (401, ChatError::AuthenticationFailed),
            (402, ChatError::RateLimited),
            (429, ChatError::RateLimited),
            (404, ChatError::ModelUnavailable),
            (408, ChatError::Timeout),
        ] {
            let server = server(status, json!({ "error": { "message": "x" } }).to_string()).await;
            assert_eq!(run(&server).await, Err(expected), "status {status}");
            // None of these is worth a second attempt.
            assert_eq!(server.requested_paths().len(), 1, "status {status}");
        }
    }

    #[tokio::test]
    async fn an_error_inside_a_successful_response_is_noticed() {
        let body = json!({ "error": { "code": 402, "message": "insufficient credits" } });
        let server = server(200, body.to_string()).await;
        assert_eq!(run(&server).await, Err(ChatError::RateLimited));
    }

    #[tokio::test]
    async fn a_server_failure_is_a_connection_problem() {
        let server = server(
            502,
            json!({ "error": { "message": "model is down" } }).to_string(),
        )
        .await;
        assert_eq!(
            run(&server).await,
            Err(ChatError::ConnectionFailed {
                detail: "model is down".to_owned()
            })
        );
    }

    #[tokio::test]
    async fn a_reply_without_usable_json_is_malformed() {
        for body in [
            reply("Sure! Here you go."),
            json!({ "choices": [] }).to_string(),
            "{}".to_owned(),
        ] {
            let server = server(200, body).await;
            assert_eq!(run(&server).await, Err(ChatError::MalformedResponse));
        }
    }

    #[tokio::test]
    async fn the_key_is_sent_as_a_bearer_token_and_omitted_when_there_is_none() {
        let with_key = server(200, reply("{}")).await;
        run(&with_key).await.unwrap();
        assert_eq!(
            with_key.header_of(CHAT_PATH, "authorization").as_deref(),
            Some("Bearer test-key")
        );

        let without_key = server(200, reply("{}")).await;
        let endpoint = Endpoint::parse(&format!("{}/v1", without_key.base_url())).unwrap();
        let client = ChatClient::new(endpoint, None, "local-model".to_owned());
        let schema = json!({});
        let task = Task {
            instructions: "x",
            schema_name: "y",
            schema: &schema,
        };
        client.run(&task, &json!({})).await.unwrap();
        assert_eq!(without_key.header_of(CHAT_PATH, "authorization"), None);
    }
}
