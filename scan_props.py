#!/usr/bin/env python3
"""Dump specific property values from GVAS files with debug."""
import struct, sys, os

data = open(sys.argv[1], 'rb').read()

def read_u32_at(off):
    if off + 4 > len(data): return None
    return struct.unpack_from('<I', data, off)[0]

def read_fname_at(off):
    slen = read_u32_at(off)
    if slen is None: return (None, off)
    if slen == 0: return (None, off + 4)
    s = data[off+4:off+4+slen-1].decode('utf-8', errors='replace')
    return (s, off + 4 + slen)

targets = [
    ('SlotName', 'StrProperty'),
    ('DisplayName', 'StrProperty'),
    ('HostAddress', 'StrProperty'),
    ('HostPort', 'IntProperty'),
    ('bIsMultiplayerSave', 'BoolProperty'),
    ('bWasMultiplayerSave', 'BoolProperty'),
    ('bOnlineSave', 'BoolProperty'),
    ('bIsUserCreated', 'BoolProperty'),
    ('LevelName', 'StrProperty'),
    ('ModeSwitchCount', 'IntProperty'),
    ('PlayTime', 'DoubleProperty'),
]

print(f"── {os.path.basename(sys.argv[1])} ──")

for prop_name, type_name in targets:
    target = prop_name.encode('utf-8')
    offset = 0
    found_val = None
    attempts = 0
    while offset < len(data) - 20 and attempts < 500:
        pos = data.find(target, offset)
        if pos < 0: break
        if pos < 4:
            offset = pos + 1; attempts += 1; continue
        # Try len(target) + 1 or len(target) (with or without null)
        name_len = read_u32_at(pos - 4)
        if name_len is None or (name_len != len(target) + 1 and name_len != len(target)):
            offset = pos + 1; attempts += 1; continue
        if data[pos + len(target)] != 0:
            offset = pos + 1; attempts += 1; continue
        after_name = pos + len(target) + 1
        (next_name, next_offset) = read_fname_at(after_name)
        if next_name != type_name:
            offset = pos + 1; attempts += 1; continue
        # Extract value
        if type_name == 'BoolProperty':
            voff = next_offset + 9
            if voff < len(data):
                found_val = 'yes' if data[voff] != 0 else 'no'
        elif type_name == 'IntProperty':
            voff = next_offset + 9
            if voff + 4 <= len(data):
                found_val = struct.unpack_from('<i', data, voff)[0]
        elif type_name == 'StrProperty':
            slen = read_u32_at(next_offset + 5)
            if slen is not None and slen > 0:
                found_val = data[next_offset+9:next_offset+9+slen-1].decode('utf-8', errors='replace')
            elif slen == 1:
                found_val = '(empty)'
        elif type_name == 'DoubleProperty':
            voff = next_offset + 5
            if voff + 8 <= len(data):
                v = struct.unpack_from('<d', data, voff)[0]
                found_val = f'{v:.0f}s ({v/3600:.1f}h)'
        break
    
    if found_val is not None:
        print(f"  {prop_name:24}  {found_val}")
