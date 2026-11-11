# Release and Distribution

## Current state

| Step | macOS | Windows |
| ---- | ----- | ------- |
| Release build of the app | Done on macOS 27.2 (Apple silicon) | Not yet tried |
| Installer | `.dmg` built (3.6 MB, ad-hoc signed by the linker only); the app inside starts | Not yet built |
| Code signing | Not done: needs the maintainer's Apple Developer ID | Not done: needs a code-signing certificate |
| Notarization | Not done | Not applicable |
| Install and uninstall on a clean machine | Not done | Not done |

No platform is described as supported until its row is complete. See
`docs/KNOWN_LIMITATIONS.md`.

## Building the installers

Prerequisites are in the README. Then, from the repository root:

``` text
pnpm install
pnpm build
```

The installers are written under `target/release/bundle/`:

-   macOS: `macos/OpenTalk AI.app` and `dmg/OpenTalk AI_<version>_<arch>.dmg`
-   Windows: `nsis/*-setup.exe` and `msi/*.msi` (MSI can only be built on
    Windows)

The same build runs in GitHub Actions for a tag that starts with `v`, or
on request, through `.github/workflows/release.yml`, and uploads the
installers as workflow artifacts.

## Release checklist

-   [ ] Version updated in `package.json`, `apps/desktop/package.json`,
    `Cargo.toml` (workspace), and `apps/desktop/src-tauri/tauri.conf.json`
-   [ ] `CHANGELOG.md` updated
-   [ ] `pnpm check` passes locally, and CI passes on macOS and Windows
-   [ ] Manual test matrices in `docs/TESTING.md` run on both platforms,
    with OS versions recorded
-   [ ] Dependency licenses reviewed (see below)
-   [ ] `docs/THREAT_MODEL.md` and `docs/PRIVACY.md` reviewed against the
    changes in this release
-   [ ] No secrets, recordings, or transcripts in the repository or the
    build artifacts
-   [ ] `docs/KNOWN_LIMITATIONS.md` updated with the exact OS versions
    tested
-   [ ] Installers signed, and the macOS one notarized
-   [ ] Install, first run, upgrade, and uninstall tested on a clean
    machine for each platform
-   [ ] SHA-256 checksums of the installers published with the release

## macOS signing and notarization

An unsigned app works on the machine that built it. Downloaded from the
internet, macOS will refuse to open it until the user overrides
Gatekeeper. Do not publish instructions for bypassing Gatekeeper as the
normal way to install; sign and notarize instead.

What is needed:

1.  An Apple Developer Program membership. A free account cannot
    notarize.
2.  A **Developer ID Application** certificate, the type for apps
    shipped outside the App Store.
3.  The certificate exported as a `.p12` file with a password.

Tauri's bundler reads these environment variables:

| Variable | Contents |
| -------- | -------- |
| `APPLE_CERTIFICATE` | The `.p12` file, base64 encoded (`openssl base64 -A -in certificate.p12`) |
| `APPLE_CERTIFICATE_PASSWORD` | The password set when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | The identity name, as listed by `security find-identity -v -p codesigning` |
| `APPLE_ID` | The Apple account email, for notarization |
| `APPLE_PASSWORD` | An app-specific password for that account |
| `APPLE_TEAM_ID` | The Apple Team ID |

Notarization can use an App Store Connect API key instead of the Apple
ID: `APPLE_API_ISSUER`, `APPLE_API_KEY`, and `APPLE_API_KEY_PATH`.

For CI, add these as repository secrets with the same names; the release
workflow passes them to the build. Never commit them.

Things specific to this app:

-   `apps/desktop/src-tauri/entitlements.plist` requests the audio-input
    entitlement, which a signed build with the hardened runtime needs in
    order to use the microphone.
-   `apps/desktop/src-tauri/Info.plist` holds the microphone usage text
    shown in the permission prompt.
-   System audio capture asks for the "Screen & System Audio Recording"
    permission the first time it is used. macOS ties that permission to
    the app's signature, so an unsigned or re-signed build may be asked
    again.
-   The minimum system version is macOS 13, where ScreenCaptureKit audio
    capture is available.

None of the signing path has been exercised yet.

## Windows signing

An unsigned installer triggers a Microsoft SmartScreen warning, and Smart
App Control can block it outright. Signing needs a code-signing
certificate from a certificate authority. Tauri's documentation notes
that since 2024 an EV certificate no longer gives immediate SmartScreen
reputation: OV and EV certificates build reputation the same way, and a
newly signed release can still show a warning.

Configuration lives under `bundle.windows` in `tauri.conf.json`:
`certificateThumbprint`, `digestAlgorithm` (normally `sha256`), and
`timestampUrl`; or `signCommand` to call another signing tool, which is
how cloud signing services such as Azure Key Vault or Azure Artifact
Signing are used. Follow the certificate issuer's instructions for
certificates issued after June 2023.

The installer uses Tauri's default WebView2 mode, which downloads the
WebView2 runtime during installation if the machine lacks it. Windows 10
(1803 and later) and Windows 11 normally have it already.

None of this has been exercised yet: the app has not been built on
Windows.

## Dependency licenses

Reviewed on 2026-11-11 from `cargo metadata`: 541 third-party Rust
crates, all under permissive licenses (MIT, Apache-2.0, BSD, ISC, Zlib,
Unicode, Unlicense, CC0), except seven under MPL-2.0, a file-level
copyleft license that allows use in this project unmodified. No GPL or
AGPL dependencies. Two crates offer LGPL as one of several choices and
are used under MIT or Apache-2.0.

Bundled native code: whisper.cpp (MIT), built into the app. Whisper
model files are not part of the app; they are published by OpenAI under
the MIT license and downloaded by the user.

JavaScript runtime dependencies are React, React DOM, and the Tauri API
package, all MIT or Apache-2.0.

Re-run the review when dependencies change. Automated license and
vulnerability scanning in CI (for example `cargo deny` and
`cargo audit`) is not set up yet.

## Updates

There is no auto-update. Before adding one, review Tauri updater
signing, key handling, update metadata integrity, and rollback. Never
ship an unsigned or unverified update mechanism.
