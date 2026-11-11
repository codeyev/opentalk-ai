# Architecture Decision Records

## ADR-001: Tauri 2 + React + TypeScript + Rust

Status: Accepted

Context: The product needs a cross-platform desktop UI and native audio
capture. The maintainer is comfortable with web development, while audio
capture requires native OS integration.

Decision: Use Tauri 2 for desktop packaging, React/TypeScript for UI,
and Rust for orchestration/domain services. Native platform code is
isolated behind adapters.

Consequences: Smaller desktop footprint than many web-runtime
alternatives, but native audio capture and packaging need
platform-specific engineering and testing.

## ADR-002: Platform audio adapters

Status: Accepted

Decision: macOS system audio via ScreenCaptureKit and Windows system
loopback via WASAPI. Microphone capture is separate. Do not pretend one
implementation covers both OSes.

## ADR-003: Provider-neutral AI layer

Status: Accepted

Decision: Define internal traits for transcription, translation, reply
generation, and optional TTS. Vendor SDKs stay inside adapters.

## ADR-004: Privacy-first defaults

Status: Accepted

Decision: Explicit capture, visible indicator, no audio persistence,
history off by default, no auto-send, cloud processing only with
user-selected provider and disclosure.

## ADR-005: MIT license

Status: Accepted

Decision: Use MIT for a permissive fully open-source codebase. Review
third-party dependencies and model licenses independently; the
repository license does not relicense external models or services.

## ADR-006: Repository layout and toolchain (Phase 0)

Status: Accepted

Decision: One repository with a pnpm workspace (`apps/*`) and a Cargo
workspace rooted at the repository. The desktop app lives in
`apps/desktop` with its Rust crate in `apps/desktop/src-tauri`. Shared
Rust crates under `crates/` are created only when a phase needs them.

App commands are declared in `src-tauri/build.rs` so that each one needs
an explicit permission in a capability file; a command that is not
granted to a window cannot be invoked from it. The production Content
Security Policy allows only bundled scripts and styles and the Tauri IPC
endpoint.

Consequences: Every new IPC command must be added to `build.rs`, the
invoke handler, and a capability file. Inline scripts and styles are
blocked in production builds.

## ADR-007: UI structure without extra libraries (Phase 1)

Status: Accepted

Decision: The UI uses no router, state-management, i18n, or CSS
library. Navigation is a single piece of React state; interface text
lives in typed dictionaries where the English file defines the keys and
TypeScript rejects a locale that misses one; styling is plain CSS with
custom properties for themes and logical properties so layouts mirror in
right-to-left mode.

Languages for transcription and translation are entered as BCP 47 tags
and validated with the platform `Intl` API rather than chosen from a
fixed list, because real support depends on the selected provider.

UI preferences are stored in the webview's local storage and validated
on load. Secrets must never be stored there.

Consequences: Few dependencies to review and update. If the app grows
deep navigation or plural-sensitive text, a router or a message-format
library may be worth adding then.

## ADR-008: Audio capture design (Phase 2)

Status: Accepted

Decision:

-   Two crates instead of the three sketched in the architecture:
    `audio-core` (platform-neutral, fully unit tested) and
    `audio-capture` (all platform adapters). The microphone adapter is
    shared by macOS and Windows, so separate per-OS crates would have
    duplicated it.
-   Microphone capture uses `cpal`, which wraps Core Audio on macOS and
    WASAPI on Windows. System audio uses ScreenCaptureKit on macOS
    through the `screencapturekit` crate, and WASAPI loopback on Windows
    through `cpal`. No private APIs are used.
-   The normalization boundary is `audio_core::normalize`: adapters
    convert to mono 32-bit float there. Sample rate is left at the
    device's native rate.
-   Capture callbacks never block. Frames go into a bounded queue; when
    it is full the new frame is dropped and counted.
-   One worker thread owns the native streams and consumes their audio.
    Stop sets a flag and joins that thread, so devices are released
    before Stop returns.
-   Microphone and system audio stay on separate tracks, each frame
    carrying its source and a monotonic timestamp.

Consequences: The workspace keeps `unsafe_code = "forbid"`; all unsafe
code is inside the reviewed dependencies. macOS builds need the Swift
toolchain, and each macOS binary adds `/usr/lib/swift` to its runtime
search path. The microphone permission state cannot be read ahead of
time without extra bindings, so on macOS a denied microphone currently
shows up as a silent level rather than an error; adding an explicit
check is tracked in the manual test matrix (case 2).

## ADR-009: Streaming transcription design (Phase 3)

Status: Accepted

Decision:

-   `crates/transcription` holds everything provider-neutral: the
    `StreamingTranscriber` trait, normalized transcript events,
    resampling, speech (turn) detection, and the pipeline. It replaces
    the separate `domain`, `application`, and `providers` crates from
    the architecture sketch until there is enough code to justify
    splitting them.
-   A provider session is a pair of bounded queues: audio in, normalized
    events out. Dropping the session cancels it. Providers never see
    capture devices and the pipeline never sees vendor types.
