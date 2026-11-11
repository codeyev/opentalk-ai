//! Checks that the parts of a sentence which must not change survived
//! translation: numbers (prices, dates, quantities), links and email
//! addresses, and glossary terms.
//!
//! A language model can silently drop a digit or "translate" a product
//! name. These checks cannot prove a translation is right, but they catch
//! that kind of slip so the UI can ask the user to verify.

use opentalk_language::arabic::strip_tashkeel;

use crate::provider::GlossaryTerm;

/// Returns the protected parts of `source` that are missing from `translation`.
///
/// Each entry is the text as it appeared in the source, or for a glossary
/// term, the translation that was expected. An empty result means nothing
/// was found missing.
#[must_use]
pub fn unpreserved(source: &str, translation: &str, glossary: &[GlossaryTerm]) -> Vec<String> {
    let translated_numbers: Vec<String> =
        numbers(translation).iter().map(|n| digits_of(n)).collect();
    let translation_folded = folded(translation);

    let mut missing = Vec::new();
    for link in links(source) {
        if !translation.contains(&link) {
            missing.push(link);
        }
    }
    for number in numbers(&without_links(source)) {
        if !translated_numbers.contains(&digits_of(&number)) {
            missing.push(number);
        }
    }
    let source_folded = folded(source);
    for entry in glossary.iter().filter(|entry| entry.is_usable()) {
        let term_used = source_folded.contains(&folded(entry.term.trim()));
        let preferred = entry.preferred.trim();
        if term_used && !translation_folded.contains(&folded(preferred)) {
            missing.push(preferred.to_owned());
        }
    }
    missing.dedup();
    missing
}

/// `text` prepared for comparison: lower case, and without Arabic
/// diacritics, so a term matches whether or not it is written with them.
fn folded(text: &str) -> String {
    strip_tashkeel(text).to_lowercase()
}

/// Web addresses and email addresses in `text`, without trailing punctuation.
fn links(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|word| word.contains("://") || is_email(word))
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/')
                .to_owned()
        })
        .collect()
}

fn is_email(word: &str) -> bool {
    word.split_once('@')
        .is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.'))
}

