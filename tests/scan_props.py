#!/usr/bin/env python3
"""Extract GVAS property values using correct offsets."""
import struct, sys

data = open(sys.argv[1], 'rb').read()

def u32(off):
    if off + 4 > len(data): return None
    return struct.unpack_from('<I', data, off)[0]

def read_fname(off):
    slen = u32(off)
    if slen is None: return (None, off)
    if slen == 0: return (None, off + 4)
    s = data[off+4:off+4+slen-1].decode('utf-8', errors='replace')
    return (s, off + 4 + slen)

def get(prop_name, type_name):
    target = prop_name.encode('utf-8')
    off = 0
    while off + 20 < len(data):
        pos = data.find(target, off)
        if pos < 0: return None
        if pos < 4: off = pos + 1; continue
        nl = u32(pos - 4)
        if nl is None or nl != len(target) + 1: off = pos + 1; continue
        if data[pos + len(target)] != 0: off = pos + 1; continue
        after = pos + len(target) + 1
        (tn, no) = read_fname(after)
        if tn != type_name: off = pos + 1; continue
        val_off = no + 9
        if type_name == 'BoolProperty':
            return 'yes' if val_off < len(data) and data[val_off] != 0 else 'no'
        elif type_name == 'IntProperty':
            if val_off + 4 <= len(data):
                return str(struct.unpack_from('<i', data, val_off)[0])
        elif type_name == 'StrProperty':
            slen = u32(val_off)
            if slen and slen > 1:
                return data[val_off+4:val_off+4+slen-1].decode('utf-8', errors='replace')
            elif slen == 1:
                return '(empty)'
        elif type_name == 'DoubleProperty':
            if val_off + 8 <= len(data):
                v = struct.unpack_from('<d', data, val_off)[0]
                return f'{v:.0f}s'
        return '✓'
    return None

# Only what was asked for
for name, typ in [('HostAddress', 'StrProperty'), ('HostPort', 'IntProperty'),
                  ('bIsUserCreated', 'BoolProperty')]:
    v = get(name, typ)
    if v is not None:
        print(f"  {name:24}  {v}")
    else:
        print(f"  {name:24}  (not found)")
