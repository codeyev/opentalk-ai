//! Reply suggestions through OpenAI's Responses API. The task itself (the
//! instructions and the shape of the request and reply) is the shared one
//! in `opentalk_replies::prompt`.

use async_trait::async_trait;
use opentalk_replies::{
    ReplyError, ReplyGenerator, ReplyProviderInfo, ReplyRequest, ReplySuggestion, prompt,
};

use crate::PROVIDER_ID;
use crate::responses::{ResponsesClient, ResponsesError, Task};

/// What this adapter offers. Available without an API key, for display in settings.
#[must_use]
pub fn reply_info() -> ReplyProviderInfo {
    ReplyProviderInfo {
        id: PROVIDER_ID,
        display_name: "OpenAI",
        is_cloud: true,
        accepts_arabic_dialect: true,
    }
}

/// Reply suggestions through OpenAI's Responses API.
pub struct OpenAiReplyGenerator {
    client: ResponsesClient,
}

impl OpenAiReplyGenerator {
    /// Creates a generator that authenticates with `api_key`.
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self::with_client(ResponsesClient::new(api_key))
    }

    pub(crate) fn with_client(client: ResponsesClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ReplyGenerator for OpenAiReplyGenerator {
    fn info(&self) -> ReplyProviderInfo {
        reply_info()
    }

    async fn generate(&self, request: &ReplyRequest) -> Result<Vec<ReplySuggestion>, ReplyError> {
        let task = Task {
            instructions: prompt::instructions(),
            schema_name: prompt::SCHEMA_NAME,
            schema: prompt::schema(),
        };
        let reply = self.client.run(&task, &prompt::input(request)).await?;
        prompt::parse(reply)
    }
}

impl From<ResponsesError> for ReplyError {
    fn from(error: ResponsesError) -> Self {
        match error {
            ResponsesError::AuthenticationFailed => Self::AuthenticationFailed,
            ResponsesError::RateLimited => Self::RateLimited,
            ResponsesError::ConnectionFailed { detail } => Self::ConnectionFailed { detail },
            ResponsesError::Timeout => Self::Timeout,
            ResponsesError::MalformedResponse => Self::MalformedResponse,
            ResponsesError::Refused => Self::Refused,
            ResponsesError::Provider { detail } => Self::Provider { detail },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_language::ArabicStyle;
    use opentalk_replies::{ConversationTurn, ReplySettings, ReplyStyle, Speaker};
    use serde_json::{Value, json};

    use super::*;
    use crate::responses::testing::{FakeHttp, replied, status};

    fn draft(text: &str) -> Value {
        json!({ "text": text, "meaning": "", "asks_for_missing_facts": false })
    }

    fn request(said_by_other: &str) -> ReplyRequest {
        ReplyRequest {
            settings: ReplySettings {
                reply_language: "en".to_owned(),
                meaning_language: None,
                business_context: "We build online stores.".to_owned(),
                arabic: ArabicStyle::default(),
            },
            conversation: vec![ConversationTurn {
                speaker: Speaker::Other,
                text: said_by_other.to_owned(),
            }],
        }
    }

    async fn generate(
        http: &Arc<FakeHttp>,
        said_by_other: &str,
    ) -> Result<Vec<ReplySuggestion>, ReplyError> {
        let client = ResponsesClient::with_http("test-key".to_owned(), http.clone());
        OpenAiReplyGenerator::with_client(client)
            .generate(&request(said_by_other))
            .await
    }

    #[tokio::test]
    async fn returns_one_suggestion_per_style() {
        let reply =
            json!({ "professional": draft("a"), "friendly": draft("b"), "concise": draft("c") });
        let http = FakeHttp::new(vec![replied(&reply)]);
        let suggestions = generate(&http, "Hello.").await.unwrap();
        let styles: Vec<ReplyStyle> = suggestions
            .iter()
            .map(|suggestion| suggestion.style)
            .collect();
        assert_eq!(styles, ReplyStyle::ALL);
    }

    #[tokio::test]
    async fn sends_the_shared_reply_task() {
        let reply =
            json!({ "professional": draft("a"), "friendly": draft("b"), "concise": draft("c") });
        let http = FakeHttp::new(vec![replied(&reply)]);
        generate(&http, "Hello.").await.unwrap();

        let (_, _, body) = http.first_request();
        assert_eq!(body["instructions"], prompt::instructions());
        assert_eq!(body["text"]["format"]["schema"], prompt::schema());
        assert_eq!(http.first_input(), prompt::input(&request("Hello.")));
    }

    #[tokio::test]
    async fn api_errors_become_reply_errors() {
        let http = FakeHttp::new(vec![status(401, &json!({}))]);
        assert_eq!(
            generate(&http, "Hello.").await,
            Err(ReplyError::AuthenticationFailed)
        );
    }

    /// Talks to the real OpenAI service. Run it deliberately with:
    ///
    /// ```text
    /// OPENAI_API_KEY=... cargo test -p opentalk-provider-openai -- --ignored real_key
    /// ```
    ///
    /// The business context contains no price, so a good reply to a price
    /// question must ask rather than quote a number.
    #[tokio::test]
    #[ignore = "uses the network and a real API key"]
    async fn live_replies_with_a_real_key_do_not_invent_a_price() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY to run this test");
        let generator = OpenAiReplyGenerator::new(api_key);
        let suggestions = generator
            .generate(&request("How much would a store with 400 products cost?"))
            .await
            .expect("suggestions should be generated");

        assert_eq!(suggestions.len(), 3);
        for suggestion in &suggestions {
            let quotes_a_price = suggestion.text.contains('$') || suggestion.text.contains("USD");
            assert!(!quotes_a_price, "invented a price: {}", suggestion.text);
        }
        assert!(
            suggestions
                .iter()
                .any(|suggestion| suggestion.asks_for_missing_facts)
        );
    }
}
