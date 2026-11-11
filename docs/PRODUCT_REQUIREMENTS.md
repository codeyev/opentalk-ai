# Product Requirements Document

## 1. Vision

OpenTalk AI helps a person understand and respond during a live
conversation in another language. It is a desktop companion, not an
autonomous meeting participant and not an automatic sender of messages.

## 2. Target users and scenarios

-   Freelancers and small businesses speaking with international
    clients.
-   Customer-facing professionals who need translation and response
    assistance.
-   Language learners who want to understand spoken conversations.
-   Users who need Arabic text with optional tashkeel.

Primary initial owner/use case: a solo web/mobile development freelancer
handling client discovery, pricing discussions, requirements
clarification, and follow-up questions.

## 3. Core user journey

1.  User opens the desktop app.
2.  Selects microphone and system-audio sources.
3.  Selects input language (Auto or explicit language), translation
    language, and reply language.
4.  Reviews provider and privacy mode (local/cloud).
5.  Presses Start. A prominent live indicator appears.
6.  The app captures authorized audio, segments speech, and transcribes
    it.
7.  The app shows source-language transcript and translated text.
8.  The assistant generates up to three suggested replies, each with a
    translation.
9.  User copies or edits a suggestion. The app never sends it
    automatically.
10. User presses Stop; capture ends and temporary buffers are cleared.

## 4. Functional requirements

### FR-01 Capture

-   Capture microphone audio.
-   Capture system output audio where OS permissions and APIs allow.
-   Select source(s), sample rate, channel layout, and input device
    where supported.
-   Explicit Start/Stop and clear capture status.
-   Handle device changes, permission denial, sleep/wake, and audio
    interruptions.
-   No hidden or background capture.

### FR-02 Transcription

-   Support streaming partial and final transcript segments.
-   Preserve timestamps and source labels (system/microphone).
-   Support automatic language detection where provider supports it.
-   Expose confidence/uncertainty only when provider supplies it.
-   Never fabricate missing words.

### FR-03 Translation

-   Translate finalized or stable transcript segments into a selected
    target language.
-   Preserve names, numbers, currency, dates, URLs, and technical terms.
-   Allow user-defined glossary entries.
-   Make source and translated text visually distinct.

### FR-04 Reply suggestions

-   Generate up to three concise responses:
    -   Professional
    -   Friendly/natural
    -   Short/direct
-   Replies use the selected response language.
-   Each reply can include a translation into the user's native
    language.
-   Suggestions must be grounded in the conversation and configured
    business context.
-   If price, delivery date, scope, or other facts are unknown, ask a
    clarifying question instead of inventing them.
-   User must approve/copy replies; no auto-send.

### FR-05 Arabic support

-   Arabic script and right-to-left layout.
-   Optional full/partial/off tashkeel preference.
-   MSA as a default option; dialect is explicit and configurable.
-   Tashkeel is a model-generated aid and may be imperfect; user can
    edit.
-   TTS pronunciation is optional and provider-dependent.

### FR-06 Settings

-   Select transcription, translation, and response providers
    independently where architecture permits.
-   Configure API keys securely.
-   Configure local endpoints/models (for example, Ollama) without
    assuming a model is installed.
-   Configure language, tone, glossary, context, retention, and hotkeys.
-   Display provider capabilities and limitations.

### FR-07 Session history

-   Off by default.
-   If enabled, store only the selected data categories.
-   Provide clear delete/export controls.
-   Never store raw audio unless a separate explicit feature is added
    with consent.

### FR-08 Accessibility and localization

-   Keyboard navigation and screen-reader labels.
-   High-contrast and light/dark themes.
-   UI localization architecture; initial UI languages can be English,
    Azerbaijani, and Russian.
-   Correct RTL rendering for Arabic.

## 5. Non-functional requirements

-   UI remains responsive during capture and inference.
-   Bounded audio queues and backpressure.
-   Configurable latency budgets; measure rather than promise.
-   Crash-safe capture shutdown.
-   Provider timeouts, retry policy, and cancellation.
-   No sensitive content in telemetry.
-   Cross-platform CI for build, lint, and tests.
-   Clear support matrix for OS versions and provider capabilities.

## 6. Out of scope for initial MVP

-   Automatic sending of messages or meeting chat.
-   Speaker identity recognition or biometric voice identification.
-   Call recording archive by default.
-   Mobile apps.
-   Browser extension.
-   Real-time voice cloning or impersonation.
-   Guaranteed legal/medical/financial translation accuracy.

## 7. Success criteria

-   User can start and stop capture reliably on each supported OS.
-   System and microphone audio can be independently enabled.
-   Transcript segments arrive and render without freezing the UI.
-   Translation and suggestions are linked to the correct conversation
    context.
-   User can switch providers without rewriting UI/domain logic.
-   No secret or audio content appears in logs or repository.
-   A new contributor can build the app using documented steps.
