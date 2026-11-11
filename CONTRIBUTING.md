# Contributing

Thanks for considering a contribution to OpenTalk AI.

## First steps

1.  Read `README.md`, `docs/ARCHITECTURE.md`, and `docs/PRIVACY.md`.
2.  Open or comment on an issue before large changes.
3.  Keep pull requests focused and explain platform impact.
4.  Do not include secrets, real transcripts, recordings, or client data
    in tests.

## Development expectations

-   Follow Rust formatting and linting.
-   Follow TypeScript strict mode and project lint rules.
-   Add tests for new behavior.
-   Update docs and supported-platform matrix.
-   Use mock providers in CI; live API tests must be opt-in.
-   Include screenshots for meaningful UI changes.
-   State which OS versions and devices were tested.

## Pull request checklist

-   [ ] Scope and rationale explained
-   [ ] Tests added/updated
-   [ ] Formatting and lint checks pass
-   [ ] No secrets or sensitive data
-   [ ] Privacy impact considered
-   [ ] macOS/Windows impact documented
-   [ ] Accessibility and localization considered
-   [ ] Documentation updated

## Provider contributions

Provider adapters must implement shared interfaces, normalize errors,
support cancellation/timeouts, declare capabilities, and document data
handling and pricing caveats. Never claim language support without
testing or provider documentation.
