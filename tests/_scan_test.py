import struct
d = open('samples/savegame_0.sav','rb').read()
i = d.find(b'ElapsedTimeDouble')
print('ElapsedTimeDouble found at:', i)
i2 = d.find(b'Elapsed')
print('Elapsed found at:', i2)
# Check FName encoding at position i2-4
print('u32 at i2-4:', struct.unpack_from('<I', d, i2-4)[0])
print('Expected:', len(b'Elapsed')+1)
