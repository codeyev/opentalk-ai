//! Translation through OpenAI's Responses API. The task itself (the
//! instructions and the shape of the request and reply) is the shared one
//! in `opentalk_translation::prompt`.

use async_trait::async_trait;
use opentalk_translation::{
    TranslationError, TranslationProviderInfo, TranslationRequest, Translator, prompt,
};

use crate::PROVIDER_ID;
use crate::responses::{ResponsesClient, ResponsesError, Task};

/// What this adapter offers. Available without an API key, for display in settings.
#[must_use]
pub fn translation_info() -> TranslationProviderInfo {
    TranslationProviderInfo {
        id: PROVIDER_ID,
        display_name: "OpenAI",
        is_cloud: true,
        // OpenAI does not publish a list of supported language pairs for this model.
        languages: None,
        accepts_arabic_dialect: true,
    }
}

/// Translation through OpenAI's Responses API.
pub struct OpenAiTranslator {
    client: ResponsesClient,
}

impl OpenAiTranslator {
    /// Creates a translator that authenticates with `api_key`.
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self::with_client(ResponsesClient::new(api_key))
    }

    pub(crate) fn with_client(client: ResponsesClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Translator for OpenAiTranslator {
    fn info(&self) -> TranslationProviderInfo {
        translation_info()
    }

    async fn translate(&self, request: &TranslationRequest) -> Result<String, TranslationError> {
        let task = Task {
            instructions: prompt::instructions(),
            schema_name: prompt::SCHEMA_NAME,
            schema: prompt::schema(),
        };
        let reply = self.client.run(&task, &prompt::input(request)).await?;
        prompt::parse(reply)
    }
}

impl From<ResponsesError> for TranslationError {
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

    use serde_json::json;

    use super::*;
    use crate::responses::testing::{FakeHttp, replied, status};

    fn request(text: &str) -> TranslationRequest {
        TranslationRequest {
            text: text.to_owned(),
            source_language: Some("en".to_owned()),
            target_language: "az".to_owned(),
            glossary: Vec::new(),
            arabic: None,
        }
    }

    async fn translate(http: &Arc<FakeHttp>, text: &str) -> Result<String, TranslationError> {
        let client = ResponsesClient::with_http("test-key".to_owned(), http.clone());
        OpenAiTranslator::with_client(client)
            .translate(&request(text))
            .await
    }

    #[tokio::test]
    async fn returns_the_translation() {
        let http = FakeHttp::new(vec![replied(&json!({ "translation": "Salam." }))]);
        assert_eq!(translate(&http, "Hello.").await, Ok("Salam.".to_owned()));
    }

    #[tokio::test]
    async fn sends_the_shared_translation_task() {
        let http = FakeHttp::new(vec![replied(&json!({ "translation": "x" }))]);
        translate(&http, "Hello.").await.unwrap();

        let (_, _, body) = http.first_request();
        assert_eq!(body["instructions"], prompt::instructions());
        assert_eq!(body["text"]["format"]["schema"], prompt::schema());
        assert_eq!(http.first_input(), prompt::input(&request("Hello.")));
    }

    #[tokio::test]
    async fn a_reply_of_the_wrong_shape_is_malformed() {
        let http = FakeHttp::new(vec![replied(&json!({ "something_else": 1 }))]);
        assert_eq!(
            translate(&http, "Hello.").await,
            Err(TranslationError::MalformedResponse)
        );
    }

    #[tokio::test]
    async fn api_errors_become_translation_errors() {
        let http = FakeHttp::new(vec![status(429, &json!({}))]);
        assert_eq!(
            translate(&http, "Hello.").await,
            Err(TranslationError::RateLimited)
        );
    }

    /// Talks to the real OpenAI service. Run it deliberately with:
    ///
    /// ```text
    /// OPENAI_API_KEY=... cargo test -p opentalk-provider-openai -- --ignored real_key
    /// ```
    #[tokio::test]
    #[ignore = "uses the network and a real API key"]
    async fn live_translation_with_a_real_key_keeps_numbers() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY to run this test");
        let source = "The invoice for 1,200 USD is due in 14 days.";
        let translation = OpenAiTranslator::new(api_key)
            .translate(&request(source))
            .await;
        let translation = translation.expect("the translation should succeed");
        let missing = opentalk_translation::preservation::unpreserved(source, &translation, &[]);
        assert!(missing.is_empty(), "not preserved: {missing:?}");
    }

    /// Checks the real endpoint, TLS and error mapping without needing an
    /// account: a made-up key must be rejected as an authentication failure.
    #[tokio::test]
    #[ignore = "uses the network"]
    async fn live_translation_rejects_an_invalid_key() {
        let translator = OpenAiTranslator::new("invalid-key-for-testing".to_owned());
        let result = translator.translate(&request("Hello.")).await;
        assert_eq!(result, Err(TranslationError::AuthenticationFailed));
    }
}
