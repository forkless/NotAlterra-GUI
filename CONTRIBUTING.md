# Contributing to NotAlterra

Thanks for considering a contribution.  NotAlterra is maintained by a single
developer and every improvement — code, documentation, design, or testing —
is appreciated.

## Getting Started

1. **Fork** the repository and clone it locally.
2. **Build** with `cargo build`.
3. **Run the test suite** with `cargo test`.
4. **Read the docs** at [https://forkless.github.io/NotAlterra/notalterra/](https://forkless.github.io/NotAlterra/notalterra/).

## Making Changes

- Work on a **feature branch** from `master`.
- Keep changes focused — one logical change per pull request.
- Write or update **inline documentation** for any new or modified function.
  The check script `python3 tests/_check.py` enforces 100% doc coverage.
- Add **tests** for new behaviour.  Integration tests live in `tests/`,
  unit tests are inline in `src/`.
- Run `cargo fmt` before committing if you have `rustfmt` installed.

## Commit Convention

Commits should be atomic and descriptive.  No strict format required, but
clear messages help reviewers.  Example:

```
Add timeout to save-folder discovery scan

The scan can hang on slow network shares.  Added a
5-second per-drive timeout with a warning to the user.
```

## Pull Requests

1. Push your branch and open a PR against `master`.
2. GitHub Actions will run checks and tests automatically.
3. A maintainer will review within a few days.
4. All PRs must pass CI before merging.

## Code of Conduct

Be respectful.  Be constructive.  Assume good intent.

NotAlterra is a small project built by someone learning as they go.
Questions are welcome, patience is appreciated, and kindness is
non-negotiable.

## AI-Assisted Contributions

NotAlterra uses agentic AI tools in its development process.  If you use AI
assistance in your contribution:

- **Review everything** the AI generates before submitting.
- **Do not** submit AI-generated changes you do not understand.
- **Disclose** significant AI usage in your PR description.
- The same quality standards apply regardless of how code is generated.

## Need Help?

Open a [discussion](https://github.com/forkless/NotAlterra/discussions) or
file an issue with the "question" label.  No question is too basic.
