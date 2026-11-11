# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

Nothing has been released yet. The first release will be 0.1.0.

## [0.1.0] - not yet released

The first version. See `docs/KNOWN_LIMITATIONS.md` for what has and has
not been verified.

### Added

- Desktop app for macOS and Windows built with Tauri 2, React, and Rust.
- Audio capture with explicit Start and Stop, a visible capture
  indicator, and a live input level for each source: microphone, and
  system audio (ScreenCaptureKit on macOS, WASAPI loopback on Windows).
- Transcription, either on your own computer with the built-in Whisper
  speech model, or through OpenAI's streaming API.
- Translation of each finished segment, with a glossary and checks that
  numbers, links, and glossary terms were carried over.
- Up to three reply suggestions (professional, friendly, concise) that
  can be copied, edited, dismissed, or refreshed. Nothing is sent on
  your behalf.
- A choice of provider for translation and replies: OpenAI, any service
  with an OpenAI-compatible API (OpenRouter, Google Gemini, Anthropic
  Claude, DeepSeek, Groq, and others), or a local model through Ollama.
- Local-only mode, which refuses any session that would use a cloud
  service.
- Arabic output options: Modern Standard Arabic or a named dialect, and
  tashkeel off, partial, or full, with generated diacritics labelled.
- Optional speech output: a Speak button on reply suggestions, and
  reading the other side's translations aloud as they arrive. The voice
  is the system's own, ElevenLabs, or any service with an OpenAI-style
  speech API.
- API keys stored in the operating system's credential store, one per
  service.
- Interface in English, Azerbaijani, and Russian; light and dark themes;
  right-to-left preview.
