# Known Limitations

Last updated 2026-11-11.

## What has been verified, and where

Everything below was run on one machine: **macOS 27.2, Apple M5, 16 GB
of memory**. Nothing has been run on Windows or on any other Mac.

| Area | Verified | Not verified |
| ---- | -------- | ------------ |
| Build and automated checks | All checks pass on macOS; release build and `.dmg` built | CI has never run; Windows build |
| Microphone capture | Real input levels from the built-in microphone | Device selection, unplugging, permission denied |
| System audio capture (macOS) | The permission-denied path | Capture with permission granted |
| Audio capture (Windows) | Compiles for Windows | Never run |
| Local transcription (Whisper) | Correct transcripts of synthetic English, Russian, and Arabic speech with the large-v3-turbo model, through the full pipeline; live microphone speech transcribed in the packaged app | Accuracy on real conversations, smaller models, the in-app download |
| Local translation and replies (Ollama) | One translation and one set of replies with a real 9B model | Other models and languages |
| OpenAI transcription, translation, replies | The endpoints reject an invalid key as expected | Any real result: no real key was used |
| Other AI services | OpenRouter rejects an invalid key as expected | Any real result; Gemini, Claude, DeepSeek, and Groq addresses were never called |
| Interface | Viewed in a browser in light and dark themes; component tests; the packaged app used by hand for a local session (microphone, Whisper, Ollama translation) | A full pass through every screen; a screen reader |
| Speech output, spoken translations | Against a fake speech API and mock ElevenLabs and OpenAI-style servers | Whether the app's web view exposes system voices; ElevenLabs and OpenAI speech against the live services; how Azerbaijani sounds; playback in the packaged app |
| Arabic output | Mechanics: diacritic removal, labels, copy order | The Arabic itself; no native-speaker review; no dialect |
| Installer | Unsigned `.dmg` builds and the app in it starts | Signing, notarization, install on a clean machine, Windows installer |

## Platform support

**No platform is claimed as supported yet.** macOS is the development
platform and the furthest along. Windows code exists and compiles in
part, but has never been built as a whole or run.

Intended minimums: macOS 13 (for system audio through ScreenCaptureKit)
and Windows 10 version 1803 or later.

## Functional limits

-   **Azerbaijani is often detected as Turkish** by Whisper when the
    spoken language is left on automatic. Set the spoken language to
    `az` under Languages. Accuracy for Azerbaijani has not been
    measured.
-   **Transcription quality depends on audio quality.** Speech is found
    with a simple loudness-based detector. Quiet speech can be missed,
    loud background noise can be sent as speech, and long sentences are
    cut at 30 seconds.
-   **Whisper gives no live partial text.** Text appears when the
    speaker pauses.
-   **Segment times are approximate**, because they come from the app's
    own speech detection.
-   **Reply suggestions need system audio.** They answer the other
    party, which is whatever the computer plays. Microphone-only capture
    produces none.
-   **Suggestions and translations can be wrong.** The rules given to
    the model (no invented prices or dates, ask when a fact is missing,
    ignore instructions in speech) reduce mistakes but do not prevent
    them. The number and glossary check catches only some errors and can
    flag a correct translation that writes a number in words.
-   **A denied microphone looks like silence on macOS** rather than
    producing an error.
-   **No session history, no export, no auto-update.**
-   **Model names for third-party services are entered by hand**, and
    the examples shown may be out of date.
-   **Anthropic's OpenAI-compatible endpoint** is described by Anthropic
    as intended for testing rather than production use.

## Security and privacy limits

See `docs/THREAT_MODEL.md` for the full list. In short: instructions to
a language model are not a security boundary; API keys are held in
process memory while in use; there is no automated dependency scanning;
builds are unsigned.

## Building

-   On macOS the Whisper engine needs the compiler's runtime library
    linked in when the app targets an older macOS than the build
    machine; `crates/provider-whisper/build.rs` does this.
-   Needs CMake and a C++ compiler for the built-in Whisper engine, and
    on macOS the Swift toolchain for ScreenCaptureKit.
-   The release profile does not strip the binary. On macOS 27, with a
    minimum macOS version set, the strip step corrupted the libraries
    used during the build.
