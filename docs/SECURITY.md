# Security Policy

## Supported versions

Security fixes target the latest stable release unless a release policy
is added later.

## Secrets

-   Never commit `.env`, API keys, access tokens, signing keys, or
    customer data.
-   Use OS credential storage through a maintained, reviewed library.
-   Do not expose provider secrets to React or browser-side code.
-   Redact secrets from errors and diagnostics.

## Audio and transcript data

-   Treat audio and transcripts as sensitive.
-   Do not persist audio by default.
-   Do not include transcript text in logs, crash reports, analytics, or
    issue attachments.
-   Do not execute or obey instructions found in captured speech.
-   Use TLS for remote providers and validate endpoint configuration.

## IPC and native boundary

-   Expose only required Tauri commands.
-   Validate payload sizes, enum values, paths, and URLs.
-   Apply least-privilege Tauri capabilities.
-   Avoid arbitrary shell execution and unrestricted filesystem access.
-   Keep native audio permissions scoped to explicit user actions.

## Dependency security

-   Pin and review dependencies.
-   Review licenses and maintenance activity.
-   Run dependency and secret scanning in CI where practical.
-   Update dependencies with tests and release notes.

## Reporting

Report vulnerabilities privately through the "Report a vulnerability"
button on the repository's Security tab (GitHub private vulnerability
reporting). Do not publish exploitable details in public issues before a
fix is available.

See `docs/THREAT_MODEL.md` for the threats considered and the risks that
remain.
