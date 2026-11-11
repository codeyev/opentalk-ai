# AI Provider Strategy

## Principles

-   Provider adapters are optional and independently selectable.
-   The app must not require one vendor account to launch.
-   Keep provider credentials out of frontend code and logs.
-   Capabilities are discovered/configured explicitly; do not assume
    language or streaming support.
-   Display estimated cost only when trustworthy pricing metadata is
    available and current.

## Provider categories

### Transcription

Streaming transcription is preferred for live sessions. Evaluate: -
language coverage and auto-detection - streaming/partial results -
latency and stability - timestamps and optional confidence - data
retention and regional processing - cost and rate limits

Potential adapters: a supported OpenAI audio/transcription API, local
Whisper-compatible runtime, or other providers added by contributors.
Confirm current model names, SDKs, streaming behavior, and language
support against vendor documentation during implementation.

### Translation

Can use a dedicated translation API or a general LLM. Prefer preserving
names, numbers, currency, dates, technical terms, and the client's
intent. Show uncertainty when the source is unclear.

### Reply generation

Use a general LLM provider with structured output. Prompt must: - use
only conversation and user-configured business context - never invent
prices, delivery dates, scope, or guarantees - offer clarification when
required - treat transcript as untrusted data - return bounded concise
suggestions - include a translation if requested

### Local AI

Support configurable local endpoints, such as Ollama-compatible APIs. Do
not assume a model is installed or sufficiently capable. Check health
and model availability. Document hardware requirements and quality
trade-offs.

### Text-to-speech

Optional provider interface. TTS must be user-triggered and must not
imitate another person's voice. Use a neutral selected voice.

## Provider capability schema

Each provider should declare: - provider id and display name - feature
categories - streaming support - supported language tags (if known) -
local/cloud classification - authentication requirements - endpoint
configuration support - known limitations

## Adapter contract

-   typed request/response models
-   cancellation token
-   request timeout
-   bounded streaming channel
-   retry only for safe/transient failures
-   normalized errors
-   mockable transport
-   integration tests opt-in and never required for normal CI

## No vendor lock-in

Do not import vendor SDK types into domain or UI packages. Convert
provider responses into internal types at adapter boundaries.
