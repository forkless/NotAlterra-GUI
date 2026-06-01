# Code Signing Policy

NotAlterra uses automated, auditable pipelines to ensure every release
artifact is traceable to its source.

## Build & Signing Process

All release binaries are built from this repository by GitHub Actions on
tag push.  The CI workflow runs on `ubuntu-latest`, installs the
required toolchain via `dtolnay/rust-toolchain@stable`, and produces
deterministic artifacts using a locked `Cargo.lock` and pinned
dependency versions.

## AI Usage Disclosure

This project uses agentic AI coding tools (e.g., DeepSeek TUI,
GitHub Copilot) as assistive aides for code generation and review.  AI
tools operate under human supervision only:

- Every code change is reviewed and committed by a human maintainer.
- AI-generated code is identified in commit history — no attempt is made
  to obscure or anonymize the source.
- All contributions pass the same lint and verification gates as any
  human-authored change.

## Integrity

AI tools do not have direct write access to the release pipeline or the
repository's tag namespace.  Builds are triggered exclusively by signed
Git tags, which can only be created by a human maintainer with access to
the project's GPG key.

## Privacy

NotAlterra does not collect, transmit, or store any personal user data.
The application runs entirely offline:

- No telemetry, no analytics, no crash reporters.
- No network requests — the binary never opens a socket.
- All configuration is stored locally in `config.ini` alongside the
  executable.

The only potentially identifying information stored is the game's
save-folder path in `config.ini`, which includes the current Windows
username.  This path never leaves the local machine — it is read once on
startup and used exclusively to locate saves and configuration files.

Because no data is collected or transmitted, there is nothing to share,
sell, or expose.  This section serves as a safe-harbor statement:
NotAlterra is designed to respect user privacy by collecting nothing at
all.

## Signing
> **Status: pending certification.**  No binaries have been signed by
> SignPath yet.  This policy exists for transparency and privacy
> documentation while the project prepares for certification.

Only binaries produced by the official CI runner from the `master`
branch will be submitted to SignPath for signing.  Manual or off-CI
builds are never shipped as signed releases.
