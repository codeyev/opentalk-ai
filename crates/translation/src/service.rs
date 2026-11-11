//! Translating one piece of text and checking the result.

use opentalk_language::{ArabicStyle, is_arabic};

use crate::preservation;
use crate::provider::{
    GlossaryTerm, MAX_TEXT_LENGTH, Translation, TranslationError, TranslationRequest, Translator,
};

/// What every translation in a session has in common.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TranslationSettings {
    /// BCP 47 tag of the language to translate into.
    pub target_language: String,
    pub glossary: Vec<GlossaryTerm>,
    /// How to write Arabic. Only used when the target language is Arabic.
    pub arabic: ArabicStyle,
}

/// Translates `text` and reports which protected parts did not survive.
///
/// When the target is Arabic, the user's Arabic style is passed to the
/// provider and then enforced where that is possible without a model:
/// with tashkeel off, any diacritics the provider added are removed.
///
/// # Errors
///
/// Returns the provider's error. Text longer than [`MAX_TEXT_LENGTH`]
/// characters is refused without contacting the provider.
pub async fn translate(
    translator: &dyn Translator,
    text: &str,
    source_language: Option<&str>,
    settings: &TranslationSettings,
) -> Result<Translation, TranslationError> {
    if text.chars().count() > MAX_TEXT_LENGTH {
        return Err(TranslationError::Provider {
            detail: "the text is too long to translate".to_owned(),
        });
    }
    let arabic = is_arabic(&settings.target_language).then(|| settings.arabic.clone());
    let request = TranslationRequest {
        text: text.to_owned(),
        source_language: source_language.map(str::to_owned),
        target_language: settings.target_language.clone(),
        glossary: settings.glossary.clone(),
        arabic: arabic.clone(),
    };
    let translated = translator.translate(&request).await?;
    let translated = arabic.map_or(translated.clone(), |style| style.apply_to(&translated));
    Ok(Translation {
        unpreserved: preservation::unpreserved(text, &translated, &settings.glossary),
        text: translated,
        target_language: settings.target_language.clone(),
    })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use opentalk_language::Tashkeel;

    use super::*;
    use crate::provider::TranslationProviderInfo;

    /// Always answers with the same text, or the same error.
    struct FixedTranslator(Result<&'static str, TranslationError>);

    #[async_trait]
    impl Translator for FixedTranslator {
        fn info(&self) -> TranslationProviderInfo {
            TranslationProviderInfo {
                id: "fixed",
                display_name: "Fixed",
                is_cloud: false,
                languages: None,
                accepts_arabic_dialect: false,
            }
        }

        async fn translate(
            &self,
            _request: &TranslationRequest,
        ) -> Result<String, TranslationError> {
            self.0.clone().map(str::to_owned)
        }
    }

    fn settings(target_language: &str) -> TranslationSettings {
        TranslationSettings {
            target_language: target_language.to_owned(),
            ..TranslationSettings::default()
        }
    }

    /// Runs a future that never actually waits, without needing an async runtime.
    fn run<T>(future: impl Future<Output = T>) -> T {
        use std::task::{Context, Poll, Waker};
        let mut future = std::pin::pin!(future);
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("the future was expected to finish immediately"),
        }
    }

    #[test]
    fn a_good_translation_has_nothing_to_check() {
        let translator = FixedTranslator(Ok("Qiyməti 1,200 USD-dir."));
        let result = run(translate(
            &translator,
            "It costs 1,200 USD.",
            Some("en"),
            &settings("az"),
        ));
        assert_eq!(
            result,
            Ok(Translation {
                text: "Qiyməti 1,200 USD-dir.".to_owned(),
                target_language: "az".to_owned(),
                unpreserved: Vec::new(),
            })
        );
    }

    #[test]
    fn a_translation_that_changed_a_number_is_flagged() {
        let translator = FixedTranslator(Ok("Qiyməti 1,300 USD-dir."));
        let result = run(translate(
            &translator,
            "It costs 1,200 USD.",
            Some("en"),
            &settings("az"),
        ));
        assert_eq!(
            result.map(|translation| translation.unpreserved),
            Ok(vec!["1,200".to_owned()])
        );
    }

    #[test]
    fn a_provider_error_is_passed_on() {
        let translator = FixedTranslator(Err(TranslationError::RateLimited));
        let result = run(translate(&translator, "Hello.", None, &settings("az")));
        assert_eq!(result, Err(TranslationError::RateLimited));
    }

    #[test]
    fn overlong_text_is_refused_without_calling_the_provider() {
        let translator = FixedTranslator(Ok("unused"));
        let text = "x".repeat(MAX_TEXT_LENGTH + 1);
        let result = run(translate(&translator, &text, None, &settings("az")));
        assert!(matches!(result, Err(TranslationError::Provider { .. })));
    }

    #[test]
    fn arabic_with_tashkeel_off_has_added_diacritics_removed() {
        let translator = FixedTranslator(Ok("مَرْحَبًا"));
        let result = run(translate(&translator, "Hello", Some("en"), &settings("ar")));
        assert_eq!(
            result.map(|translation| translation.text),
            Ok("مرحبا".to_owned())
        );
    }

    #[test]
    fn arabic_with_full_tashkeel_keeps_its_diacritics() {
        let translator = FixedTranslator(Ok("مَرْحَبًا"));
        let mut settings = settings("ar-EG");
        settings.arabic.tashkeel = Tashkeel::Full;
        let result = run(translate(&translator, "Hello", Some("en"), &settings));
        assert_eq!(
            result.map(|translation| translation.text),
            Ok("مَرْحَبًا".to_owned())
        );
    }

    #[test]
    fn the_arabic_style_is_ignored_for_other_languages() {
        // Persian uses the same marks; they are only removed for Arabic output.
        let translator = FixedTranslator(Ok("سَلام"));
        let result = run(translate(&translator, "Hello", Some("en"), &settings("fa")));
        assert_eq!(
            result.map(|translation| translation.text),
            Ok("سَلام".to_owned())
        );
    }
}