-   The first adapter is OpenAI's Realtime API in transcription mode
    with `gpt-live-transcribe`, chosen from the provider's current
    documentation. That model expects the client to mark the end of each
    turn, so turn detection runs locally. A side effect is that silence
    is never uploaded.
-   The resampler is a small windowed-sinc implementation in this
    repository rather than a dependency; it is covered by tests for
    level, aliasing, and chunking.
-   API keys live only in the operating system's credential store,
    through the `keyring` crate. The UI can save and delete a key but
    cannot read it.
-   One pipeline runs per audio source. A lost connection is reopened up
    to three times with increasing delay; other errors stop
    transcription for that source and are shown, while capture goes on.
-   Transcription is opt-in: it needs both a saved key and an explicit
    setting, and the provider screen states what is sent and where.

Consequences: Adding a provider means implementing one trait in a new
crate. Segment start times are approximate, because they come from local
speech detection rather than from the provider.

## ADR-010: Translation design (Phase 4)

Status: Accepted

Decision:

-   `crates/translation` holds the provider-neutral `Translator` trait,
    the glossary type, and the checks run on every finished translation.
-   Only final transcript segments are translated, one at a time and in
    order, from a bounded queue. When speech outruns translation the
    newest segments are reported as not translated rather than queued
    without limit.
-   A translation is never silently trusted: numbers, links, email
    addresses, and glossary terms from the source must be found in it,
    otherwise the UI asks the user to check it. The check compares
    digits only, so localized number formatting is accepted.
-   The first adapter uses OpenAI's Responses API with structured output
    and `store: false`. The transcript travels as a JSON field in the
    user message; the instructions tell the model to treat it as data.
-   Translation has its own opt-in switch, because it sends text to a
    cloud service, and it can only be on while transcription is on.
-   No language pair is listed as supported. The provider publishes no
    list, and nothing has been verified by test yet.

Consequences: Translation adds one request per spoken turn. A segment
whose detected or selected language already matches the target is not
sent.

## ADR-011: Reply suggestion design (Phase 5)

Status: Accepted

Decision:

-   `crates/replies` holds the provider-neutral `ReplyGenerator` trait,
    a bounded rolling conversation context, validation of what a
    generator returns, and a coordinator that decides when to generate.
-   Speech from system audio is the other party and triggers
    suggestions; speech from the microphone is the user and only adds
    context.
-   Generation waits briefly after speech so consecutive sentences share
    one request, automatic requests are spaced at least four seconds
    apart, and starting a request cancels the one in flight.
-   The model must return one draft per style in a fixed JSON schema,
    including a flag that says whether the draft asks for a missing
    fact. Output is validated and length-limited before display.
-   The conversation is passed as data inside a JSON document; the
    instructions name it untrusted and forbid inventing commercial
    facts. This reduces, but cannot eliminate, the risk of a model
    following spoken instructions or guessing, so the UI presents
    suggestions as drafts and has no send action.
-   The OpenAI translator and reply generator share one small Responses
    API client inside `provider-openai`.
-   Reply suggestions have their own opt-in switch.

Consequences: Suggestions require system audio capture. Each suggestion
round is one request carrying up to 6,000 characters of conversation.

## ADR-012: Interface design

Status: Accepted

Decision: The interface follows a plain desktop-utility look: white and
light-gray surfaces, one restrained blue accent, thin borders, almost no
shadow, system fonts. No gradients, glow, glass effects, or decorative
graphics. Dark mode redefines the same colour tokens.

-   The sidebar holds navigation and an always-visible session card
    (state and running time).
-   The live workspace has two columns that scroll independently, the
    conversation with each translation under its segment and the reply
    suggestions, above a fixed control bar with the sources and a single
    Start/Stop button.
-   The level meters show real input levels; nothing is animated for
    decoration.
-   Icons are a small set of line drawings in `components/Icon.tsx`,
    always next to a text label. Flags, globes, faces, and eyes are not
    used: a flag does not identify a language, and the others add
    nothing a label does not say.
-   The capture session lives above the screens, so visiting settings
    does not lose the session shown in the UI.

Consequences: New screens should reuse the existing tokens and building
blocks (`panel`, `field`, `button`, `note`, `warning`) rather than add
one-off styles.

## ADR-013: Arabic output and speech (Phase 6)

Status: Accepted

Decision:

-   `crates/language` holds the Arabic style (variety and tashkeel
    level) shared by translation and reply suggestions.
-   "Tashkeel off" is enforced by the app, which strips the diacritic
    code points from provider output, so that setting is a guarantee.
    "Partial" and "full" can only be requested from the model.
-   The UI labels text containing diacritics as AI-generated, by
    inspecting the text itself, so the label cannot get out of step with
    what is shown.
-   A dialect is offered only when a provider declares that it accepts a
    dialect preference. That declaration means the request is passed on,
    not that the output was verified, and the UI says so.
-   Glossary checks compare text without diacritics, so a preferred
    translation matches whether or not it is vowelled.
-   Speech output sits behind a `SpeechSynthesizer` interface in the UI.
    The one implementation uses the system's installed voices through
    the webview's speech API: it is local, needs no key, and sends
    nothing anywhere. It is opt-in and only ever starts from a button
    press. The interface is in TypeScript rather than Rust because the
    system voices are reachable from the webview without native code; a
    cloud voice would be a second implementation.

