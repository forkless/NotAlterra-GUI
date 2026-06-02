# Security Policy

## Reporting a Vulnerability

If you discover a security issue in NotAlterra, please report it privately:

- **Email**: forkless@protonmail.com
- **GPG**: [314BB48A3C72D8EC2830B8BED2B0DF63E2CBEA16](https://github.com/forkless.gpg)

Do not open a public issue.

## Response Timeline

- **Acknowledgment**: Within 48 hours of receipt.
- **Patch**: Fix committed within 48 hours of triage.
- **Disclosure**: Public advisory posted after the patch release ships.

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
| Latest (v0.2.x) | Yes |
| Older | No |
