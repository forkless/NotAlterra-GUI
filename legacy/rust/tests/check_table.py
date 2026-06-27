#!/usr/bin/env python3
# Check alignment of last two table rows
c1, c2 = 33, 56
# Build the last three rows exactly as rendered
rows = [
    "\u2502 Stale config.ini from" + " " * (c1 - len(" Stale config.ini from")) + "\u2502 Resolved in v0.4.0 - auto-removed" + " " * (c2 - len(" Resolved in v0.4.0 - auto-removed")) + "\u2502",
    "\u2502 prior versions" + " " * (c1 - len(" prior versions")) + "\u2502 on first launch" + " " * (c2 - len(" on first launch")) + "\u2502",
    "\u2514" + "\u2500" * c1 + "\u2534" + "\u2500" * c2 + "\u2518",
]
for l in rows:
    print(f"len={len(l):3d}  {l}")
