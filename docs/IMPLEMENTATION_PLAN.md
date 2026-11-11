# Implementation Plan

Work phase-by-phase. Each phase must compile and have tests before
moving on.

## Phase 0 --- Repository foundation

Deliver: - Tauri 2 + React + TypeScript + Rust starter. -
Workspace/repository structure and formatting/lint configuration. -
README, license, contribution, security, privacy, code of conduct. - CI
for frontend checks and Rust formatting/tests. Acceptance: - Clean clone
can install dependencies and run the starter app. - CI passes on
supported runners. - No secrets committed.

## Phase 1 --- Desktop shell and UX skeleton

Deliver: - Responsive live workspace, settings navigation, dark/light
themes. - UI localization scaffolding and RTL test page. - Mock provider
interfaces only for UI development; clearly label demo data.
Acceptance: - Keyboard-accessible controls. - Demo mode visually
distinguished from real capture. - No fake feature presented as working.

## Phase 2 --- Audio capture vertical slice

Deliver: - Audio source abstraction and Start/Stop lifecycle. - macOS
system audio and microphone capture. - Windows loopback and microphone
capture. - Device/permission error handling. Acceptance: - Real capture
tests on physical macOS and Windows machines. - Separate source tracks
where possible. - Stop reliably releases devices and tasks. - Document
OS version and permissions.

## Phase 3 --- Streaming transcription

Deliver: - Provider trait and first streaming STT adapter. -
Partial/final transcript events with timestamps. - Provider capability
display and error handling. Acceptance: - Unit tests with mocked
provider stream. - Manual tests for at least English, Russian, and
Arabic if provider supports them. - No raw audio persisted.

## Phase 4 --- Translation and language settings

Deliver: - Translation provider trait and first adapter. - Auto/explicit
source language and target language settings. - Glossary and
preservation of names/numbers. Acceptance: - Test cases for supported
language pairs. - Unsupported languages clearly reported. - RTL Arabic
text renders correctly.

## Phase 5 --- Contextual reply suggestions

Deliver: - Reply generator trait and first adapter. - Three styles:
professional, friendly, concise. - Business context editor. - Copy/edit
and refresh actions. Acceptance: - Suggestions grounded in
transcript/context. - Unknown facts trigger clarifying questions. - No
automatic sending. - Transcript is treated as untrusted data.

## Phase 6 --- Arabic quality features and optional TTS

Deliver: - MSA/dialect preference. - Tashkeel preference:
off/partial/full. - Optional speech synthesis adapter. Acceptance: - UI
warns that generated diacritics can be imperfect. - User can edit and
copy Arabic text. - TTS is optional and uses explicit provider
selection.

## Phase 7 --- Local providers and offline mode

Deliver: - Local LLM adapter (e.g. Ollama-compatible endpoint). - Local
transcription adapter only after model/runtime evaluation. - Provider
health checks and model selection. Acceptance: - Local-only mode does
not make network calls. - Missing models are explained, never silently
downloaded. - Resource usage and limitations documented.

## Phase 8 --- Hardening and release

Deliver: - Installer/package builds for macOS and Windows. -
Signing/notarization guidance; Windows signing guidance. - Crash
handling, bounded queues, cancellation, privacy review. - Screenshots,
demo video, release notes, issue templates. Acceptance: - Fresh-machine
installation tests on both OSes. - CI green. - Threat model and privacy
docs reviewed. - Known limitations published.

## Definition of Done

A feature is done only when implementation, tests, user-facing errors,
docs, accessibility, privacy review, and platform verification are
complete.
