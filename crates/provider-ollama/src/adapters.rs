//! Translation and reply suggestions on a local model. Both use the shared
//! tasks from the provider-neutral crates, so a local model is given
//! exactly the same rules as a cloud one.

use async_trait::async_trait;
use opentalk_replies::{
    ReplyError, ReplyGenerator, ReplyProviderInfo, ReplyRequest, ReplySuggestion,
    prompt as reply_prompt,
};
use opentalk_translation::{
    TranslationError, TranslationProviderInfo, TranslationRequest, Translator,
    prompt as translation_prompt,
};

use crate::PROVIDER_ID;
use crate::client::{OllamaClient, OllamaError};

const DISPLAY_NAME: &str = "Ollama";

/// What the local translator offers, for display in settings.
#[must_use]
pub fn translation_info() -> TranslationProviderInfo {
    TranslationProviderInfo {
        id: PROVIDER_ID,
        display_name: DISPLAY_NAME,
        is_cloud: false,
        // Which languages work depends entirely on the model the user installed.
        languages: None,
        accepts_arabic_dialect: true,
    }
}

/// What the local reply generator offers, for display in settings.
#[must_use]
pub fn reply_info() -> ReplyProviderInfo {
    ReplyProviderInfo {
        id: PROVIDER_ID,
        display_name: DISPLAY_NAME,
        is_cloud: false,
        accepts_arabic_dialect: true,
    }
}

/// Translation on a model installed in Ollama.
pub struct OllamaTranslator {
    client: OllamaClient,
    model: String,
}

impl OllamaTranslator {
    #[must_use]
    pub fn new(client: OllamaClient, model: String) -> Self {
        Self { client, model }
    }
}

#[async_trait]
impl Translator for OllamaTranslator {
    fn info(&self) -> TranslationProviderInfo {
        translation_info()
    }

    async fn translate(&self, request: &TranslationRequest) -> Result<String, TranslationError> {
        let reply = self
            .client
            .run(
                &self.model,
                &translation_prompt::instructions(),
                &translation_prompt::input(request),
                &translation_prompt::schema(),
            )
            .await?;
        translation_prompt::parse(reply)
    }
}

/// Reply suggestions on a model installed in Ollama.
pub struct OllamaReplyGenerator {
    client: OllamaClient,
    model: String,
}

impl OllamaReplyGenerator {
    #[must_use]
    pub fn new(client: OllamaClient, model: String) -> Self {
        Self { client, model }
    }
}

#[async_trait]
impl ReplyGenerator for OllamaReplyGenerator {
    fn info(&self) -> ReplyProviderInfo {
        reply_info()
    }

    async fn generate(&self, request: &ReplyRequest) -> Result<Vec<ReplySuggestion>, ReplyError> {
        let reply = self
            .client
            .run(
                &self.model,
                &reply_prompt::instructions(),
                &reply_prompt::input(request),
                &reply_prompt::schema(),
            )
            .await?;
        reply_prompt::parse(reply)
    }
}

impl From<OllamaError> for TranslationError {
    fn from(error: OllamaError) -> Self {
        match error {
            OllamaError::NotLocal | OllamaError::NotRunning => Self::ConnectionFailed {
                detail: error.to_string(),
            },
            OllamaError::NoModelSelected | OllamaError::ModelMissing { .. } => {
                Self::ModelUnavailable
            }
            OllamaError::Timeout => Self::Timeout,
            OllamaError::MalformedResponse => Self::MalformedResponse,
            OllamaError::Server { detail } => Self::Provider { detail },
        }
    }
}

