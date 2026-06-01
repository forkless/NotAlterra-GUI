# Governance

NotAlterra is maintained by a single developer.  This document describes how
decisions are made, how access is controlled, and what happens if the
maintainer becomes unavailable.

## Decision Making

| Area | Process |
|---|---|
| Feature scope | Maintainer decides. Community input via issues and discussions is encouraged but non-binding. |
| Code review | All changes pass through CI (`cargo check`, `cargo test`, `cargo doc`). Human review is performed by the maintainer before signing. |
| Release | Signed GPG tag by the maintainer. No automated tag creation. CI builds, packages, and attaches provenance. |
| Policy documents | Maintainer drafts. Significant changes are committed with justification in the commit message. |
| Security issues | Reported via email. Patched within 48 hours. Disclosed publicly after patch release. |

## Maintainer

- **GitHub**: [forkless](https://github.com/forkless)
- **Contact**: forkless@proton.me
- **GPG key**: [314BB48A3C72D8EC2830B8BED2B0DF63E2CBEA16](https://github.com/forkless.gpg)

## Bus Factor

NotAlterra has a bus factor of one — only the maintainer holds the GPG key
and push access to the repository.

### Access Recovery Plan

If the maintainer becomes unavailable for an extended period (unreachable
for 90+ days with no public activity), the following steps are available to
the community:

1. **Fork the repository.**  All code, documentation, and build scripts are
   publicly available under the MIT license.  The project can be continued
   under new maintainership.

2. **Contact GitHub Support.**  Repository transfer can be requested through
   GitHub's deceased user policy or owner unreachability process.

3. **Replace the GPG key.**  The signing key belongs to the maintainer and
   cannot be transferred.  A new maintainer should generate a new key, add
   it to the CI secrets, and update this document.

4. **Re-establish provenance.**  SLSA provenance will need to be regenerated
   under the new maintainer's identity.  Historical provenance for prior
   releases remains valid.

### What the Maintainer Periodically Verifies

- GPG key expiration (checked quarterly).
- CI pipeline is functional (every commit push triggers it).
- Backup of repository and signing subkey exists in offline storage.

## Code of Conduct

Be respectful.  Be constructive.  Assume good intent.

This project is maintained by someone learning as they go.  Questions are
welcome.  Patience is appreciated.  Kindness is non-negotiable.

## Changes to This Document

This document is versioned with the repository.  Proposed changes should be
filed as pull requests.  The maintainer has final approval.

Last updated: 2026-06-01.
