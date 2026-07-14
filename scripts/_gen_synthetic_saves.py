#!/usr/bin/env python3
"""Generate synthetic .sav files with all 13 metadata fields populated."""
import struct, os

DIR = "gvas-files"

def write_str_prop(f, key, val):
    k = key.encode("ascii")
    f.write(struct.pack("<I", len(k) + 1))
    f.write(k + b"\x00")
    t = b"StrProperty\x00"
    f.write(struct.pack("<I", len(t)))
    f.write(t)
    f.write(struct.pack("<I", 0))  # size placeholder
    f.write(b"\x00" * 5)           # padding
    vb = val.encode("utf-16-le")
    f.write(struct.pack("<i", len(vb) + 2))
    f.write(vb + b"\x00\x00")

def write_bool_prop(f, key, val):
    k = key.encode("ascii")
    f.write(struct.pack("<I", len(k) + 1))
    f.write(k + b"\x00")
    t = b"BoolProperty\x00"
    f.write(struct.pack("<I", len(t)))
    f.write(t)
    f.write(struct.pack("<I", 1))   # value size = 1 byte
    f.write(b"\x00" * 4)             # 4 bytes field index / padding
    f.write(b"\x00")                  # 1 byte flag
    f.write(b"\x01" if val else b"\x00")

def write_int_prop(f, key, val):
    k = key.encode("ascii")
    f.write(struct.pack("<I", len(k) + 1))
    f.write(k + b"\x00")
    t = b"IntProperty\x00"
    f.write(struct.pack("<I", len(t)))
    f.write(t)
    f.write(struct.pack("<I", 4))   # value size = 4 bytes
    f.write(b"\x00" * 4)             # 4 bytes field index / padding
    f.write(b"\x00")                  # 1 byte flag
    f.write(struct.pack("<I", val))

def write_gvas(path, slot_name, display_name, game_mode, level_name,
               is_online, was_multiplayer, build_number, build_branch,
               saves_count, latest_version, data_version, playtime_seconds):
    props = [
        ("SlotName", slot_name, "str"),
        ("DisplayName", display_name, "str"),
        ("bIsMultiplayerSave", is_online, "bool"),
        ("bWasMultiplayerSave", was_multiplayer, "bool"),
        ("GameMode", game_mode, "str"),
        ("LevelName", level_name, "str"),
        ("BuildNumber", build_number, "int"),
        ("BuildBranch", build_branch, "str"),
        ("SavesCount", saves_count, "int"),
        ("LatestVersion", latest_version, "int"),
        ("DataVersion", data_version, "int"),
    ]

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
        f.write(b"267C59A642F17F4DB528F5D0B6066E97")
        f.write(struct.pack("<i", 18))
        # Properties count
        f.write(struct.pack("<i", len(props)))
        for key, val, typ in props:
            if typ == "str":
                write_str_prop(f, key, val)
            elif typ == "bool":
                write_bool_prop(f, key, val)
            elif typ == "int":
                write_int_prop(f, key, val)

        # Terminator
        f.write(b"\x00\x00\x00\x00")
        # Playtime data with "Elapsed" marker for heuristic scan
        f.write(b"Elapsed\x00")
        f.write(b"DoubleProperty\x00")
        f.write(struct.pack("<d", playtime_seconds))
        # Footer
        f.write(b"\x00" * 48)
        f.write(b"savegame_meta\x00" + struct.pack("<I", 0))

os.makedirs(DIR, exist_ok=True)

def make(path, slot, display, game, level, online, was_mp, build, branch,
         saves, version, data_ver, playtime):
    write_gvas(path, slot, display, game, level, online, was_mp, build,
               branch, saves, version, data_ver, playtime)
    print(f"{path}: {os.path.getsize(path)} bytes")

# Slot 0 — single-player Survival
make(os.path.join(DIR, "savegame_0.sav"),
     "savegame_0", "Spoonmore's Adventure", "Survival",
     "Subnautica2_Ocean", False, False, 12789, "release/5.0",
     13, 18, 7, 152280.0)

# Slot 1 — solo Creative
make(os.path.join(DIR, "savegame_1.sav"),
     "savegame_1", "Creative Test World", "Creative",
     "Subnautica2_Reef", False, False, 12790, "release/5.0",
     42, 25, 9, 88920.0)

# Slot 2 — was multiplayer, now single
make(os.path.join(DIR, "savegame_2.sav"),
     "savegame_2", "Old Multiplayer Save", "Survival",
     "Subnautica2_Deep", False, True, 12750, "release/4.9",
     67, 32, 11, 345600.0)

# Slot 3 — active multiplayer
make(os.path.join(DIR, "savegame_3.sav"),
     "savegame_3", "Co-op Run", "Hardcore",
     "Subnautica2_Lava", True, True, 12812, "experimental/5.1",
     28, 21, 8, 198000.0)

# Backups for savegame_0
make(os.path.join(DIR, "savegame_0_1.bak"),
     "savegame_0", "Spoonmore's Adventure - checkpoint", "Survival",
     "Subnautica2_Ocean", False, False, 12785, "release/5.0",
     10, 15, 7, 140580.0)

make(os.path.join(DIR, "savegame_0_2.bak"),
     "savegame_0", "Spoonmore's Adventure - earlier", "Survival",
     "Subnautica2_Ocean", False, False, 12780, "release/5.0",
     7, 12, 6, 120000.0)

# Backup for savegame_1
make(os.path.join(DIR, "savegame_1_1.bak"),
     "savegame_1", "Creative Test World - backup", "Creative",
     "Subnautica2_Reef", False, False, 12788, "release/5.0",
     38, 23, 8, 80000.0)

# Keep the existing corrupt files as-is (they're useful)

# Pad all non-corrupt saves past the 100KB corruption threshold
for pad_name in sorted(os.listdir(DIR)):
    if 'corrupt' in pad_name: continue
    if not (pad_name.endswith('.sav') or pad_name.endswith('.bak')): continue
    p = os.path.join(DIR, pad_name)
    with open(p, 'ab') as f:
        f.write(b'\x80' * max(0, 300000 - os.path.getsize(p)))