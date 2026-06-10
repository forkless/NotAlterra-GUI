# Security Policy

## Reporting a Vulnerability

If you discover a security issue in NotAlterra, report it privately
through one of the channels below. Do **not** open a public issue.

### Preferred: GitHub Private Vulnerability Reporting

1. Go to the repository's **Security** tab:
   <https://github.com/forkless/NotAlterra/security/advisories>
2. Click **Report a vulnerability**.
3. Fill in the form — no GPG needed, the thread is private by default.
4. GitHub can assign a CVE ID directly through their CNA.

### Fallback: Email

If you cannot use the GitHub advisory form, email the maintainer directly:

- **Email**: forkless@protonmail.com
- **GPG**: [314BB48A3C72D8EC2830B8BED2B0DF63E2CBEA16](https://github.com/forkless.gpg)

Encrypted email is preferred when the report includes sensitive details
or proof-of-concept code.

## Safe Harbor

If you report a vulnerability in good faith and follow this policy —
report privately, allow time for a patch, do not publish exploit code
before a release — NotAlterra will not pursue legal action against you.
Your testing is authorized within the scope defined below. No other
authorization, express or implied, is granted.

## Response Timeline

- **Acknowledgment**: Within 48 hours of receipt.
- **Patch**: Fix committed within 48 hours of triage.
- **Disclosure**: Public advisory posted after the patch release ships.

These are best-effort targets. NotAlterra is maintained by a single
person — real-life delays happen. If you haven't heard back within the
timeline, a polite follow-up is welcome.

## Scope

NotAlterra is an offline desktop application.  Security concerns include
but are not limited to:

- Unintended file writes outside declared paths.
- Path traversal in save-folder or backup-folder handling.
- Silent data corruption during backup or restore.
- Dependency vulnerabilities (monitored via `cargo-deny`).

## Out of Scope

- Issues requiring physical access to the user's machine.
- Social engineering attacks.
- Malicious Subnautica 2 save files deliberately crafted to crash the
  parser (GVAS parsing errors are handled gracefully — no unsafe code).

## Supported Versions

Only the latest release receives security patches.

| Version | Supported |
|---|---|
| Latest | Yes |
| Older | No |
