import re, struct

c = open('gvas-files/_extracted/savegame_1.sav','rb').read()
s = open('gvas-files/_extracted/savegame_0.sav','rb').read()

for label, data in [('Survival', s), ('Creative', c)]:
    print(f'\n=== {label} ===')
    for m in re.finditer(b'Elapsed', data):
        pos = m.start()
        chunk = data[pos:pos+80]
        # Look for property type string nearby
        for t in [b'FloatProperty\x00', b'DoubleProperty\x00', b'IntProperty\x00', b'ByteProperty\x00']:
            tpos = chunk.find(t)
            if tpos >= 0:
                # Check what value follows the type
                val_start = pos + tpos + len(t)
                if t in [b'FloatProperty\x00', b'IntProperty\x00']:
                    val = struct.unpack('<f', data[val_start:val_start+4])[0]
                    print(f'  Elapsed at {pos}, type {t[:12]} at offset {tpos}, val={val:.2f} ({val/60:.1f}m)')
                elif t == b'ByteProperty\x00':
                    print(f'  Elapsed at {pos}, type Byte, val={data[val_start]}')
                break
        else:
            print(f'  Elapsed at {pos}: no known type found nearby')
            print(f'    hex: {chunk[:60].hex()}')