Consequences: Speech availability depends on the webview and on the
voices the user has installed. Persian and Urdu text share the Arabic
script but are left untouched by the Arabic style.

## ADR-014: Local providers and shared model tasks (Phase 7)

Status: Accepted

Decision:

-   The translation and reply tasks (instructions, input document, reply
    schema, and parser) live in the provider-neutral crates, in their
    `prompt` modules. Every language-model provider, cloud or local, is
    given the same rules.
-   `crates/provider-ollama` runs those tasks on a model in a local
    Ollama server. Its endpoint type can only hold a loopback address,
    and its HTTP client ignores system proxies, so the crate cannot send
    a request to another machine.
-   The adapter checks that the chosen model is installed before using
    it and never calls Ollama's download endpoint.
-   Model "thinking" is switched off in requests: measured with a 9B
    model it cut a reply from about a minute to a few seconds.
-   The user chooses a provider per feature. Local-only mode is enforced
    in the app core as well as in the interface.
-   Local transcription was evaluated (see `docs/LOCAL_AI.md`) but not
    implemented in this phase.

Consequences: Local-only mode cannot transcribe until a local
transcription provider exists. A local model's quality and speed depend
on what the user installed.

## ADR-015: Any OpenAI-compatible service

Status: Accepted

Decision:

-   `crates/provider-openai-compatible` runs the shared translation and
    reply tasks on any service that speaks the OpenAI Chat Completions
    format, configured with a base URL, a model name, and an API key.
    OpenRouter is the reference service; nothing in the crate is
    specific to it.
-   Only widely implemented request fields are sent. Vendor extensions
    (routing, reasoning controls, data-retention flags) are left out,
    because a field one service accepts another rejects.
-   The service is asked to enforce the reply's JSON schema; if it
    refuses, the request is retried once asking only for JSON.
-   A remote address must be `https`; `http` is accepted only for a
    server on this computer, which then counts as local for local-only
    mode.
-   Each service has its own entry in the system credential store, so a
    key is only ever sent to the service it was entered for.
-   Transcription is not covered: there is no common streaming
    speech-recognition API across these services.

Consequences: The user can choose models and prices freely, but quality,
speed, and data handling depend on the service and model they pick, and
the app cannot verify them. Per-request privacy controls of individual
gateways are not used.

## ADR-016: Built-in local transcription and open provider choice

Status: Accepted

Decision:

-   Local transcription is built into the app rather than delegated to a
    separately installed program: `crates/provider-whisper` links
    whisper.cpp through the `whisper-rs` crate. A user needs only a model
    file, on any supported computer. This replaces the earlier idea of
    talking to a separately run `whisper-server`.
-   The app offers four multilingual models with their published size
    and SHA-256 pinned in the source. A download happens only on an
    explicit button press, is verified before the file is given its
    final name, and is refused in local-only mode. Any other `ggml`
    model can be copied into the models folder.
-   The UI names a model only by file name; the core checks the name and
    resolves it inside the models folder, so it cannot be pointed at
    another file.
-   Whisper is not a streaming model, so this provider reports one final
    result per turn. It fits the existing `StreamingTranscriber` trait
    with `streams_partial_results` set to false.
-   Transcription, translation, and replies each have their own provider
    choice. For translation and replies the choice includes any
    OpenAI-compatible service; well-known services are offered as
    presets that only fill in the address.
-   API keys for those services are stored per host, so changing the
    address can never send one service's key to another.
-   The OpenAI-compatible client sends no sampling settings, because at
    least one service rejects a temperature other than its default.

Consequences: Building the app now needs CMake and a C++ toolchain on
every platform, and the Windows build of the engine is untested. The
binary is larger. Model names for third-party services are the user's
responsibility; the presets show an example only.

## ADR-017: Spoken translations and speech providers

Context: Users want to hear the other side of a call in their own
language. The system voices used so far cover few languages; neither
macOS nor Windows is known to ship an Azerbaijani voice.

Decision:

-   Reading translations aloud is a separate, off-by-default switch.
    Only translations of system audio are spoken, each once, in order.
-   Speech providers sit behind a `SpeechSynthesizer` trait in
    `crates/speech`. Every provider returns the same audio (mono 16-bit
    PCM at 24 kHz), so the UI has one player and no decoder.
-   Two providers are added: ElevenLabs, and any service with an
    OpenAI-style `/audio/speech` endpoint. The second also covers
    speech servers on the user's own computer, which keeps a fully
    local path possible beyond the system voices.
-   Requests go through the core, which holds the keys, applies
    local-only mode, and bounds the text length. The UI receives audio
    only.
-   The app plays through the webview. On macOS the app's own sound is
    already excluded from system audio capture, so a spoken translation
    is not transcribed again.
-   At most three texts wait to be spoken; older ones are skipped. Being
    current matters more than being complete, and the full text stays
    on screen.

Consequences: Spoken translations lag by a few seconds and overlap the
original voice. On Windows the loopback capture may hear the spoken
translation. Speaking the user's own words to the other side is not
part of this; it needs a virtual audio device.
