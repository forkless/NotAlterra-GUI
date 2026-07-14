---
name: Pull request
about: Propose a change to NotAlterra
title: ''
labels: ''
assignees: ''
---

> **I will not respond to PRs that do not include a clear description or
> that introduce unnecessary complexity.** If you are fixing a bug, say
> what bug and how it was reproduced. If you are adding a feature, explain
> why it belongs in this tool and not in a fork.

## Description

What does this change do, and why is it needed?

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor / code quality
- [ ] Documentation
- [ ] Build / CI
- [ ] Other: ___________

## Verification

- [ ] `cargo test` passes (all tests, including integration)
- [ ] `python3 tests/_check.py` passes (100% doc coverage)
- [ ] `npm install -g @inspecode/check && check --cargo-extra-args="-- --nocapture"` passes (if applicable)
- [ ] No new compiler warnings
- [ ] CHANGELOG.md updated if the change is user-facing

## Notes (optional)

Any background context, design trade-offs, or areas that need careful
review.

---

NotAlterra is maintained by a single person. PRs may take time to review.
A polite ping after a week is fine. Please be patient.
