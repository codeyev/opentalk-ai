//! Decides when suggestions are generated.
//!
//! Suggestions are refreshed after the other side finishes saying
//! something, or when the user asks. Requests are kept few:
//!
//! - A short wait after speech lets several sentences in a row lead to one
//!   request instead of one each.
//! - Automatic requests are spaced at least [`MIN_INTERVAL`] apart.
//! - Starting a new request cancels the one in flight, so suggestions for
//!   an outdated conversation are never shown.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::time::{self, Instant};

use crate::context::{ConversationContext, Speaker};
use opentalk_language::is_arabic;

use crate::provider::{ReplyEvent, ReplyGenerator, ReplyRequest, ReplySettings, ReplySuggestion};
use crate::validation;

/// How long to wait after the other side speaks before generating.
pub const SETTLE_DELAY: Duration = Duration::from_millis(800);
/// Shortest time between the starts of two automatic generations.
pub const MIN_INTERVAL: Duration = Duration::from_secs(4);

type EventHandler = Box<dyn Fn(ReplyEvent) + Send + Sync>;

/// Keeps the conversation context and produces suggestions for it.
/// Dropping the coordinator cancels any generation in progress.
pub struct ReplyCoordinator {
    shared: Arc<Shared>,
    schedule: Mutex<Schedule>,
}

struct Shared {
    generator: Arc<dyn ReplyGenerator>,
    settings: ReplySettings,
    context: Mutex<ConversationContext>,
    on_event: EventHandler,
}