/// `text` with links removed, so the digits inside a URL are not counted as numbers.
fn without_links(text: &str) -> String {
    text.split_whitespace()
        .filter(|word| !word.contains("://") && !is_email(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Numbers in `text`, each including any separators inside it: `1,200.50`,
/// `14`, `2026-11-11`, and `1 200` where a space groups thousands, as many
/// languages write it.
fn numbers(text: &str) -> Vec<String> {
    let is_separator = |c: char| matches!(c, ',' | '.' | ':' | '-' | '/' | '\u{066B}' | '\u{066C}');
    let characters: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut current = String::new();
    for (index, &character) in characters.iter().enumerate() {
        let continues_number = !current.is_empty()
            && (is_separator(character) || groups_thousands(&characters, index));
        if character.is_numeric() || continues_number {
            current.push(character);
        } else if !current.is_empty() {
            found.push(std::mem::take(&mut current));
        }
    }
    found.push(current);

    found
        .into_iter()
        .map(|number| number.trim_end_matches(is_separator).to_owned())
        .filter(|number| !number.is_empty())
        .collect()
}

/// Whether the character at `index` is a space used as a thousands
/// separator: a (possibly non-breaking) space after a digit, followed by
/// exactly three digits.
fn groups_thousands(characters: &[char], index: usize) -> bool {
    let is_digit = |offset: usize| {
        characters
            .get(index + offset)
            .is_some_and(|c| c.is_numeric())
    };
    matches!(characters[index], ' ' | '\u{00A0}' | '\u{202F}')
        && index > 0
        && characters[index - 1].is_numeric()
        && is_digit(1)
        && is_digit(2)
        && is_digit(3)
        && !is_digit(4)
}

/// The digits of a number as ASCII, ignoring separators. Digits written in
/// another script, such as Arabic-Indic `١٢٠٠`, compare equal to `1200`, so
/// a translation may localize how a number is written without being flagged.
fn digits_of(number: &str) -> String {
    number
        .chars()
        .filter_map(|character| {
            character
                .to_digit(10)
                .or_else(|| other_script_digit(character))
        })
        .filter_map(|digit| char::from_digit(digit, 10))
        .collect()
}

/// The value of a decimal digit from the Arabic-Indic or Extended Arabic-Indic blocks.
fn other_script_digit(character: char) -> Option<u32> {
    match character {
        '\u{0660}'..='\u{0669}' => Some(character as u32 - 0x0660),
        '\u{06F0}'..='\u{06F9}' => Some(character as u32 - 0x06F0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(term: &str, preferred: &str) -> GlossaryTerm {
        GlossaryTerm {
            term: term.to_owned(),
            preferred: preferred.to_owned(),
        }
    }

    fn check(source: &str, translation: &str) -> Vec<String> {
        unpreserved(source, translation, &[])
    }

    #[test]
    fn a_faithful_translation_has_nothing_missing() {
        let source = "The project costs 1,200 USD and takes 14 days.";
        let translation = "Layihə 1,200 USD-yə başa gəlir və 14 gün çəkir.";
        assert!(check(source, translation).is_empty());
    }

    #[test]
    fn a_changed_number_is_reported() {
        let missing = check("It costs 1,200 USD.", "Qiyməti 1,300 USD-dir.");
        assert_eq!(missing, vec!["1,200"]);
    }

    #[test]
    fn a_dropped_number_is_reported() {
        assert_eq!(
            check("Delivery in 14 days.", "Çatdırılma bir neçə günə."),
            vec!["14"]
        );
    }

    #[test]
    fn a_number_may_change_its_separators() {
        // English and many European languages write the same amount differently.
        assert!(check("It costs 1,200.50.", "Qiyməti 1.200,50-dir.").is_empty());
    }

    #[test]
    fn a_space_may_group_thousands() {
        // Russian, French and others write 1,200 as "1 200", often with a non-breaking space.
        assert!(check("It costs 1,200 USD.", "Это стоит 1 200 долларов.").is_empty());
        assert!(check("It costs 1,250,000.", "Это стоит 1\u{00A0}250\u{00A0}000.").is_empty());
        // Two separate numbers are not merged into one.
        assert_eq!(
            check("Rooms 14 and 2000.", "Комнаты 14 2000 и 15."),
            Vec::<String>::new()
        );
        assert_eq!(check("It costs 1,200.", "Это стоит 1 300."), vec!["1,200"]);
    }

    #[test]
    fn digits_in_another_script_still_match() {
        assert!(
            check(
                "Delivery in 14 days for 1200 USD.",
                "التسليم خلال ١٤ يومًا بسعر ١٢٠٠ دولار."
            )
            .is_empty()
        );
    }

    #[test]
    fn dates_are_treated_as_numbers() {
        assert_eq!(
            check("Due on 2026-11-30.", "Son tarix 2026-12-30."),
            vec!["2026-11-30"]
        );
    }

    #[test]
    fn a_link_must_survive_unchanged() {
        let source = "See https://example.com/docs/v2 for details.";
        assert!(check(source, "Ətraflı: https://example.com/docs/v2.").is_empty());
        assert_eq!(
            check(source, "Ətraflı: sənədlərə baxın."),
            vec!["https://example.com/docs/v2"]
        );
    }

    #[test]
    fn digits_inside_a_link_are_not_separate_numbers() {
        let source = "See https://example.com/docs/v2 today.";
        assert_eq!(
            check(source, "Bu gün baxın."),
            vec!["https://example.com/docs/v2"]
        );
    }

    #[test]
    fn an_email_address_must_survive() {
        assert_eq!(
            check("Write to sales@example.com.", "Satış şöbəsinə yazın."),
            vec!["sales@example.com"]
        );
    }

    #[test]
    fn a_glossary_term_must_use_its_preferred_translation() {
        let glossary = [term("invoice", "hesab-faktura")];
        let source = "I will send the Invoice tomorrow.";
        assert!(unpreserved(source, "Sabah Hesab-faktura göndərəcəyəm.", &glossary).is_empty());
        assert_eq!(
            unpreserved(source, "Sabah qaimə göndərəcəyəm.", &glossary),
            vec!["hesab-faktura"]
        );
    }

    #[test]
    fn an_arabic_glossary_term_matches_with_or_without_diacritics() {
        let glossary = [term("invoice", "فاتورة")];
        let source = "I will send the invoice tomorrow.";
        assert!(unpreserved(source, "سَأُرْسِلُ الْفَاتُورَةَ غَدًا.", &glossary).is_empty());
        assert!(unpreserved(source, "سأرسل الفاتورة غدا.", &glossary).is_empty());
    }

    #[test]
    fn a_glossary_term_that_is_not_in_the_source_is_ignored() {
        let glossary = [term("invoice", "hesab-faktura")];
        assert!(unpreserved("Hello there.", "Salam.", &glossary).is_empty());
    }

    #[test]
    fn incomplete_glossary_entries_are_ignored() {
        let glossary = [term("invoice", " "), term("", "x")];
        assert!(unpreserved("The invoice.", "Qaimə.", &glossary).is_empty());
    }

    #[test]
    fn text_without_protected_parts_has_nothing_to_check() {
        assert!(check("Thanks for joining.", "Qoşulduğunuz üçün təşəkkürlər.").is_empty());
    }
}
