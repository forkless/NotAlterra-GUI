# Known Issues

## Segoe MDL2 glyphs not rendering

On some systems, the chevron glyphs (collapse/expand) and corruption icons show as Euro signs or missing-glyph boxes.

**Workaround:** Font family is hardcoded to "Segoe MDL2 Assets" in SaveSlotsPage.xaml. If the font is missing or corrupted, glyphs won't render.

## Firewall/AV false positives

The Inno Setup installer and the app itself may trigger Windows Defender or third-party AV for being unsigned/unrecognized.

**Context:** Standard practice for 1-person private open source game tools. Self-signed cert — no budget for $300/year EV certificate. User clicks "More info" → "Run anyway" on first launch.

## No .bak files in test fixtures

The gvas-files directory contains only synthetic .sav files. No .bak files exist for testing backup enumeration and recovery UI.

**Impact:** Save slots page shows slots but never displays .bak children when running against test fixtures. Works normally against real Subnautica 2 save folders.
