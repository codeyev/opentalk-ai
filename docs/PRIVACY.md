# Privacy

OpenTalk AI processes potentially sensitive conversations. Privacy is a
product requirement, not a marketing afterthought.

## Default behavior

-   Capture is off until the user explicitly presses Start.
-   A persistent visual indicator shows active capture.
-   Stop immediately terminates capture and processing tasks.
-   Raw audio is held in bounded memory buffers and discarded after
    processing.
-   Session history is disabled by default.
-   No analytics or telemetry containing audio, transcript, prompts,
    replies, API keys, or client details.
-   Cloud processing is never enabled silently.

## Before cloud processing

Clearly show: - which audio/text is sent - which provider receives it -
whether processing is local or cloud - provider retention/processing
policy link where available - user controls to choose another provider
or cancel

## History

If a user enables history: - explain exactly what is stored - allow
deletion of one session and all sessions - store locally - do not store
raw audio - provide export only after explicit user action

## Consent and laws

Users are responsible for complying with local recording, workplace, and
data-protection laws and obtaining any required consent from
conversation participants. The app should provide a reminder before
first capture and a persistent capture indicator. This document is not
legal advice.

## Privacy tests

-   Assert that logging redacts secrets and conversation content.
-   Assert Stop cancels capture and streaming tasks.
-   Assert history-off mode writes no session transcript.
-   Assert local-only mode makes no external network requests.

## Review against the implementation (2026-11-11)

How each commitment above is met in the current code.

| Commitment | Status |
| ---------- | ------ |
| Capture is off until Start | Met. Capture starts only from the Start button. |
| Persistent visual indicator | Met. Session state and running time are shown in the sidebar on every screen, and the workspace shows "Capturing audio" with a text label, not colour alone. |
| Stop terminates capture and processing | Met. Stop joins the capture thread before returning; pipelines, translation, and reply generation end with the session. Quitting the app does the same. |
| Raw audio only in bounded memory | Met. Queues are bounded and audio is never written to disk. |
| History disabled by default | Met, and more: there is no history feature at all. The transcript exists only on screen for the current session. |
| No telemetry | Met. The app sends no analytics and writes no logs. |
| Cloud processing never enabled silently | Met. Each cloud feature has its own switch, off by default. |
| Before cloud processing, show what is sent and to whom | Met on the Providers screen and summarised on the workspace. Links to each provider's retention policy are not shown, because the interface loads no remote content. |
| Reminder before first capture | Partly met. A consent reminder is shown above the conversation whenever capture is stopped, rather than as a one-time prompt. |
| Privacy tests | Partly met. Tests cover that nothing is sent by default, that local-only mode sends nothing to a cloud service, that keys never reach settings, and that Stop ends the session. There is no automated test that inspects network traffic. |

What is stored on the device: settings (theme, languages, provider
choices, business context, glossary) in the app's local storage; API
keys in the operating system's credential store; Whisper model files in
the app's data folder. Nothing else.

What can leave the device, and only when the user turned it on: speech
audio to the chosen transcription provider; transcript text and the
glossary to the chosen translation provider; the recent conversation and
business context to the chosen reply provider; and a model download
request to Hugging Face when the user presses Download.
