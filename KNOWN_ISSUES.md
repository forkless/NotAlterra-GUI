# Known Issues

## Stale config.ini from prior versions

Users upgrading from v0.3.0 or earlier will have a `config.ini` file next to
the binary that no longer serves any function. It can be safely deleted.

**Planned**: Auto-remove stale `config.ini` on first launch after upgrade.
