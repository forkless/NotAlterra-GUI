# Known Issues

## Discovery module is deprecated

Auto-scan for save folders (`discovery.rs`) is deprecated in favor of the
**Set save folder** menu option. The module remains for `validate_custom_path`
and `derive_ini_path` utilities.

**Planned**: Extract the two utility functions into `config.rs`, then remove
`discovery.rs` entirely in v0.4.0.
