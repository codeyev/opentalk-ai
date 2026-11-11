//! A bounded, rolling window of the conversation.

use std::collections::VecDeque;

use serde::Serialize;

/// Most turns kept. Older turns are forgotten first.
pub const MAX_TURNS: usize = 24;
/// Most characters kept across all turns.
pub const MAX_CHARACTERS: usize = 6_000;

/// Who said something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Speaker {
    /// The user of the app, heard through the microphone.
    Me,
    /// The other side of the conversation, heard through system audio.
    Other,
}

/// One thing that was said.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationTurn {
    pub speaker: Speaker,
    pub text: String,
}

/// The most recent part of the conversation, within fixed limits, so a long
/// session cannot grow the context without bound.
#[derive(Debug, Default)]
pub struct ConversationContext {
    turns: VecDeque<ConversationTurn>,
    characters: usize,
}

impl ConversationContext {
    /// Adds what was said and drops the oldest turns that no longer fit.
    /// Text without content is ignored.
    pub fn push(&mut self, speaker: Speaker, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        // A single turn longer than the whole budget keeps only its end.
        let length = text.chars().count();
        let text: String = text
            .chars()
            .skip(length.saturating_sub(MAX_CHARACTERS))
            .collect();

        self.characters += text.chars().count();
        self.turns.push_back(ConversationTurn { speaker, text });
        while self.turns.len() > MAX_TURNS || self.characters > MAX_CHARACTERS {
            if let Some(dropped) = self.turns.pop_front() {
                self.characters -= dropped.text.chars().count();
            }
        }
    }

    /// The remembered turns, oldest first.
    #[must_use]
    pub fn turns(&self) -> Vec<ConversationTurn> {
        self.turns.iter().cloned().collect()
    }

    /// Whether the other side has said anything that could be replied to.
    #[must_use]
    pub fn has_something_to_reply_to(&self) -> bool {
        self.turns.iter().any(|turn| turn.speaker == Speaker::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(context: &ConversationContext) -> Vec<String> {
        context.turns().into_iter().map(|turn| turn.text).collect()
    }

    #[test]
    fn keeps_turns_in_order() {
        let mut context = ConversationContext::default();
        context.push(Speaker::Other, "How much is it?");
        context.push(Speaker::Me, " Let me check. ");
        assert_eq!(
            context.turns(),
            vec![
                ConversationTurn {
                    speaker: Speaker::Other,
                    text: "How much is it?".to_owned()
                },
                ConversationTurn {
                    speaker: Speaker::Me,
                    text: "Let me check.".to_owned()
                },
            ]
        );
    }

    #[test]
    fn ignores_empty_text() {
        let mut context = ConversationContext::default();
        context.push(Speaker::Other, "   ");
        assert!(context.turns().is_empty());
    }

    #[test]
    fn forgets_the_oldest_turns_beyond_the_turn_limit() {
        let mut context = ConversationContext::default();
        for number in 0..MAX_TURNS + 3 {
            context.push(Speaker::Other, &format!("turn {number}"));
        }
        let texts = texts(&context);
        assert_eq!(texts.len(), MAX_TURNS);
        assert_eq!(texts.first().map(String::as_str), Some("turn 3"));
    }

    #[test]
    fn forgets_the_oldest_turns_beyond_the_character_limit() {
        let mut context = ConversationContext::default();
        let long = "x".repeat(MAX_CHARACTERS / 2);
        context.push(Speaker::Other, "first");
        context.push(Speaker::Other, &long);
        context.push(Speaker::Other, &long);
        assert_eq!(texts(&context), vec![long.clone(), long]);
    }

    #[test]
    fn a_single_oversized_turn_keeps_its_end() {
        let mut context = ConversationContext::default();
        let text = format!("{}END", "x".repeat(MAX_CHARACTERS));
        context.push(Speaker::Other, &text);
        let kept = &texts(&context)[0];
        assert_eq!(kept.chars().count(), MAX_CHARACTERS);
        assert!(kept.ends_with("END"));
    }

    #[test]
    fn only_the_other_side_gives_something_to_reply_to() {
        let mut context = ConversationContext::default();
        assert!(!context.has_something_to_reply_to());
        context.push(Speaker::Me, "Hello?");
        assert!(!context.has_something_to_reply_to());
        context.push(Speaker::Other, "Hi.");
        assert!(context.has_something_to_reply_to());
    }
}