#[derive(Default)]
struct Schedule {
    /// The generation waiting or in flight, if any.
    task: Option<JoinHandle<()>>,
    /// When the most recent generation was due to start.
    last_start: Option<Instant>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A poisoned lock only means another thread panicked; the data is still usable.
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl ReplyCoordinator {
    /// Creates a coordinator. `on_event` is called with every change to
    /// the suggestions. Must be used from within a Tokio runtime.
    pub fn new(
        generator: Arc<dyn ReplyGenerator>,
        settings: ReplySettings,
        on_event: impl Fn(ReplyEvent) + Send + Sync + 'static,
    ) -> Self {
        Self {
            shared: Arc::new(Shared {
                generator,
                settings,
                context: Mutex::default(),
                on_event: Box::new(on_event),
            }),
            schedule: Mutex::default(),
        }
    }

    /// Records something that was said. When it was the other side
    /// speaking, new suggestions are generated shortly afterwards.
    pub fn heard(&self, speaker: Speaker, text: &str) {
        lock(&self.shared.context).push(speaker, text);
        if speaker == Speaker::Other {
            let mut schedule = lock(&self.schedule);
            let earliest = schedule
                .last_start
                .map_or_else(Instant::now, |last| last + MIN_INTERVAL);
            let start = earliest.max(Instant::now() + SETTLE_DELAY);
            self.generate_at(&mut schedule, start);
        }
    }

    /// Generates new suggestions now, because the user asked for them.
    pub fn refresh(&self) {
        let mut schedule = lock(&self.schedule);
        self.generate_at(&mut schedule, Instant::now());
    }

    /// Replaces whatever is scheduled with a generation starting at `start`.
    fn generate_at(&self, schedule: &mut Schedule, start: Instant) {
        if !lock(&self.shared.context).has_something_to_reply_to() {
            return;
        }
        if let Some(stale) = schedule.task.take() {
            stale.abort();
        }
        schedule.last_start = Some(start);
        (self.shared.on_event)(ReplyEvent::Pending);

        let shared = Arc::clone(&self.shared);
        schedule.task = Some(tokio::spawn(async move {
            time::sleep_until(start).await;
            let request = ReplyRequest {
                settings: shared.settings.clone(),
                conversation: lock(&shared.context).turns(),
            };
            let generated = shared.generator.generate(&request).await;
            let event = match generated.and_then(validation::validate) {
                Ok(suggestions) => ReplyEvent::Ready {
                    suggestions: with_arabic_style(suggestions, &shared.settings),
                },
                Err(error) => ReplyEvent::Failed { error },
            };
            (shared.on_event)(event);
        }));
    }
}

/// Enforces the user's Arabic style on the parts of each suggestion that
/// are in Arabic. With tashkeel off this removes diacritics a provider added.
fn with_arabic_style(
    suggestions: Vec<ReplySuggestion>,
    settings: &ReplySettings,
) -> Vec<ReplySuggestion> {
    let reply_is_arabic = is_arabic(&settings.reply_language);
    let meaning_is_arabic = settings.meaning_language.as_deref().is_some_and(is_arabic);
    suggestions
        .into_iter()
        .map(|suggestion| ReplySuggestion {
            text: if reply_is_arabic {
                settings.arabic.apply_to(&suggestion.text)
            } else {
                suggestion.text
            },
            meaning: suggestion.meaning.map(|meaning| {
                if meaning_is_arabic {
                    settings.arabic.apply_to(&meaning)
                } else {
                    meaning
                }
            }),
            ..suggestion
        })
        .collect()
}

impl Drop for ReplyCoordinator {
    fn drop(&mut self) {
        if let Some(task) = lock(&self.schedule).task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;

    use super::*;
    use crate::context::ConversationTurn;
    use crate::provider::{ReplyError, ReplyProviderInfo, ReplyStyle, ReplySuggestion};

    /// Answers after `latency`, echoing how many turns it was given.
    struct FakeGenerator {
        latency: Duration,
        calls: AtomicUsize,
        result: Result<(), ReplyError>,
        last_conversation: Mutex<Vec<ConversationTurn>>,
    }

    impl FakeGenerator {
        fn new(latency: Duration) -> Arc<Self> {
            Self::with_result(latency, Ok(()))
        }

        fn with_result(latency: Duration, result: Result<(), ReplyError>) -> Arc<Self> {
            Arc::new(Self {
                latency,
                calls: AtomicUsize::new(0),
                result,
                last_conversation: Mutex::default(),
            })
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ReplyGenerator for FakeGenerator {
        fn info(&self) -> ReplyProviderInfo {
            ReplyProviderInfo {
                id: "fake",
                display_name: "Fake",
                is_cloud: false,
                accepts_arabic_dialect: false,
            }
        }

        async fn generate(
            &self,
            request: &ReplyRequest,
        ) -> Result<Vec<ReplySuggestion>, ReplyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *lock(&self.last_conversation) = request.conversation.clone();
            time::sleep(self.latency).await;
            self.result.clone()?;
            Ok(vec![ReplySuggestion {
                style: ReplyStyle::Concise,
                text: format!("reply to {} turns", request.conversation.len()),
                meaning: None,
                asks_for_missing_facts: false,
            }])
        }
    }

    type Events = Arc<Mutex<Vec<ReplyEvent>>>;

    fn coordinator(generator: Arc<FakeGenerator>) -> (ReplyCoordinator, Events) {
        let events = Events::default();
        let recorded = Arc::clone(&events);
        let coordinator =
            ReplyCoordinator::new(generator, ReplySettings::default(), move |event| {
                lock(&recorded).push(event);
            });
        (coordinator, events)
    }

    fn ready_texts(events: &Events) -> Vec<String> {
        lock(events)
            .iter()
            .filter_map(|event| match event {
                ReplyEvent::Ready { suggestions } => Some(suggestions[0].text.clone()),
                ReplyEvent::Pending | ReplyEvent::Failed { .. } => None,
            })
            .collect()
    }

    async fn wait(duration: Duration) {
        time::sleep(duration).await;
    }

    #[tokio::test(start_paused = true)]
    async fn suggestions_follow_what_the_other_side_said() {
        let generator = FakeGenerator::new(Duration::from_millis(100));
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Other, "How much does it cost?");

        assert_eq!(lock(&events).first(), Some(&ReplyEvent::Pending));
        wait(Duration::from_secs(2)).await;
        assert_eq!(ready_texts(&events), vec!["reply to 1 turns"]);
        assert_eq!(generator.calls(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn my_own_speech_is_remembered_but_does_not_trigger_suggestions() {
        let generator = FakeGenerator::new(Duration::ZERO);
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Me, "Hello, can you hear me?");
        wait(Duration::from_secs(10)).await;
        assert!(lock(&events).is_empty());
        assert_eq!(generator.calls(), 0);

        coordinator.heard(Speaker::Other, "Yes.");
        wait(Duration::from_secs(2)).await;
        assert_eq!(ready_texts(&events), vec!["reply to 2 turns"]);
    }

    #[tokio::test(start_paused = true)]
    async fn sentences_in_quick_succession_lead_to_one_request() {
        let generator = FakeGenerator::new(Duration::from_millis(100));
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Other, "We need a new store.");
        wait(Duration::from_millis(300)).await;
        coordinator.heard(Speaker::Other, "With four hundred products.");
        wait(Duration::from_secs(10)).await;

        assert_eq!(generator.calls(), 1);
        assert_eq!(ready_texts(&events), vec!["reply to 2 turns"]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_request_in_flight_is_cancelled_by_newer_speech() {
        // The provider is slow: the first answer would arrive after the second question.
        let generator = FakeGenerator::new(Duration::from_secs(3));
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Other, "First question?");
        wait(Duration::from_secs(1)).await;
        coordinator.heard(Speaker::Other, "Actually, a different question?");
        wait(Duration::from_secs(20)).await;

        // Only the answer that saw both questions is ever shown.
        assert_eq!(ready_texts(&events), vec!["reply to 2 turns"]);
    }

    #[tokio::test(start_paused = true)]
    async fn automatic_requests_are_spaced_apart() {
        let generator = FakeGenerator::new(Duration::from_millis(100));
        let (coordinator, _events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Other, "One.");
        wait(Duration::from_secs(1)).await;
        assert_eq!(generator.calls(), 1);

        // Spoken one second after the first request started: too soon for another.
        coordinator.heard(Speaker::Other, "Two.");
        wait(Duration::from_secs(1)).await;
        assert_eq!(generator.calls(), 1);

        wait(MIN_INTERVAL).await;
        assert_eq!(generator.calls(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn refresh_generates_immediately() {
        let generator = FakeGenerator::new(Duration::from_millis(100));
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.heard(Speaker::Other, "One.");
        wait(Duration::from_secs(1)).await;

        coordinator.refresh();
        wait(Duration::from_millis(200)).await;
        assert_eq!(generator.calls(), 2);
        assert_eq!(ready_texts(&events).len(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn refresh_does_nothing_before_the_other_side_has_spoken() {
        let generator = FakeGenerator::new(Duration::ZERO);
        let (coordinator, events) = coordinator(generator.clone());
        coordinator.refresh();
        wait(Duration::from_secs(1)).await;
        assert_eq!(generator.calls(), 0);
        assert!(lock(&events).is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn a_provider_error_is_reported() {
        let generator = FakeGenerator::with_result(Duration::ZERO, Err(ReplyError::RateLimited));
        let (coordinator, events) = coordinator(generator);
        coordinator.heard(Speaker::Other, "Hello?");
        wait(Duration::from_secs(2)).await;
        assert_eq!(
            lock(&events).last(),
            Some(&ReplyEvent::Failed {
                error: ReplyError::RateLimited
            })
        );
    }

    #[test]
    fn arabic_replies_follow_the_tashkeel_setting() {
        let suggestion = ReplySuggestion {
            style: ReplyStyle::Concise,
            text: "مَرْحَبًا".to_owned(),
            meaning: Some("Hello".to_owned()),
            asks_for_missing_facts: false,
        };
        let arabic_replies = ReplySettings {
            reply_language: "ar".to_owned(),
            meaning_language: Some("en".to_owned()),
            ..ReplySettings::default()
        };
        let styled = with_arabic_style(vec![suggestion.clone()], &arabic_replies);
        // Tashkeel is off by default, so the diacritics are removed; the English meaning is untouched.
        assert_eq!(styled[0].text, "مرحبا");
        assert_eq!(styled[0].meaning.as_deref(), Some("Hello"));

        let english_replies = ReplySettings {
            reply_language: "en".to_owned(),
            ..ReplySettings::default()
        };
        assert_eq!(
            with_arabic_style(vec![suggestion.clone()], &english_replies),
            vec![suggestion]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_coordinator_cancels_the_request() {
        let generator = FakeGenerator::new(Duration::from_secs(5));
        let (coordinator, events) = coordinator(generator);
        coordinator.heard(Speaker::Other, "Hello?");
        wait(Duration::from_secs(1)).await;
        drop(coordinator);
        wait(Duration::from_secs(10)).await;
        assert!(ready_texts(&events).is_empty());
    }
}