impl From<OllamaError> for ReplyError {
    fn from(error: OllamaError) -> Self {
        match error {
            OllamaError::NotLocal | OllamaError::NotRunning => Self::ConnectionFailed {
                detail: error.to_string(),
            },
            OllamaError::NoModelSelected | OllamaError::ModelMissing { .. } => {
                Self::ModelUnavailable
            }
            OllamaError::Timeout => Self::Timeout,
            OllamaError::MalformedResponse => Self::MalformedResponse,
            OllamaError::Server { detail } => Self::Provider { detail },
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_language::ArabicStyle;
    use opentalk_replies::{ConversationTurn, ReplySettings, ReplyStyle, Speaker};
    use serde_json::{Value, json};

    use super::*;
    use crate::testing::{MockServer, Route, client_for};

    const TAGS: &str = r#"{"models":[{"name":"tiny:1b","size":1}]}"#;

    async fn server_replying(content: &Value) -> MockServer {
        let reply = json!({ "message": { "role": "assistant", "content": content.to_string() } });
        MockServer::start(vec![
            Route::new("/api/tags", 200, TAGS),
            Route::new("/api/chat", 200, reply.to_string()),
        ])
        .await
    }

    fn translation_request() -> TranslationRequest {
        TranslationRequest {
            text: "Hello.".to_owned(),
            source_language: Some("en".to_owned()),
            target_language: "az".to_owned(),
            glossary: Vec::new(),
            arabic: None,
        }
    }

    fn reply_request() -> ReplyRequest {
        ReplyRequest {
            settings: ReplySettings {
                reply_language: "en".to_owned(),
                meaning_language: None,
                business_context: String::new(),
                arabic: ArabicStyle::default(),
            },
            conversation: vec![ConversationTurn {
                speaker: Speaker::Other,
                text: "How much is it?".to_owned(),
            }],
        }
    }

    #[tokio::test]
    async fn translates_with_the_shared_translation_task() {
        let server = server_replying(&json!({ "translation": "Salam." })).await;
        let translator = OllamaTranslator::new(client_for(&server), "tiny:1b".to_owned());

        assert_eq!(
            translator.translate(&translation_request()).await,
            Ok("Salam.".to_owned())
        );
        let request = server.request_to("/api/chat");
        assert_eq!(request["format"], translation_prompt::schema());
        assert!(
            request["messages"][0]["content"]
                .as_str()
                .unwrap()
                .starts_with(&translation_prompt::instructions())
        );
    }

    #[tokio::test]
    async fn suggests_replies_with_the_shared_reply_task() {
        let draft =
            json!({ "text": "Let me check.", "meaning": "", "asks_for_missing_facts": true });
        let server =
            server_replying(&json!({ "professional": draft, "friendly": draft, "concise": draft }))
                .await;
        let generator = OllamaReplyGenerator::new(client_for(&server), "tiny:1b".to_owned());

        let suggestions = generator.generate(&reply_request()).await.unwrap();
        let styles: Vec<ReplyStyle> = suggestions
            .iter()
            .map(|suggestion| suggestion.style)
            .collect();
        assert_eq!(styles, ReplyStyle::ALL);
        assert!(suggestions[0].asks_for_missing_facts);
        assert_eq!(
            server.request_to("/api/chat")["format"],
            reply_prompt::schema()
        );
    }

    #[tokio::test]
    async fn a_missing_model_is_reported_as_unavailable() {
        let server = server_replying(&json!({})).await;
        let translator = OllamaTranslator::new(client_for(&server), "not-installed".to_owned());
        assert_eq!(
            translator.translate(&translation_request()).await,
            Err(TranslationError::ModelUnavailable)
        );

        let generator = OllamaReplyGenerator::new(client_for(&server), "not-installed".to_owned());
        assert_eq!(
            generator.generate(&reply_request()).await,
            Err(ReplyError::ModelUnavailable)
        );
    }

    #[tokio::test]
    async fn local_providers_are_declared_as_not_cloud() {
        assert!(!translation_info().is_cloud);
        assert!(!reply_info().is_cloud);
    }

    /// Uses a real Ollama server and model on this computer. Run it with:
    ///
    /// ```text
    /// OLLAMA_MODEL=<installed model> cargo test -p opentalk-provider-ollama -- --ignored live --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "needs a running Ollama server and an installed model"]
    async fn live_local_model_translates_and_suggests_replies() {
        let model = std::env::var("OLLAMA_MODEL").expect("set OLLAMA_MODEL to an installed model");
        let endpoint = crate::LocalEndpoint::parse(crate::DEFAULT_ENDPOINT).unwrap();
        let client = OllamaClient::new(endpoint);
        let status = client.status().await.expect("Ollama should be running");
        println!(
            "Ollama {} with {} model(s)",
            status.version,
            status.models.len()
        );

        let started = std::time::Instant::now();
        let mut request = translation_request();
        request.text = "The invoice for 1,200 USD is due in 14 days.".to_owned();
        request.target_language = "ru".to_owned();
        let translation = OllamaTranslator::new(client.clone(), model.clone())
            .translate(&request)
            .await
            .expect("the translation should succeed");
        println!("translation ({:.1?}): {translation}", started.elapsed());
        let missing =
            opentalk_translation::preservation::unpreserved(&request.text, &translation, &[]);
        assert!(missing.is_empty(), "not preserved: {missing:?}");

        let started = std::time::Instant::now();
        let suggestions = OllamaReplyGenerator::new(client, model)
            .generate(&reply_request())
            .await
            .expect("suggestions should be generated");
        println!("replies ({:.1?}):", started.elapsed());
        for suggestion in &suggestions {
            println!(
                "  {:?} asks={} {}",
                suggestion.style, suggestion.asks_for_missing_facts, suggestion.text
            );
        }
        assert_eq!(suggestions.len(), 3);
    }
}
