//! Requests to the Ollama server, written against Ollama's API reference
//! as published in November 2026.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::endpoint::LocalEndpoint;

/// Longest wait for the server to describe itself.
const STATUS_TIMEOUT: Duration = Duration::from_secs(3);
/// Longest wait for a model's reply. Local models can be slow, and the
/// first request also loads the model into memory.
const CHAT_TIMEOUT: Duration = Duration::from_secs(90);

/// Why the local server could not do what was asked.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum OllamaError {
    #[error("the address must be an http address on this computer, such as http://localhost:11434")]
    NotLocal,
    #[error("no Ollama server is answering at that address")]
    NotRunning,
    #[error("no local model has been chosen")]
    NoModelSelected,
    #[error("the model {model} is not installed in Ollama")]
    ModelMissing { model: String },
    #[error("the local model did not answer in time")]
    Timeout,
    #[error("the local server sent a response that could not be understood")]
    MalformedResponse,
    #[error("the local server reported an error: {detail}")]
    Server { detail: String },
}

/// A model installed in Ollama.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModel {
    pub name: String,
    /// Size on disk, in bytes. Running it needs roughly this much memory.
    pub size_bytes: u64,
    /// The model's size class as Ollama reports it, such as "8.0B".
    pub parameter_size: Option<String>,
}

/// What a running Ollama server reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub version: String,
    pub models: Vec<LocalModel>,
}

#[derive(Deserialize)]
struct VersionReply {
    version: String,
}

#[derive(Deserialize)]
struct TagsReply {
    models: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    details: ModelDetails,
}

#[derive(Deserialize, Default)]
struct ModelDetails {
    parameter_size: Option<String>,
}

/// A connection to the Ollama server on this computer.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    endpoint: LocalEndpoint,
    http: reqwest::Client,
}

impl OllamaClient {
    #[must_use]
    pub fn new(endpoint: LocalEndpoint) -> Self {
        // A system proxy would send "local" requests to another machine.
        let http = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_default();
        Self { endpoint, http }
    }

    /// Health check and model discovery: the server's version and the
    /// models installed in it.
    ///
    /// # Errors
    ///
    /// Returns [`OllamaError::NotRunning`] if nothing answers at the endpoint.
    pub async fn status(&self) -> Result<OllamaStatus, OllamaError> {
        let version: VersionReply = self.get("/api/version").await?;
        Ok(OllamaStatus {
            version: version.version,
            models: self.models().await?,
        })
    }

    async fn models(&self) -> Result<Vec<LocalModel>, OllamaError> {
        let tags: TagsReply = self.get("/api/tags").await?;
        Ok(tags
            .models
            .into_iter()
            .map(|entry| LocalModel {
                name: entry.name,
                size_bytes: entry.size,
                parameter_size: entry.details.parameter_size,
            })
            .collect())
    }

    async fn get<Reply: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<Reply, OllamaError> {
        let response = self
            .http
            .get(self.endpoint.url(path))
            .timeout(STATUS_TIMEOUT)
            .send()
            .await
            .map_err(request_error)?;
        let body = response.text().await.map_err(request_error)?;
        serde_json::from_str(&body).map_err(|_| OllamaError::MalformedResponse)
    }

    /// Runs a structured task on a model and returns its reply as JSON.
    ///
    /// The model must already be installed; this never downloads one.
    pub(crate) async fn run(
        &self,
        model: &str,
        instructions: &str,
        input: &Value,
        schema: &Value,
    ) -> Result<Value, OllamaError> {
        if model.trim().is_empty() {
            return Err(OllamaError::NoModelSelected);
        }
        if !is_installed(&self.models().await?, model) {
            return Err(OllamaError::ModelMissing {
                model: model.to_owned(),
            });
        }

        let body = json!({
            "model": model,
            "stream": false,
            // Reasoning models would otherwise think at length before
            // answering: about a minute per reply instead of a second or two
            // when measured with a 9B model. Models without a thinking mode
            // accept and ignore this.
            "think": false,
            // The reply must be JSON of this shape.
            "format": schema,
            "options": { "temperature": 0 },
            "messages": [
                {
                    "role": "system",
                    // Ollama advises repeating the schema in the prompt.
                    "content": format!("{instructions}\n\nReply with JSON that matches this schema:\n{schema}"),
                },
                { "role": "user", "content": input.to_string() },
            ],
        });
        let response = self
            .http
            .post(self.endpoint.url("/api/chat"))
            .timeout(CHAT_TIMEOUT)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(request_error)?;
        let status = response.status();
        let reply: Value = serde_json::from_str(&response.text().await.map_err(request_error)?)
            .unwrap_or(Value::Null);

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(OllamaError::ModelMissing {
                model: model.to_owned(),
            });
        }
        if !status.is_success() {
            return Err(OllamaError::Server {
                detail: reply["error"]
                    .as_str()
                    .map_or_else(|| status.to_string(), str::to_owned),
            });
        }
        let content = reply["message"]["content"]
            .as_str()
            .ok_or(OllamaError::MalformedResponse)?;
        serde_json::from_str(content).map_err(|_| OllamaError::MalformedResponse)
    }
}

/// Whether `model` is among the installed models. Ollama lists a model
/// pulled without a tag as `name:latest`.
fn is_installed(models: &[LocalModel], model: &str) -> bool {
    models
        .iter()
        .any(|installed| installed.name == model || installed.name == format!("{model}:latest"))
}

