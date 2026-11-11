//! Small language helpers shared by translation and reply suggestions.

pub mod arabic;

pub use arabic::{ArabicStyle, Tashkeel};

/// The language part of a BCP 47 tag, lowercased: `ar-EG` gives `ar`.
fn language_of(tag: &str) -> String {
    tag.split('-')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

/// Whether two BCP 47 tags name the same language, ignoring region and
/// script: `en` and `en-GB` are the same language, `en` and `az` are not.
#[must_use]
pub fn same_language(first: &str, second: &str) -> bool {
    !language_of(first).is_empty() && language_of(first) == language_of(second)
}

/// Whether a BCP 47 tag names Arabic, in any regional form.
#[must_use]
pub fn is_arabic(tag: &str) -> bool {
    language_of(tag) == "ar"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_are_compared_without_region() {
        assert!(same_language("en", "en-GB"));
        assert!(same_language("AR-eg", "ar"));
        assert!(!same_language("en", "az"));
        assert!(!same_language("", ""));
    }

    #[test]
    fn arabic_is_recognized_in_any_regional_form() {
        assert!(is_arabic("ar"));
        assert!(is_arabic("ar-EG"));
        assert!(is_arabic("AR-sa"));
        assert!(!is_arabic("fa"));
        assert!(!is_arabic("az-Arab"));
    }
}
