# Threat Model

Reviewed on 2026-11-11 against the code at that date. This is a
developer's review, not an external audit.

## What is being protected

| Asset | Why it matters |
| ----- | -------------- |
| Captured audio | Other people's voices and private conversations |
| Transcripts, translations, reply suggestions | The content of those conversations |
| Business context and glossary | Commercial information the user typed in |
| API keys | Access to paid accounts |
| The user's intent | The app must not record, send, or say anything the user did not choose |

## Trust boundaries

1.  **Web view and Rust core.** The interface runs in a web view and
    reaches the core only through named IPC commands.
2.  **The app and the operating system.** Microphone, system audio, and
    the credential store are gated by the operating system.
3.  **The app and remote services.** Audio or text leaves the machine
    only to a provider the user chose and turned on.
4.  **Trusted configuration and untrusted content.** Settings typed by
    the user are trusted. Everything heard in a conversation, and
    everything a provider returns, is not.

## Threats and what addresses them

### Capturing or sending without the user's knowledge

-   Capture starts only from the Start button and stops on Stop, on
    quit, and if audio processing fails. The capture state and a running
    time are always visible in the sidebar.
-   Each cloud feature has its own switch, off by default, next to a
    statement of what is sent and where.
-   Only stretches detected as speech are sent for transcription.
-   Local-only mode is enforced in the core: a session that would use a
    cloud provider is refused, whatever the interface asked for.
-   Audio is never written to disk. There is no session history.

Residual risk: the speech detector is simple, so background speech from
people who did not consent can be captured and transcribed. The app
reminds the user to obtain consent but cannot verify it.

### Content of a conversation steering the app (prompt injection)

Someone on a call can say anything, including text that looks like
instructions to a language model.

-   Spoken text is passed to models as a JSON field of data. The
    instructions say to treat it as untrusted and never follow
    instructions in it.
-   Model output must match a fixed JSON schema and is validated and
    length-limited before display.
-   Output is only ever displayed. The app has no tools, no file access
    on behalf of a model, and no way to send a message, so a model that
    is fooled can produce a bad suggestion, not an action.
-   Translations are checked for numbers, links, and glossary terms that
    went missing.

Residual risk: instructions to a model are not a security boundary. A
suggestion or translation can be wrong or manipulated. The user reads
every suggestion before using it, and the interface presents suggestions
as drafts.

### A compromised or malicious page in the web view

-   The content security policy allows only the app's own bundled
    scripts and styles and the IPC endpoint. No remote content is
    loaded.
-   Each IPC command must be granted in the window's capability file.
    Only the commands the app uses are exposed. There is no shell, file
    system, or HTTP plugin.
-   The core validates what the interface sends: language tags, model
    file names, endpoint addresses, glossary and text sizes.
-   API keys flow one way. The interface can save or delete a key and
    ask whether one exists, but cannot read one back.
-   Model files are named only by file name and resolved inside the
    app's models folder.

Residual risk: a page that could run script in the web view could start
capture or change settings through the granted commands. Keeping remote
content out is the main defence.

### Theft or leak of API keys

-   Keys are stored in the operating system's credential store, never in
    a file, in settings, or in logs.
-   A key is stored per service and sent only to that service. Changing
    a service's address cannot send its key elsewhere.
-   Remote services must use HTTPS. Plain HTTP is accepted only for a
    server on the same computer.
-   The app writes no logs, so there is nothing to redact.

Residual risk: keys are held in ordinary process memory while in use and
are not zeroed afterwards. Another process running as the same user can
ask the credential store for the key; on macOS that prompts the user
unless they have allowed it.

### Sending data to the wrong place

-   The Ollama address must be on the same computer, and requests to it
    bypass any system proxy.
-   A custom service address must be HTTPS or on the same computer, with
    no embedded credentials.
-   Local-only mode treats an address it cannot parse as a cloud
    address.

Residual risk: the user can enter the address of a service they should
not trust. The interface shows the host name the text will go to.

### A tampered model download

-   A model is downloaded only on an explicit button press, over HTTPS,
    and must match a size and SHA-256 pinned in the source code before it
    is used. A file that does not match is deleted.

Residual risk: a model file the user copies into the folder themselves
is only checked for the file format. A malformed model file is parsed by
whisper.cpp, which is native code.

### Resource exhaustion

-   Every queue between capture, transcription, translation, and replies
    is bounded. When a consumer falls behind, new audio frames or
    segments are dropped and counted rather than stored.
-   Requests have timeouts; reply generation is rate-limited and stale
    requests are cancelled; a turn of speech is cut at 30 seconds.

### Supply chain

-   Dependency versions are locked in `Cargo.lock` and `pnpm-lock.yaml`.
-   Licenses were reviewed (see `docs/RELEASE.md`).

Residual risk: there is no automated vulnerability or license scanning
in CI yet, and the builds are not reproducible or signed.

## Out of scope

-   An attacker with control of the user's operating system account.
-   The security and data handling of the providers the user chooses.
-   Legal compliance for recording conversations, which is the user's
    responsibility.