fn request_error(error: reqwest::Error) -> OllamaError {
    if error.is_timeout() {
        OllamaError::Timeout
    } else if error.is_connect() {
        OllamaError::NotRunning
    } else {
        OllamaError::Server {
            detail: error.without_url().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MockServer, Route, client_for};

    const TAGS: &str = r#"{"models":[
        {"name":"qwen3.5:latest","model":"qwen3.5:latest","size":11000000000,"details":{"parameter_size":"9.0B"}},
        {"name":"tiny:1b","size":800000000}
    ]}"#;

    fn chat_reply(content: &Value) -> String {
        json!({ "model": "tiny:1b", "message": { "role": "assistant", "content": content.to_string() }, "done": true })
            .to_string()
    }

    async fn server_with_chat(status: u16, body: String) -> MockServer {
        MockServer::start(vec![
            Route::new("/api/tags", 200, TAGS),
            Route::new("/api/chat", status, body),
        ])
        .await
    }

    async fn run(server: &MockServer, model: &str) -> Result<Value, OllamaError> {
        client_for(server)
            .run(
                model,
                "Do the task.",
                &json!({ "text": "hello" }),
                &json!({ "type": "object" }),
            )
            .await
    }

    #[tokio::test]
    async fn status_reports_the_version_and_installed_models() {
        let server = MockServer::start(vec![
            Route::new("/api/version", 200, r#"{"version":"0.40.14"}"#),
            Route::new("/api/tags", 200, TAGS),
        ])
        .await;

        let status = client_for(&server).status().await.unwrap();
        assert_eq!(status.version, "0.40.14");
        assert_eq!(
            status.models,
            vec![
                LocalModel {
                    name: "qwen3.5:latest".to_owned(),
                    size_bytes: 11_000_000_000,
                    parameter_size: Some("9.0B".to_owned()),
                },
                LocalModel {
                    name: "tiny:1b".to_owned(),
                    size_bytes: 800_000_000,
                    parameter_size: None,
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_server_that_is_not_running_is_reported_as_such() {
        // Reserve a port, then close it, so nothing is listening there.
        let server = MockServer::start(Vec::new()).await;
        let client = client_for(&server);
        drop(server);
        assert_eq!(client.status().await, Err(OllamaError::NotRunning));
    }

    #[tokio::test]
    async fn something_that_is_not_ollama_is_a_malformed_response() {
        let server =
            MockServer::start(vec![Route::new("/api/version", 200, "<html>hello</html>")]).await;
        assert_eq!(
            client_for(&server).status().await,
            Err(OllamaError::MalformedResponse)
        );
    }

    #[tokio::test]
    async fn a_task_returns_the_models_json_reply() {
        let server = server_with_chat(200, chat_reply(&json!({ "answer": 42 }))).await;
        assert_eq!(run(&server, "tiny:1b").await, Ok(json!({ "answer": 42 })));
    }

    #[tokio::test]
    async fn a_task_asks_for_one_structured_unstreamed_reply() {
        let server = server_with_chat(200, chat_reply(&json!({}))).await;
        run(&server, "tiny:1b").await.unwrap();

        let request = server.request_to("/api/chat");
        assert_eq!(request["model"], "tiny:1b");
        assert_eq!(request["stream"], false);
        assert_eq!(request["think"], false);
        assert_eq!(request["format"], json!({ "type": "object" }));
        assert_eq!(request["options"]["temperature"], 0);
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
    async fn a_model_pulled_without_a_tag_is_found_as_latest() {
        let server = server_with_chat(200, chat_reply(&json!({}))).await;
        assert!(run(&server, "qwen3.5").await.is_ok());
    }

    #[tokio::test]
    async fn a_model_that_is_not_installed_is_never_requested_or_downloaded() {
        let server = server_with_chat(200, chat_reply(&json!({}))).await;
        assert_eq!(
            run(&server, "not-installed").await,
            Err(OllamaError::ModelMissing {
                model: "not-installed".to_owned()
            })
        );
        // Only the model list was read: no chat request, and nothing was pulled.
        assert_eq!(server.requested_paths(), vec!["/api/tags"]);
    }

    #[tokio::test]
    async fn no_model_selected_is_reported_without_contacting_the_server() {
        let server = server_with_chat(200, chat_reply(&json!({}))).await;
        assert_eq!(run(&server, "  ").await, Err(OllamaError::NoModelSelected));
        assert!(server.requested_paths().is_empty());
    }

    #[tokio::test]
    async fn a_server_error_such_as_running_out_of_memory_is_passed_on() {
        let body = r#"{"error":"model requires more system memory than is available"}"#;
        let server = server_with_chat(500, body.to_owned()).await;
        assert_eq!(
            run(&server, "tiny:1b").await,
            Err(OllamaError::Server {
                detail: "model requires more system memory than is available".to_owned()
            })
        );
    }

    #[tokio::test]
    async fn a_reply_that_is_not_json_is_malformed() {
        let not_json = json!({ "message": { "content": "Sure! Here you go." } }).to_string();
        let server = server_with_chat(200, not_json).await;
        assert_eq!(
            run(&server, "tiny:1b").await,
            Err(OllamaError::MalformedResponse)
        );
    }
}
