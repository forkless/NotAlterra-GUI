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

## Signing

Only binaries produced by the official CI runner from the `master`
branch will be submitted to SignPath for signing.  Manual or off-CI
builds are never shipped as signed releases.
