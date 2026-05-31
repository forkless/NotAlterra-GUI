import struct, os
samples = 'samples'
for f in sorted(os.listdir(samples)):
    if not f.endswith('.sav'): continue
    path = os.path.join(samples, f)
    d = open(path, 'rb').read()
    i = d.find(b'Elapsed')
    if i >= 0:
        for off in range(8, 50):
            try:
                v = struct.unpack_from('<d', d, i+off)[0]
                if 60 < v < 1000000:
                    h = int(v//3600); m = int((v%3600)//60)
                    print(f'{f}: {h}h{m}m')
            except: pass
