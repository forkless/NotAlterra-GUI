#!/usr/bin/env python3
"""Generate synthetic .sav files with fake metadata for test fixtures."""
import struct, os

DIR = "gvas-files"

def write_gvas(path, **props):
    """Write a minimal GVAS file with fake properties."""
    with open(path, "wb") as f:
        # Magic + version
        f.write(b"GVAS\x01\x00\x00\x00")
        # Package name (UE5)
        name = b"/Game/Subnautica2/Maps/Subnautica2"
        f.write(struct.pack("<i", len(name)))
        f.write(name)
        # Engine version
        f.write(struct.pack("<I", 5))
        # Custom version format
        f.write(struct.pack("<i", 3))
        # Custom versions count
        f.write(struct.pack("<i", 1))
        f.write(b"267C59A642F17F4DB528F5D0B6066E97")  # UE5 format
        f.write(struct.pack("<i", 18))
        # Properties count
        f.write(struct.pack("<i", len(props)))
        for key, val in props.items():
            key_bytes = key.encode("ascii")
            f.write(key_bytes + b"\x00")
            f.write(b"StrProperty\x00")
            f.write(struct.pack("<I", 0))  # size
            f.write(b"\x00" * 9)  # padding
            val_bytes = val.encode("utf-16-le")
            f.write(struct.pack("<i", len(val_bytes) + 1))
            f.write(val_bytes + b"\x00\x00")
        # Terminator
        f.write(b"\x00\x00\x00\x00")
        # Footer
        f.write(b"\x00" * 48)
        f.write(b"savegame_meta\x00" + struct.pack("<I", 0))

os.makedirs(DIR, exist_ok=True)

# Slot 0 — synthetic single-player save
write_gvas(os.path.join(DIR, "savegame_0.sav"),
    DisplayName="Synthetic Save 0",
    GameMode="Survival",
    PlaytimeSeconds="3600",
    IsOnline="False",
    BuildNumber="12345",
)

# Slot 1 — synthetic multiplayer save
write_gvas(os.path.join(DIR, "savegame_1.sav"),
    DisplayName="Synthetic Save 1",
    GameMode="Creative",
    PlaytimeSeconds="7200",
    IsOnline="True",
    BuildNumber="12346",
)

# Corrupt files — already synthetic, but make them more clearly fake
# ... (existing corrupt files are fine, they're small and obviously synthetic)

for f in ["savegame_0.sav", "savegame_1.sav"]:
    p = os.path.join(DIR, f)
    print(f"{p}: {os.path.getsize(p)} bytes")
