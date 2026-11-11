//! Translation and reply suggestions on an OpenAI-compatible service. Both
//! use the shared tasks from the provider-neutral crates.

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
use crate::client::{ChatClient, ChatError, Task};

const DISPLAY_NAME: &str = "OpenAI-compatible service";

/// What this translator offers, for display in settings.
#[must_use]
pub fn translation_info() -> TranslationProviderInfo {
    TranslationProviderInfo {
        id: PROVIDER_ID,
        display_name: DISPLAY_NAME,
        // Usually a remote service; the endpoint says whether it is local.
        is_cloud: true,
        // Which languages work depends on the model the user chose.
        languages: None,
        accepts_arabic_dialect: true,
    }
}

/// What this reply generator offers, for display in settings.
#[must_use]
pub fn reply_info() -> ReplyProviderInfo {
    ReplyProviderInfo {
        id: PROVIDER_ID,
        display_name: DISPLAY_NAME,
        is_cloud: true,
        accepts_arabic_dialect: true,
    }
}

/// Translation on an OpenAI-compatible service.
pub struct CompatibleTranslator {
    client: ChatClient,
}

impl CompatibleTranslator {
    #[must_use]
    pub fn new(client: ChatClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Translator for CompatibleTranslator {
    fn info(&self) -> TranslationProviderInfo {
        translation_info()
    }

    async fn translate(&self, request: &TranslationRequest) -> Result<String, TranslationError> {
        let task = Task {
            instructions: &translation_prompt::instructions(),
            schema_name: translation_prompt::SCHEMA_NAME,
            schema: &translation_prompt::schema(),
        };
        let reply = self
            .client
            .run(&task, &translation_prompt::input(request))
            .await?;
        translation_prompt::parse(reply)
    }
}

/// Reply suggestions on an OpenAI-compatible service.
pub struct CompatibleReplyGenerator {
    client: ChatClient,
}

impl CompatibleReplyGenerator {
    #[must_use]
    pub fn new(client: ChatClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ReplyGenerator for CompatibleReplyGenerator {
    fn info(&self) -> ReplyProviderInfo {
        reply_info()
    }

    async fn generate(&self, request: &ReplyRequest) -> Result<Vec<ReplySuggestion>, ReplyError> {
        let task = Task {
            instructions: &reply_prompt::instructions(),
            schema_name: reply_prompt::SCHEMA_NAME,
            schema: &reply_prompt::schema(),
        };
        let reply = self
            .client
            .run(&task, &reply_prompt::input(request))
            .await?;
        reply_prompt::parse(reply)
    }
}

impl From<ChatError> for TranslationError {
    fn from(error: ChatError) -> Self {
        match error {
            ChatError::AuthenticationFailed => Self::AuthenticationFailed,
            ChatError::RateLimited => Self::RateLimited,
            ChatError::ModelUnavailable => Self::ModelUnavailable,
            ChatError::ConnectionFailed { detail } => Self::ConnectionFailed { detail },
            ChatError::Timeout => Self::Timeout,
            ChatError::MalformedResponse => Self::MalformedResponse,
            ChatError::Service { detail } => Self::Provider { detail },
        }
    }
}

impl From<ChatError> for ReplyError {
    fn from(error: ChatError) -> Self {
        match error {
            ChatError::AuthenticationFailed => Self::AuthenticationFailed,
            ChatError::RateLimited => Self::RateLimited,
            ChatError::ModelUnavailable => Self::ModelUnavailable,
            ChatError::ConnectionFailed { detail } => Self::ConnectionFailed { detail },
            ChatError::Timeout => Self::Timeout,
            ChatError::MalformedResponse => Self::MalformedResponse,
            ChatError::Service { detail } => Self::Provider { detail },
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_language::ArabicStyle;
    use opentalk_replies::{ConversationTurn, ReplySettings, ReplyStyle, Speaker};
    use opentalk_test_support::{MockServer, Route};
    use serde_json::{Value, json};

    use super::*;
    use crate::endpoint::Endpoint;

    const CHAT_PATH: &str = "/v1/chat/completions";

    async fn server_replying(content: &Value) -> MockServer {
        let body = json!({ "choices": [{ "message": { "content": content.to_string() } }] });
        MockServer::start(vec![Route::new(CHAT_PATH, 200, body.to_string())]).await
    }

    fn client_for(server: &MockServer) -> ChatClient {
        let endpoint = Endpoint::parse(&format!("{}/v1", server.base_url())).unwrap();
        ChatClient::new(
            endpoint,
            Some("test-key".to_owned()),
            "vendor/model".to_owned(),
        )
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
        let translator = CompatibleTranslator::new(client_for(&server));

        assert_eq!(
            translator.translate(&translation_request()).await,
            Ok("Salam.".to_owned())
        );
        let request = server.request_to(CHAT_PATH);
        assert_eq!(
            request["response_format"]["json_schema"]["schema"],
            translation_prompt::schema()
        );
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
        let generator = CompatibleReplyGenerator::new(client_for(&server));

        let suggestions = generator.generate(&reply_request()).await.unwrap();
        let styles: Vec<ReplyStyle> = suggestions
            .iter()
            .map(|suggestion| suggestion.style)
            .collect();
        assert_eq!(styles, ReplyStyle::ALL);
        assert_eq!(
            server.request_to(CHAT_PATH)["response_format"]["json_schema"]["schema"],
            reply_prompt::schema()
        );
    }

    #[tokio::test]
    async fn an_unknown_model_is_reported_as_unavailable() {
        let server = MockServer::start(vec![Route::new(
            CHAT_PATH,
            404,
            r#"{"error":{"message":"no such model"}}"#,
        )])
        .await;
        let translator = CompatibleTranslator::new(client_for(&server));
        assert_eq!(
            translator.translate(&translation_request()).await,
            Err(TranslationError::ModelUnavailable)
        );
    }

    /// Checks a real service's address, TLS and error mapping without an
    /// account: a made-up key must be rejected as an authentication failure.
    ///
    /// ```text
    /// cargo test -p opentalk-provider-openai-compatible -- --ignored invalid_key
    /// ```
    #[tokio::test]
    #[ignore = "uses the network"]
    async fn live_openrouter_rejects_an_invalid_key() {
        let endpoint = Endpoint::parse(crate::DEFAULT_BASE_URL).unwrap();
        let client = ChatClient::new(
            endpoint,
            Some("invalid-key-for-testing".to_owned()),
            "openai/gpt-4o-mini".to_owned(),
        );
        let result = CompatibleTranslator::new(client)
            .translate(&translation_request())
            .await;
        assert_eq!(result, Err(TranslationError::AuthenticationFailed));
    }
}
