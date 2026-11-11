//! Checks on what a generator returned, before any of it is shown.
//!
//! Provider output is untrusted: it may be missing styles, repeat one, or
//! be far longer than a spoken reply should be.

use crate::provider::{ReplyError, ReplyStyle, ReplySuggestion};

/// Longest reply shown, in characters. A reply is meant to be said aloud.
pub const MAX_REPLY_LENGTH: usize = 600;

/// Returns at most one usable suggestion per style, in display order.
///
/// Suggestions with no text are dropped, text is trimmed and cut to
/// [`MAX_REPLY_LENGTH`], and an empty meaning becomes `None`.
///
/// # Errors
///
/// Returns [`ReplyError::MalformedResponse`] if nothing usable is left.
pub fn validate(suggestions: Vec<ReplySuggestion>) -> Result<Vec<ReplySuggestion>, ReplyError> {
    let mut valid = Vec::new();
    for style in ReplyStyle::ALL {
        let candidate = suggestions
            .iter()
            .find(|suggestion| suggestion.style == style && !suggestion.text.trim().is_empty());
        if let Some(suggestion) = candidate {
            valid.push(ReplySuggestion {
                style,
                text: shortened(&suggestion.text),
                meaning: suggestion
                    .meaning
                    .as_deref()
                    .map(shortened)
                    .filter(|meaning| !meaning.is_empty()),
                asks_for_missing_facts: suggestion.asks_for_missing_facts,
            });
        }
    }
    if valid.is_empty() {
        Err(ReplyError::MalformedResponse)
    } else {
        Ok(valid)
    }
}

fn shortened(text: &str) -> String {
    text.trim().chars().take(MAX_REPLY_LENGTH).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion(style: ReplyStyle, text: &str) -> ReplySuggestion {
        ReplySuggestion {
            style,
            text: text.to_owned(),
            meaning: Some(" meaning ".to_owned()),
            asks_for_missing_facts: false,
        }
    }

    #[test]
    fn orders_suggestions_by_style() {
        let result = validate(vec![
            suggestion(ReplyStyle::Concise, "c"),
            suggestion(ReplyStyle::Professional, "p"),
            suggestion(ReplyStyle::Friendly, "f"),
        ]);
        let styles: Vec<ReplyStyle> = result.unwrap().iter().map(|s| s.style).collect();
        assert_eq!(styles, ReplyStyle::ALL);
    }

    #[test]
    fn keeps_only_the_first_suggestion_of_each_style() {
        let result = validate(vec![
            suggestion(ReplyStyle::Friendly, "first"),
            suggestion(ReplyStyle::Friendly, "second"),
        ])
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "first");
    }

    #[test]
    fn drops_suggestions_without_text() {
        let result = validate(vec![
            suggestion(ReplyStyle::Professional, "  "),
            suggestion(ReplyStyle::Concise, "Sure."),
        ])
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].style, ReplyStyle::Concise);
    }

    #[test]
    fn trims_text_and_meaning_and_cuts_overlong_text() {
        let long = format!("  {}  ", "x".repeat(MAX_REPLY_LENGTH + 50));
        let result = validate(vec![suggestion(ReplyStyle::Concise, &long)]).unwrap();
        assert_eq!(result[0].text.chars().count(), MAX_REPLY_LENGTH);
        assert_eq!(result[0].meaning.as_deref(), Some("meaning"));
    }

    #[test]
    fn an_empty_meaning_becomes_none() {
        let mut blank = suggestion(ReplyStyle::Concise, "Sure.");
        blank.meaning = Some("  ".to_owned());
        assert_eq!(validate(vec![blank]).unwrap()[0].meaning, None);
    }

    #[test]
    fn nothing_usable_is_an_error() {
        assert_eq!(validate(Vec::new()), Err(ReplyError::MalformedResponse));
        assert_eq!(
            validate(vec![suggestion(ReplyStyle::Friendly, "")]),
            Err(ReplyError::MalformedResponse)
        );
    }
}
