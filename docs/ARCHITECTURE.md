# Architecture

## 1. Architectural style

Use a modular desktop architecture with a thin React UI, a Rust
application core, and isolated platform adapters. AI vendors are
adapters, never dependencies of the domain model.

## 2. High-level flow

``` mermaid
flowchart TD
  UI[React UI] --> IPC[Tauri IPC Commands / Events]
  IPC --> APP[Rust Application Services]
  APP --> CAP[Audio Capture Abstraction]
  CAP --> MAC[macOS ScreenCaptureKit + AVAudioEngine]
  CAP --> WIN[Windows WASAPI Loopback + Audio Input]
  APP --> PIPE[Audio Pipeline: VAD / framing / bounded queues]
  PIPE --> STT[Transcription Provider Trait]
  STT --> SEG[Transcript Segment Store]
  SEG --> TR[Translation Provider Trait]
  SEG --> CTX[Conversation Context Builder]
  TR --> UI
  CTX --> LLM[Reply Generation Provider Trait]
  LLM --> UI
  APP --> SEC[Secure Settings / OS Keychain]
  APP --> DB[Optional Local SQLite History]
```

## 3. Workspace layout

``` text
apps/
  desktop/
    src/                       # React UI
    src-tauri/                 # Tauri bootstrap, IPC commands, capabilities
crates/
  audio-core/                  # Audio frames, normalization, bounded queue, errors
  audio-capture/               # Microphone and system-audio adapters, capture session
  transcription/               # Provider trait, transcript events, resampling, pipeline
  translation/                 # Translator trait, glossary, checks, shared model task
  replies/                     # Reply generator trait, conversation context, scheduling
  language/                    # Language tags and Arabic output style
  provider-openai/             # OpenAI: streaming transcription, translation, replies
  provider-openai-compatible/  # Any OpenAI-compatible service (OpenRouter, Gemini, ...)
  speech/                      # Speech synthesizer trait, audio format
  provider-elevenlabs/         # Speech output through ElevenLabs
  provider-ollama/             # Local translation and replies through Ollama
  provider-whisper/            # Local speech recognition, built in
  test-support/                # Loopback mock server for provider tests
docs/
.github/workflows/
```

This differs from the first sketch (separate `domain`, `application`,
`providers`, and per-OS audio crates). The reasons are recorded in
`docs/DECISIONS.md`. The boundaries are the same: the UI knows nothing
about providers' wire formats, provider crates know nothing about
capture, and vendor types never leave a provider crate.

## 4. Core domain types

Define stable, vendor-neutral types: - `AudioSource`: Microphone \|
System \| Combined (with separate tracks preferred). - `AudioFrame`:
source, monotonic timestamp, sample rate, channels, PCM format, bounded
payload. - `TranscriptSegment`: id, source, start/end timestamps, text,
language tag, partial/final state, optional provider confidence. -
`Translation`: source segment id, source language, target language,
text, provider metadata. - `ReplySuggestion`: id, style, language, text,
optional translation, grounding references, warnings. -
`ProviderCapabilities`: supported modalities, languages if known,
streaming support, local/cloud, pricing metadata if known. -
`SessionConfig`: selected sources, language settings, provider IDs,
privacy/retention settings.

Avoid storing provider-specific JSON in core domain objects. Keep opaque
metadata only at adapter boundaries.

## 5. Audio design

-   Capture adapters emit normalized frames to a bounded channel.
-   Prefer separate microphone and system tracks so the UI can label
    source accurately.
-   Define a resampling/format-normalization boundary. Do not assume
    every device outputs the same format.
-   Add VAD/segmentation as an independent stage.
-   Apply backpressure and drop policy explicitly; never allow unbounded
    memory growth.
-   Stop must cancel tasks, stop native capture, close streams, and
    clear ephemeral buffers.
-   macOS: use ScreenCaptureKit for authorized system audio and
    AVAudioEngine/Core Audio input APIs for microphone. Validate OS
    versions, entitlements, permissions, and app distribution
    requirements.
-   Windows: use WASAPI loopback for output capture and an appropriate
    Windows audio input API for microphone. Handle default-device
    changes and COM/threading requirements.
-   Never use undocumented private APIs.

## 6. Provider interfaces

Create separate traits/interfaces for: - `StreamingTranscriber` -
`Translator` - `ReplyGenerator` - `SpeechSynthesizer` (optional) -
`ProviderCatalog` / capability discovery

Provider adapters own vendor-specific request schemas, auth, retries,
streaming protocols, and rate-limit handling. Core services consume
normalized domain events.

Do not assume one provider supports all languages, live streaming,
diarization, or tashkeel. Capabilities must be explicit and testable.

## 7. Orchestration

Suggested pipeline: 1. Capture audio frames. 2. Normalize/resample and
segment. 3. Stream to selected transcription provider. 4. Render partial
transcript as provisional. 5. On final/stable segment, translate. 6.
Update bounded conversation context. 7. Generate reply suggestions only
when useful (e.g. after a question, pause, or user request), not for
every token. 8. Display suggestions with provenance/uncertainty. 9.
Cancel stale generation when newer context supersedes it.

## 8. Context management

-   Keep a bounded rolling context window.
-   Allow user-supplied business profile and glossary.
-   Separate trusted user configuration from untrusted transcript
    content.
-   Prompt-injection defense: transcript text is data, never system
    instructions.
-   Do not infer or invent prices, commitments, delivery dates, or legal
    claims.
-   Provide a "refresh suggestions" action.

## 9. Storage

-   Settings: non-secret preferences in app config.
-   Secrets: OS credential store/keychain, not frontend localStorage.
-   History: SQLite only when explicitly enabled.
-   Audio: ephemeral in-memory buffers by default.
-   Add schema migrations and deletion tests before enabling history.

## 10. UI architecture

Screens: - Live workspace - Provider and API settings - Audio
source/device settings - Language and Arabic settings - Business
context/glossary - Privacy and history - About / diagnostics

Live workspace should include: - capture state and elapsed time -
selected audio sources - transcript and translation panels - three reply
cards - copy/edit controls - latency and provider status, without
exposing secrets

## 11. Error handling

Typed errors: permission denied, device unavailable, provider
unavailable, auth failure, rate limited, unsupported language, network
timeout, malformed provider response, queue overflow, capture
interrupted.

Show actionable user-facing messages. Never display raw secrets or full
provider payloads.
